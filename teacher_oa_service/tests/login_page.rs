#[macro_use]
mod common;

use actix_web::{http::StatusCode, test};
use serde_json::{Value, json};
use teacher_oa_service::services::{auth_service, development_seed};

#[actix_web::test]
async fn administrators_save_images_publish_and_withdraw_announcements() {
    // All fixtures and mutations stay in the test helper's temporary SQLite database and memory store.
    let (mut state, _database) = common::setup().await;
    std::sync::Arc::make_mut(&mut state.config)
        .auth
        .mock_enabled = true;
    development_seed::seed(state.db_handles.writer())
        .await
        .unwrap();
    let app = create_test_app!(state.clone());
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/mock/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"identity": "admin"}))
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
    let session: Value = test::read_body_json(response).await;
    let csrf = session["data"]["csrf_token"].as_str().unwrap();

    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(4, 4)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let bytes = png.into_inner();
    let mut multipart = b"--login-image\r\nContent-Disposition: form-data; name=\"file\"; filename=\"logo.png\"\r\nContent-Type: image/png\r\n\r\n".to_vec();
    multipart.extend_from_slice(&bytes);
    multipart.extend_from_slice(b"\r\n--login-image--\r\n");
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/login-page/assets")
            .cookie(cookie.clone())
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .insert_header(("X-CSRF-Token", csrf))
            .insert_header(("Content-Type", "multipart/form-data; boundary=login-image"))
            .set_payload(multipart)
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let uploaded: Value = test::read_body_json(response).await;
    let url = uploaded["data"]["url"].as_str().unwrap();
    let response = test::call_service(&app, test::TestRequest::get().uri(url).to_request()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("content-type").unwrap(), "image/png");
    assert_eq!(test::read_body(response).await.as_ref(), bytes.as_slice());

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/login-page")
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    let initial: Value = test::read_body_json(response).await;
    let mut settings = initial["data"].clone();
    settings["title"] = json!("教师工作平台");
    settings["logo_url"] = json!(url);
    settings["background_url"] = json!(url);
    settings["footer"] = json!("学校教务处");
    settings["announcements"] = json!([
        {"id":"notice-1", "title":"公开通知", "content":"公告正文\n第二行", "published":true, "published_at":null},
        {"id":"notice-2", "title":"未发布草稿", "content":"暂存内容", "published":false, "published_at":null}
    ]);
    let response = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/login-page")
            .cookie(cookie.clone())
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .insert_header(("X-CSRF-Token", csrf))
            .set_json(&settings)
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let saved: Value = test::read_body_json(response).await;
    assert_eq!(saved["data"]["announcements"].as_array().unwrap().len(), 2);
    assert!(saved["data"]["announcements"][0]["published_at"].is_string());
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/login-page")
            .to_request(),
    )
    .await;
    let public: Value = test::read_body_json(response).await;
    assert_eq!(public["data"]["title"], "教师工作平台");
    assert_eq!(public["data"]["logo_url"], url);
    assert_eq!(public["data"]["announcements"].as_array().unwrap().len(), 1);
    assert_eq!(public["data"]["announcements"][0]["title"], "公开通知");
    // A second editor cannot overwrite a newer configuration with its old version.
    let response = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/login-page")
            .cookie(cookie.clone())
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .insert_header(("X-CSRF-Token", csrf))
            .set_json(&settings)
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let mut withdraw = saved["data"].clone();
    withdraw["announcements"][0]["published"] = json!(false);
    let response = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/login-page")
            .cookie(cookie)
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .insert_header(("X-CSRF-Token", csrf))
            .set_json(&withdraw)
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/login-page")
            .to_request(),
    )
    .await;
    let public: Value = test::read_body_json(response).await;
    assert!(
        public["data"]["announcements"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
