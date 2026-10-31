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
async fn invalid_management_credentials_fail_closed() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    std::fs::write(&path, CONFIG).unwrap();
    assert!(Management::new("short".into(), path).is_err());
}
