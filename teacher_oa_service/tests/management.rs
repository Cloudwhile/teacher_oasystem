#[macro_use]
mod common;

use actix_web::{App, http::StatusCode, test, web};
use management_protocol::{ConfigSnapshot, SaveConfig};
use std::sync::Arc;
use teacher_oa_service::services::management_service::Management;
use tokio_util::sync::CancellationToken;

const TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CONFIG: &str = r#"
[server]
host = "127.0.0.1"
port = 3000
[database]
url = { base_url = "postgres://example.invalid/service", username = "private-user", password = "private-password" }
pool_size = 10
[logging]
level = "info"
format = "text"
"#;

#[actix_web::test]
async fn local_capability_is_required_and_configuration_preserves_secrets() {
    let (mut state, _db) = common::setup().await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("service.toml");
    std::fs::write(&path, CONFIG).unwrap();
    state.management = Some(Arc::new(
        Management::new(TOKEN.into(), path.clone()).unwrap(),
    ));
    let shutdown = CancellationToken::new();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::Data::new(shutdown.clone()))
            .configure(teacher_oa_service::api::configure),
    )
    .await;

    for (peer, token, origin, expected) in [
        ("127.0.0.1:9000", None, None, StatusCode::UNAUTHORIZED),
        (
            "127.0.0.1:9000",
            Some("wrong"),
            None,
            StatusCode::UNAUTHORIZED,
        ),
        ("192.0.2.1:9000", Some(TOKEN), None, StatusCode::FORBIDDEN),
        (
            "127.0.0.1:9000",
            Some(TOKEN),
            Some("https://evil.invalid"),
            StatusCode::FORBIDDEN,
        ),
    ] {
        let mut request = test::TestRequest::get()
            .uri("/api/v1/management/config")
            .peer_addr(peer.parse().unwrap());
        if let Some(token) = token {
            request = request.insert_header(("Authorization", format!("Bearer {token}")));
        }
        if let Some(origin) = origin {
            request = request.insert_header(("Origin", origin));
        }
        let response = test::call_service(&app, request.to_request()).await;
        assert_eq!(response.status(), expected);
    }
    let request = || {
        test::TestRequest::get()
            .peer_addr("127.0.0.1:9000".parse().unwrap())
            .insert_header(("Authorization", format!("Bearer {TOKEN}")))
    };
    let response = test::call_service(
        &app,
        request().uri("/api/v1/management/status").to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let status: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(status["data"]["pid"], std::process::id());
    assert!(
        status["data"]["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["healthy"] == true)
    );

    let response = test::call_service(
        &app,
        request().uri("/api/v1/management/config").to_request(),
    )
    .await;
    let body = test::read_body(response).await;
    let text = std::str::from_utf8(&body).unwrap();
    assert!(
        !text.contains("private-password")
            && !text.contains("private-user")
            && !text.contains("example.invalid")
    );
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let snapshot: ConfigSnapshot = serde_json::from_value(json["data"].clone()).unwrap();
    let mut save = SaveConfig {
        config: snapshot.saved,
        revision: snapshot.revision,
    };
    save.config.log_level = "warn".into();
    let put = || {
        test::TestRequest::put()
            .uri("/api/v1/management/config")
            .peer_addr("127.0.0.1:9000".parse().unwrap())
            .insert_header(("Authorization", format!("Bearer {TOKEN}")))
    };
    let response = test::call_service(&app, put().set_json(&save).to_request()).await;
    assert_eq!(response.status(), StatusCode::OK);
    let result: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(result["data"]["restart_required"], true);
    let stored = std::fs::read_to_string(&path).unwrap();
    assert!(stored.contains("private-password") && stored.contains("private-user"));
    assert_eq!(
        toml::from_str::<toml::Value>(&stored).unwrap()["logging"]["level"].as_str(),
        Some("warn")
    );
    let response = test::call_service(&app, put().set_json(&save).to_request()).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    save.config.port = 0;
    let response = test::call_service(&app, put().set_json(&save).to_request()).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), stored);

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/management/shutdown")
            .peer_addr("127.0.0.1:9000".parse().unwrap())
            .insert_header(("Authorization", format!("Bearer {TOKEN}")))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    tokio::time::timeout(std::time::Duration::from_secs(2), shutdown.cancelled())
        .await
        .unwrap();
}

#[actix_web::test]
async fn ordinary_startup_does_not_enable_desktop_management() {
    let (state, _db) = common::setup().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::Data::new(CancellationToken::new()))
            .configure(teacher_oa_service::api::configure),
    )
    .await;
    for path in ["status", "config"] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/api/v1/management/{path}"))
                .peer_addr("127.0.0.1:9000".parse().unwrap())
                .insert_header(("Authorization", format!("Bearer {TOKEN}")))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

#[actix_web::test]
async fn listener_changes_update_only_matching_direct_loopback_origins() {
    for (host, origin, next_host, expected) in [
        (
            "127.0.0.1",
            "http://127.0.0.1:3000",
            "127.0.0.1",
            "http://127.0.0.1:3001",
        ),
        (
            "0.0.0.0",
            "http://127.0.0.1:3000",
            "::",
            "http://[::1]:3001",
        ),
        (
            "::1",
            "http://[::1]:3000",
            "127.0.0.1",
            "http://127.0.0.1:3001",
        ),
        (
            "127.0.0.1",
            "https://oa.example.edu.cn",
            "127.0.0.1",
            "https://oa.example.edu.cn",
        ),
        (
            "127.0.0.1",
            "http://127.0.0.1:9000",
            "127.0.0.1",
            "http://127.0.0.1:9000",
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("service.toml");
        let content = format!(
            "{}\n[auth]\npublic_origin = {origin:?}\n",
            CONFIG.replace("host = \"127.0.0.1\"", &format!("host = {host:?}"))
        );
        std::fs::write(&path, &content).unwrap();
        let active = toml::from_str(&content).unwrap();
        let management = Management::new(TOKEN.into(), path.clone()).unwrap();
        let snapshot = management.snapshot(&active).unwrap();
        let mut config = snapshot.saved;
        config.host = next_host.into();
        config.port = 3001;

        management
            .save(
                &active,
                SaveConfig {
                    config,
                    revision: snapshot.revision,
                },
            )
            .unwrap();

        let saved: teacher_oa_service::config::AppConfig =
            toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            saved.auth.public_origin, expected,
            "listener {host}, origin {origin}"
        );
        let request = test::TestRequest::post()
            .insert_header(("Origin", expected))
            .to_http_request();
        assert!(
            teacher_oa_service::services::auth_service::ensure_origin(&request, &saved.auth)
                .is_ok()
        );
    }
}

#[actix_web::test]
async fn invalid_management_credentials_fail_closed() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    std::fs::write(&path, CONFIG).unwrap();
    assert!(Management::new("short".into(), path).is_err());
}

#[actix_web::test]
async fn management_bearer_challenges_case_insensitive_scheme_and_404_when_disabled() {
    let (mut state, _db) = common::setup().await;
    let shutdown = CancellationToken::new();

    let disabled_app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(shutdown.clone()))
            .configure(teacher_oa_service::api::configure),
    )
    .await;
    let disabled = test::call_service(
        &disabled_app,
        test::TestRequest::post()
            .uri("/api/v1/management/status")
            .peer_addr("127.0.0.1:9000".parse().unwrap())
            .to_request(),
    )
    .await;
    assert_eq!(disabled.status(), StatusCode::NOT_FOUND);

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("service.toml");
    std::fs::write(&path, CONFIG).unwrap();
    state.management = Some(Arc::new(Management::new(TOKEN.into(), path).unwrap()));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .app_data(web::Data::new(shutdown))
            .configure(teacher_oa_service::api::configure),
    )
    .await;

    let missing = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/management/status")
            .peer_addr("127.0.0.1:9000".parse().unwrap())
            .to_request(),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        missing
            .headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok()),
        Some("Bearer realm=\"TeacherOA Management\"")
    );

    let bad = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/management/status")
            .peer_addr("127.0.0.1:9000".parse().unwrap())
            .insert_header(("Authorization", "Bearer invalid-token"))
            .to_request(),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::UNAUTHORIZED);
    let challenge = bad
        .headers()
        .get("www-authenticate")
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(challenge.contains("error=\"invalid_token\""), "{challenge}");

    let lowercase = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/management/status")
            .peer_addr("127.0.0.1:9000".parse().unwrap())
            .insert_header(("Authorization", format!("bearer {TOKEN}")))
            .to_request(),
    )
    .await;
    assert_eq!(lowercase.status(), StatusCode::OK);

    let method_not_allowed = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri("/api/v1/management/status")
            .peer_addr("127.0.0.1:9000".parse().unwrap())
            .to_request(),
    )
    .await;
    assert_eq!(method_not_allowed.status(), StatusCode::METHOD_NOT_ALLOWED);
    let allow = method_not_allowed
        .headers()
        .get("allow")
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(allow.contains("GET") && allow.contains("HEAD"), "{allow}");
}
