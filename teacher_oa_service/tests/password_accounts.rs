#[macro_use]
mod common;

use actix_web::{http::StatusCode, test};
use sea_orm::EntityTrait;
use serde_json::{Value, json};
use teacher_oa_service::{
    repositories::entities::login_settings,
    services::{account_service as accounts, auth_service, settings_service},
};

fn account(username: &str, role: &str) -> accounts::AccountInput {
    accounts::AccountInput {
        id: None,
        username: username.into(),
        name: "正式用户".into(),
        department: "教务处".into(),
        role: role.into(),
        active: true,
        password: Some("Initial-password-123".into()),
        wecom_userid: None,
    }
}

#[actix_web::test]
async fn password_login_requires_change_and_revokes_previous_sessions() {
    let (state, _database) = common::setup().await;
    let id = accounts::save(
        state.db_handles.writer(),
        None,
        "",
        account("admin", "admin"),
    )
    .await
    .unwrap();
    let app = create_test_app!(state.clone());
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .insert_header(("Origin", "https://untrusted.example"))
            .set_json(json!({"username":"admin","password":"Initial-password-123"}))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"username":"ADMIN","password":"Initial-password-123"}))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .response()
        .cookies()
        .find(|c| c.name() == auth_service::SESSION_COOKIE)
        .unwrap()
        .into_owned();
    let body: Value = test::read_body_json(response).await;
    assert_eq!(body["data"]["must_change_password"], true);
    let csrf = body["data"]["csrf_token"].as_str().unwrap();
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/admin/accounts")
                .cookie(cookie.clone())
                .to_request()
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let response = test::call_service(&app, test::TestRequest::post().uri("/api/v1/auth/password")
        .cookie(cookie.clone()).insert_header(("Origin", "http://127.0.0.1:3000"))
        .set_json(json!({"current_password":"Initial-password-123","new_password":"New-password-456"})).to_request()).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = test::call_service(&app, test::TestRequest::post().uri("/api/v1/auth/password")
        .cookie(cookie.clone()).insert_header(("Origin", "http://127.0.0.1:3000")).insert_header(("X-CSRF-Token", csrf))
        .set_json(json!({"current_password":"Initial-password-123","new_password":"New-password-456"})).to_request()).await;
    assert_eq!(response.status(), StatusCode::OK);
    let new_cookie = response
        .response()
        .cookies()
        .find(|c| c.name() == auth_service::SESSION_COOKIE)
        .unwrap()
        .into_owned();
    let body: Value = test::read_body_json(response).await;
    assert_eq!(body["data"]["must_change_password"], false);
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/me")
                .cookie(cookie)
                .to_request()
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/admin/accounts")
                .cookie(new_cookie.clone())
                .to_request()
        )
        .await
        .status(),
        StatusCode::OK
    );
    let mut reset = account("admin", "admin");
    reset.id = Some(id);
    accounts::save(state.db_handles.writer(), None, "", reset)
        .await
        .unwrap();
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/me")
                .cookie(new_cookie)
                .to_request()
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[actix_web::test]
async fn wrong_password_lockout_and_teacher_admin_denial() {
    let (state, _database) = common::setup().await;
    let db = state.db_handles.writer();
    let id = accounts::save(db, None, "", account("teacher", "teacher"))
        .await
        .unwrap();
    for _ in 0..5 {
        assert!(
            accounts::login(
                db,
                accounts::PasswordLogin {
                    username: "teacher".into(),
                    password: "incorrect".into()
                }
            )
            .await
            .is_err()
        );
    }
    assert!(
        accounts::login(
            db,
            accounts::PasswordLogin {
                username: "teacher".into(),
                password: "Initial-password-123".into()
            }
        )
        .await
        .is_err()
    );
    let row = accounts::credential_for(db, id).await.unwrap().unwrap();
    assert!(row.locked_until.is_some());
    assert!(row.password_hash.starts_with("$argon2id$"));
    accounts::change(
        db,
        id,
        accounts::PasswordChange {
            current_password: "Initial-password-123".into(),
            new_password: "Changed-password-456".into(),
        },
    )
    .await
    .unwrap();
    let app = create_test_app!(state);
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"username":"teacher","password":"Changed-password-456"}))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .response()
        .cookies()
        .find(|c| c.name() == auth_service::SESSION_COOKIE)
        .unwrap()
        .into_owned();
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/admin/wecom")
                .cookie(cookie)
                .to_request()
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
}

#[actix_web::test]
async fn wecom_secret_encrypted_versioned_and_binding_unique() {
    let (mut state, _database) = common::setup().await;
    let directory = tempfile::tempdir().unwrap();
    std::sync::Arc::make_mut(&mut state.config).storage.root =
        directory.path().to_string_lossy().into();
    let id = accounts::save(
        state.db_handles.writer(),
        None,
        "",
        account("admin", "admin"),
    )
    .await
    .unwrap();
    let save = || settings_service::SaveWecomSettings {
        enabled: true,
        corp_id: "wwExampleCorp".into(),
        agent_id: "1000002".into(),
        secret: Some("private-wecom-secret".into()),
        version: 0,
    };
    let settings = settings_service::save(&state, id, save()).await.unwrap();
    let value = serde_json::to_string(&settings).unwrap();
    assert!(settings.enabled && settings.secret_configured);
    assert!(!value.contains("private-wecom-secret"));
    let row = login_settings::Entity::find_by_id(1)
        .one(state.db_handles.writer())
        .await
        .unwrap()
        .unwrap();
    assert!(!row.secret_ciphertext.contains("private-wecom-secret"));
    assert!(settings_service::save(&state, id, save()).await.is_err());
    let mut keep = save();
    keep.secret = None;
    keep.version = 1;
    settings_service::save(&state, id, keep).await.unwrap();
    let preserved = login_settings::Entity::find_by_id(1)
        .one(state.db_handles.writer())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.secret_ciphertext, preserved.secret_ciphertext);
    let mut first = account("member1", "teacher");
    first.wecom_userid = Some("zhangsan".into());
    accounts::save(state.db_handles.writer(), Some(id), "wwExampleCorp", first)
        .await
        .unwrap();
    let mut duplicate = account("member2", "teacher");
    duplicate.wecom_userid = Some("zhangsan".into());
    assert!(
        accounts::save(
            state.db_handles.writer(),
            Some(id),
            "wwExampleCorp",
            duplicate
        )
        .await
        .is_err()
    );
    assert_eq!(
        accounts::accounts(state.db_handles.writer())
            .await
            .unwrap()
            .items
            .len(),
        2
    );
}

#[actix_web::test]
async fn admin_role_and_status_changes_apply_to_existing_sessions() {
    let (state, _database) = common::setup().await;
    let db = state.db_handles.writer();
    let id = accounts::save(db, None, "", account("reviewer", "reviewer"))
        .await
        .unwrap();
    let app = create_test_app!(state.clone());
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"username":"reviewer","password":"Initial-password-123"}))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .response()
        .cookies()
        .find(|c| c.name() == auth_service::SESSION_COOKIE)
        .unwrap()
        .into_owned();
    let grants = || async {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/me")
                .cookie(cookie.clone())
                .to_request(),
        )
        .await;
        let status = response.status();
        let body: Value = test::read_body_json(response).await;
        (status, body["data"]["user"]["grants"].to_string())
    };
    let (status, before) = grants().await;
    assert_eq!(status, StatusCode::OK);
    assert!(before.contains("reviews:write"));

    let mut demote = account("reviewer", "teacher");
    demote.id = Some(id);
    demote.password = None;
    accounts::save(db, None, "", demote).await.unwrap();
    let (status, after) = grants().await;
    assert_eq!(status, StatusCode::OK);
    assert!(!after.contains("reviews:write"));

    let mut disable = account("reviewer", "teacher");
    disable.id = Some(id);
    disable.password = None;
    disable.active = false;
    accounts::save(db, None, "", disable).await.unwrap();
    assert_eq!(grants().await.0, StatusCode::FORBIDDEN);

    let mut enable = account("reviewer", "teacher");
    enable.id = Some(id);
    enable.password = None;
    accounts::save(db, None, "", enable).await.unwrap();
    assert_eq!(grants().await.0, StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn shared_proxy_ip_does_not_throttle_other_accounts() {
    let (state, _database) = common::setup().await;
    let db = state.db_handles.writer();
    let app = create_test_app!(state.clone());
    let login = |username: String, password: &'static str| {
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .peer_addr("10.20.0.1:40000".parse().unwrap())
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"username":username,"password":password}))
            .to_request()
    };
    // More teachers than the old shared per-IP limit of 20 attempts per minute.
    for n in 0..25 {
        let username = format!("shared{n:02}");
        accounts::save(db, None, "", account(&username, "teacher"))
            .await
            .unwrap();
        let response = test::call_service(&app, login(username, "Initial-password-123")).await;
        assert_eq!(response.status(), StatusCode::OK, "teacher {n}");
    }
    // shared00 already used one of its ten attempts on the successful login above.
    for _ in 0..9 {
        let response = test::call_service(&app, login("shared00".into(), "wrong-password")).await;
        assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
    let response = test::call_service(&app, login("SHARED00 ".into(), "wrong-password")).await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let response = test::call_service(&app, login("shared01".into(), "Initial-password-123")).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[actix_web::test]
async fn wecom_login_entry_is_rate_limited_per_client() {
    let (state, _database) = common::setup().await;
    let app = create_test_app!(state);
    let entry = |peer: &str| {
        test::TestRequest::get()
            .uri("/api/v1/auth/wecom/login")
            .peer_addr(peer.parse().unwrap())
            .to_request()
    };
    for _ in 0..300 {
        let response = test::call_service(&app, entry("10.30.0.1:40000")).await;
        assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
    let response = test::call_service(&app, entry("10.30.0.1:40000")).await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let response = test::call_service(&app, entry("10.30.0.2:40000")).await;
    assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}
