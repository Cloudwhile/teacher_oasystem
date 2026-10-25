#[macro_use]
mod common;

use actix_web::{cookie::Cookie, http::StatusCode, test};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
use serde_json::{Value, json};
use std::sync::Arc;
use teacher_oa_service::{
    api::error::ApiError,
    integrations::wecom::{LoginMode, WeComAdapter, WeComIdentity},
    services::auth_service,
};

async fn setup_auth() -> (
    teacher_oa_service::runtime::AppState,
    aster_forge_test::temp::SqliteTestDatabase,
) {
    let (mut state, database) = common::setup().await;
    Arc::make_mut(&mut state.config).auth.mock_enabled = true;
    teacher_oa_service::services::development_seed::seed(state.db_handles.writer())
        .await
        .expect("seed identities");
    (state, database)
}

macro_rules! login {
    ($app:expr, $identity:expr) => {{
        let response = test::call_service(&$app, test::TestRequest::post().uri("/api/v1/auth/mock/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"identity": $identity})).to_request()).await;
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response.response().cookies().find(|cookie| cookie.name() == auth_service::SESSION_COOKIE).expect("session cookie").into_owned();
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(response.headers().get("cache-control").expect("cache control"), "no-store");
        let body: Value = test::read_body_json(response).await;
        (cookie, body["data"]["csrf_token"].as_str().expect("csrf token").to_string())
    }};
}

#[actix_web::test]
async fn anonymous_invalid_json_and_cross_origin_requests_are_rejected() {
    let (state, _database) = setup_auth().await;
    let app = create_test_app!(state);
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/teachers/me")
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let request_id = response
        .headers()
        .get("x-request-id")
        .expect("request id")
        .to_str()
        .expect("id text")
        .to_string();
    let body: Value = test::read_body_json(response).await;
    assert_eq!(body["request_id"], request_id);
    assert_eq!(body["code"], "unauthorized");
    for origin in [None, Some("https://other.example")] {
        let mut request = test::TestRequest::post()
            .uri("/api/v1/auth/mock/login")
            .set_json(json!({"identity": "teacher"}));
        if let Some(origin) = origin {
            request = request.insert_header(("Origin", origin));
        }
        assert_eq!(
            test::call_service(&app, request.to_request())
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/mock/login")
            .insert_header(("Content-Type", "application/json"))
            .set_payload("invalid")
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: Value = test::read_body_json(response).await;
    assert_eq!(body["code"], "invalid_request");
}

#[actix_web::test]
async fn profile_permissions_csrf_logout_and_session_revocation_work_together() {
    let (state, _database) = setup_auth().await;
    let app = create_test_app!(state.clone());
    let (cookie, csrf) = login!(app, "teacher");
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/teachers/me")
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = test::read_body_json(response).await;
    assert_eq!(body["data"]["employee_no"], "DEMO-91001");
    assert_eq!(body["data"]["department_name"], "数学教研组");
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/auth/permissions")
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = test::read_body_json(response).await;
    assert!(
        body["data"]
            .as_array()
            .expect("grants")
            .iter()
            .all(|grant| grant["scope"] == "SELF")
    );
    for id in [91002, 999999] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/api/v1/teachers/{id}"))
                .cookie(cookie.clone())
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    for (origin, token) in [
        ("http://127.0.0.1:3000", "wrong"),
        ("https://other.example", csrf.as_str()),
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/auth/logout")
                .cookie(cookie.clone())
                .insert_header(("Origin", origin))
                .insert_header(("X-CSRF-Token", token))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/logout")
            .cookie(cookie.clone())
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .insert_header(("X-CSRF-Token", csrf))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
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
    let audits = aster_forge_db::audit_log::Entity::find()
        .all(state.db_handles.writer())
        .await
        .expect("read audits");
    assert!(audits.iter().any(|entry| entry.action == "auth.login"));
    assert!(audits.iter().any(|entry| entry.action == "auth.logout"));
    assert!(audits.iter().any(|entry| entry.action == "teacher.read"));
}

#[actix_web::test]
async fn department_grants_do_not_allow_other_departments_but_admin_can_read_all() {
    let (state, _database) = setup_auth().await;
    let app = create_test_app!(state);
    let (leader, _) = login!(app, "leader");
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/teachers/91001")
                .cookie(leader.clone())
                .to_request()
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/teachers/91003")
                .cookie(leader)
                .to_request()
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let (admin, _) = login!(app, "admin");
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/teachers/91001")
                .cookie(admin)
                .to_request()
        )
        .await
        .status(),
        StatusCode::OK
    );
}

#[actix_web::test]
async fn disabled_accounts_inactive_teachers_expired_sessions_and_rotation_revoke_access() {
    use teacher_oa_service::repositories::entities::{teacher, user};
    let (state, _database) = setup_auth().await;
    let app = create_test_app!(state.clone());
    let (old_cookie, _) = login!(app, "teacher");
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/mock/login")
            .cookie(old_cookie.clone())
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"identity":"teacher"}))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == auth_service::SESSION_COOKIE)
        .expect("cookie")
        .into_owned();
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/me")
                .cookie(old_cookie)
                .to_request()
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    user::Entity::update_many()
        .col_expr(user::Column::Status, Expr::value("disabled"))
        .filter(user::Column::Id.eq(91001))
        .exec(state.db_handles.writer())
        .await
        .expect("disable user");
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
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/auth/mock/login")
                .insert_header(("Origin", "http://127.0.0.1:3000"))
                .set_json(json!({"identity":"teacher"}))
                .to_request()
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let (leader, _) = login!(app, "leader");
    teacher::Entity::update_many()
        .col_expr(teacher::Column::EmploymentStatus, Expr::value("inactive"))
        .filter(teacher::Column::Id.eq(91002))
        .exec(state.db_handles.writer())
        .await
        .expect("deactivate teacher");
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/me")
                .cookie(leader)
                .to_request()
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let token = auth_service::random_token();
    let expired = auth_service::Session {
        user_id: 91004,
        csrf_token: auth_service::random_token(),
        expires_at: 0,
    };
    state
        .auth_store
        .put(
            &format!("teacher_oa:session:{token}"),
            serde_json::to_vec(&expired).expect("serialize"),
            60,
        )
        .await
        .expect("store expired session");
    assert_eq!(
        test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/me")
                .cookie(Cookie::new(auth_service::SESSION_COOKIE, token))
                .to_request()
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
}

struct TestWeCom;
#[async_trait::async_trait]
impl WeComAdapter for TestWeCom {
    fn enabled(&self) -> bool {
        true
    }
    fn authorization_url(&self, state: &str, _mode: LoginMode) -> Result<String, ApiError> {
        Ok(format!(
            "https://open.work.weixin.qq.com/wwopen/sso/qrConnect?state={state}"
        ))
    }
    async fn exchange(&self, code: &str) -> Result<WeComIdentity, ApiError> {
        Ok(WeComIdentity {
            corp_id: if code == "foreign" {
                "foreign"
            } else {
                "test-corp"
            }
            .into(),
            member_id: if code == "unknown" {
                "unknown"
            } else {
                "bound-member"
            }
            .into(),
        })
    }
}

#[actix_web::test]
async fn wecom_callback_requires_browser_binding_one_time_state_and_prebound_member() {
    use sea_orm::{ActiveModelTrait, Set};
    use teacher_oa_service::repositories::entities::identity;
    let (mut state, _database) = setup_auth().await;
    Arc::make_mut(&mut state.config).auth.wecom_corp_id = "test-corp".into();
    state.wecom = Arc::new(TestWeCom);
    identity::ActiveModel {
        user_id: Set(91001),
        provider: Set("wecom".into()),
        corp_id: Set("test-corp".into()),
        provider_userid: Set("bound-member".into()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    }
    .insert(state.db_handles.writer())
    .await
    .expect("bind member");
    let app = create_test_app!(state.clone());
    for (code, valid_binding, expected) in [
        ("valid", true, "/dashboard"),
        ("valid", false, "/login?error=authorization_failed"),
        ("unknown", true, "/login?error=authorization_failed"),
        ("foreign", true, "/login?error=authorization_failed"),
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/auth/wecom/login")
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FOUND);
        let location = response
            .headers()
            .get("location")
            .expect("authorization URL")
            .to_str()
            .expect("URL text");
        let url = url::Url::parse(location).expect("authorization URL");
        let value = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .expect("state")
            .1
            .into_owned();
        let binding = response
            .response()
            .cookies()
            .find(|cookie| cookie.name() == auth_service::OAUTH_COOKIE)
            .expect("binding")
            .into_owned();
        let uri = format!("/api/v1/auth/wecom/callback?state={value}&code={code}");
        let request = test::TestRequest::get()
            .uri(&uri)
            .cookie(if valid_binding {
                binding.clone()
            } else {
                Cookie::new(auth_service::OAUTH_COOKIE, auth_service::random_token())
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get("location").expect("redirect"),
            expected
        );
        assert_eq!(
            response
                .response()
                .cookies()
                .any(|cookie| cookie.name() == auth_service::SESSION_COOKIE),
            expected == "/dashboard"
        );
        let replay = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&uri)
                .cookie(binding)
                .to_request(),
        )
        .await;
        assert_eq!(
            replay.headers().get("location").expect("replay redirect"),
            "/login?error=authorization_failed"
        );
    }
    let value = auth_service::random_token();
    let binding = auth_service::random_token();
    state
        .auth_store
        .put(
            &format!("teacher_oa:oauth:{value}"),
            binding.as_bytes().to_vec(),
            0,
        )
        .await
        .expect("expired state");
    assert!(
        auth_service::consume_state(&state, &value, &binding)
            .await
            .is_err()
    );
    assert!(
        auth_service::consume_state(&state, &auth_service::random_token(), &binding)
            .await
            .is_err()
    );
}

#[actix_web::test]
async fn mock_login_is_disabled_by_default() {
    let (state, _database) = common::setup().await;
    let app = create_test_app!(state);
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/mock/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"identity":"teacher"}))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
