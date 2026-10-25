#[macro_use]
mod common;
use actix_web::{http::StatusCode, test};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
use serde_json::{Value, json};
use teacher_oa_service::{repositories::entities::user, services::auth_service::SESSION_COOKIE};

#[actix_web::test]
async fn disabled_account_can_still_terminate_its_own_session() {
    let (mut state, _database) = common::setup().await;
    std::sync::Arc::make_mut(&mut state.config)
        .auth
        .mock_enabled = true;
    teacher_oa_service::services::development_seed::seed(state.db_handles.writer())
        .await
        .expect("seed");
    let app = create_test_app!(state.clone());
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/mock/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"identity":"teacher"}))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == SESSION_COOKIE)
        .expect("cookie")
        .into_owned();
    let body: Value = test::read_body_json(response).await;
    user::Entity::update_many()
        .col_expr(user::Column::Status, Expr::value("disabled"))
        .filter(user::Column::Id.eq(91001))
        .exec(state.db_handles.writer())
        .await
        .expect("disable");
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/logout")
            .cookie(cookie.clone())
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .insert_header((
                "X-CSRF-Token",
                body["data"]["csrf_token"].as_str().expect("csrf"),
            ))
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
}
