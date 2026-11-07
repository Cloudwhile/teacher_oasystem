#[macro_use]
mod common;

use actix_web::{http::StatusCode, test};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::{Value, json};
use teacher_oa_service::{
    repositories::entities::{teacher, user},
    services::{auth_service::SESSION_COOKIE, development_seed},
};

#[actix_web::test]
async fn teacher_options_include_all_active_teachers_in_each_authorized_scope() {
    let (mut state, _database) = common::setup().await;
    std::sync::Arc::make_mut(&mut state.config)
        .auth
        .mock_enabled = true;
    let db = state.db_handles.writer();
    development_seed::seed(db).await.expect("seed identities");
    // Insert equal names in reverse ID order to exercise the stable tie breaker.
    for id in (100_000..100_303).rev() {
        user::ActiveModel {
            id: Set(id),
            display_name: Set(format!("Catalog {id}")),
            status: Set("active".into()),
            created_at: Set(Utc::now()),
        }
        .insert(db)
        .await
        .expect("insert user");
        teacher::ActiveModel {
            id: Set(id),
            user_id: Set(id),
            employee_no: Set(format!("CATALOG-{id}")),
            name: Set("Catalog teacher".into()),
            department_id: Set(if id == 100_301 { 91002 } else { 91001 }),
            subject_code: Set("math".into()),
            employment_status: Set(if id == 100_302 { "inactive" } else { "active" }.into()),
            created_at: Set(Utc::now()),
        }
        .insert(db)
        .await
        .expect("insert teacher");
    }
    let app = create_test_app!(state);
    for (identity, mut expected) in [
        (
            "admin",
            (100_000..100_302)
                .chain([91001, 91002, 91003, 91004])
                .collect::<Vec<i64>>(),
        ),
        (
            "leader",
            (100_000..100_301)
                .chain([91001, 91002])
                .collect::<Vec<i64>>(),
        ),
        ("teacher", vec![91001]),
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/auth/mock/login")
                .insert_header(("Origin", "http://127.0.0.1:3000"))
                .set_json(json!({"identity": identity}))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK, "login {identity}");
        let cookie = response
            .response()
            .cookies()
            .find(|cookie| cookie.name() == SESSION_COOKIE)
            .expect("session cookie")
            .into_owned();
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/teachers")
                .cookie(cookie)
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK, "catalog {identity}");
        let body: Value = test::read_body_json(response).await;
        let options = body["data"].as_array().expect("teacher array contract");
        let mut actual = options
            .iter()
            .map(|option| option["id"].as_i64().expect("teacher ID"))
            .collect::<Vec<_>>();
        expected.sort_unstable();
        actual.sort_unstable();
        assert_eq!(
            actual, expected,
            "complete authorized options for {identity}"
        );
        let same_name_ids = options
            .iter()
            .filter(|option| option["name"] == "Catalog teacher")
            .map(|option| option["id"].as_i64().expect("teacher ID"))
            .collect::<Vec<_>>();
        let expected_same_name = expected
            .into_iter()
            .filter(|id| *id >= 100_000)
            .collect::<Vec<_>>();
        assert_eq!(same_name_ids, expected_same_name, "stable name ties");
    }
}
