#[macro_use]
mod common;

use actix_web::{http::StatusCode, test};
use serde_json::{Value, json};
use teacher_oa_service::{
    domain::achievements::AchievementInput,
    repositories::identity,
    services::{
        account_service as accounts, achievement_service, auth_service, development_seed,
        file_service,
    },
};

#[actix_web::test]
async fn admin_and_school_leader_can_view_all_teachers_dossiers_and_filter_achievements() {
    let (mut state, _database) = common::setup().await;
    std::sync::Arc::make_mut(&mut state.config)
        .auth
        .mock_enabled = true;
    let db = state.db_handles.writer();
    development_seed::seed(db).await.expect("seed identities");

    let leader_user_id = accounts::save(
        db,
        None,
        "",
        accounts::AccountInput {
            id: None,
            username: "principal".into(),
            name: "张校长".into(),
            department: "校长室".into(),
            role: "school_leader".into(),
            active: true,
            password: Some("Principal-pass-123".into()),
            wecom_userid: None,
            employee_no: Some("ADM-0001".into()),
            subject_code: Some("education_management".into()),
        },
    )
    .await
    .expect("create school_leader");

    // Change initial password so must_change_password is false for school_leader.
    accounts::change(
        db,
        leader_user_id,
        accounts::PasswordChange {
            current_password: "Principal-pass-123".into(),
            new_password: "Principal-pass-456".into(),
        },
    )
    .await
    .expect("activate school_leader password");

    // Seed one submitted achievement (with ready file) and one private draft for teacher 91001 (数学教研组).
    let teacher_user = identity::current_user(db, 91001).await.expect("teacher");
    let mut png_buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(2, 2)
        .write_to(&mut png_buf, image::ImageFormat::Png)
        .expect("encode png");
    let material = file_service::upload(
        &state,
        &teacher_user,
        "公开课证明.png".into(),
        "image/png".into(),
        png_buf.into_inner(),
    )
    .await
    .expect("upload material");

    let submitted_input: AchievementInput = serde_json::from_value(json!({
        "type_code": "open_class",
        "schema_version": 1,
        "title": "全校示范公开课",
        "subject_code": "math",
        "attributes": {},
        "file_ids": [material.id]
    }))
    .expect("submitted input");
    let created = achievement_service::create(db, &teacher_user, submitted_input)
        .await
        .expect("create achievement");
    achievement_service::submit(db, &teacher_user, created.record.id, created.record.version)
        .await
        .expect("submit achievement");

    let draft_input: AchievementInput = serde_json::from_value(json!({
        "type_code": "open_class",
        "schema_version": 1,
        "title": "未提交私人草稿",
        "subject_code": "math",
        "attributes": {},
        "file_ids": []
    }))
    .expect("draft input");
    achievement_service::create(db, &teacher_user, draft_input)
        .await
        .expect("create private draft");

    let app = create_test_app!(state.clone());

    // 1. Ordinary teacher, group_leader, and reviewer cannot access school-wide teacher directory or dossier.
    for mock_identity in ["teacher", "leader", "reviewer"] {
        let login = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/auth/mock/login")
                .insert_header(("Origin", "http://127.0.0.1:3000"))
                .set_json(json!({"identity": mock_identity}))
                .to_request(),
        )
        .await;
        assert_eq!(login.status(), StatusCode::OK);
        let cookie = login
            .response()
            .cookies()
            .find(|c| c.name() == auth_service::SESSION_COOKIE)
            .expect("session cookie")
            .into_owned();

        let dir_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/teachers/directory")
                .cookie(cookie.clone())
                .to_request(),
        )
        .await;
        assert_eq!(
            dir_resp.status(),
            StatusCode::FORBIDDEN,
            "{mock_identity} must not access /teachers/directory"
        );

        let dossier_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/teachers/91001/dossier")
                .cookie(cookie)
                .to_request(),
        )
        .await;
        assert_eq!(
            dossier_resp.status(),
            StatusCode::FORBIDDEN,
            "{mock_identity} must not access /teachers/91001/dossier"
        );
    }

    // 2. Authenticate both admin (mock) and school_leader (password login) and verify full directory, dossier, file preview, and filters.
    let admin_login = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/mock/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"identity": "admin"}))
            .to_request(),
    )
    .await;
    let admin_cookie = admin_login
        .response()
        .cookies()
        .find(|c| c.name() == auth_service::SESSION_COOKIE)
        .expect("admin cookie")
        .into_owned();

    let leader_login = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"username": "principal", "password": "Principal-pass-456"}))
            .to_request(),
    )
    .await;
    assert_eq!(leader_login.status(), StatusCode::OK);
    let leader_cookie = leader_login
        .response()
        .cookies()
        .find(|c| c.name() == auth_service::SESSION_COOKIE)
        .expect("school_leader cookie")
        .into_owned();

    for (label, cookie) in [("admin", admin_cookie), ("school_leader", leader_cookie)] {
        let dir_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/teachers/directory")
                .cookie(cookie.clone())
                .to_request(),
        )
        .await;
        assert_eq!(dir_resp.status(), StatusCode::OK, "{label} directory");
        let dir_body: Value = test::read_body_json(dir_resp).await;
        assert_eq!(dir_body["data"]["total"], 5, "{label} sees all 5 teachers");
        let math_teacher = dir_body["data"]["items"]
            .as_array()
            .expect("items")
            .iter()
            .find(|item| item["id"] == 91001)
            .expect("teacher 91001 in directory");
        assert_eq!(math_teacher["achievement_count"], 1);
        assert_eq!(math_teacher["material_count"], 1);

        let principal_item = dir_body["data"]["items"]
            .as_array()
            .expect("items")
            .iter()
            .find(|item| item["employee_no"] == "ADM-0001")
            .expect("principal in directory");
        assert_eq!(principal_item["subject_code"], "education_management");

        // Filter directory by department
        let filtered_dir = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/teachers/directory?department_id=91001")
                .cookie(cookie.clone())
                .to_request(),
        )
        .await;
        let filtered_body: Value = test::read_body_json(filtered_dir).await;
        assert_eq!(filtered_body["data"]["total"], 2);

        // Read teacher 91001 dossier (draft excluded, submitted achievement + material included)
        let dossier_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/teachers/91001/dossier")
                .cookie(cookie.clone())
                .to_request(),
        )
        .await;
        assert_eq!(dossier_resp.status(), StatusCode::OK, "{label} dossier");
        let dossier: Value = test::read_body_json(dossier_resp).await;
        assert_eq!(dossier["data"]["profile"]["name"], "测试教师");
        assert_eq!(dossier["data"]["achievement_total"], 1);
        assert_eq!(
            dossier["data"]["achievements"][0]["title"],
            "全校示范公开课"
        );
        assert_eq!(dossier["data"]["files"][0]["id"], material.id);

        // Preview teacher 91001's evidence material directly
        let preview_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/api/v1/files/{}/preview", material.id))
                .cookie(cookie.clone())
                .to_request(),
        )
        .await;
        assert_eq!(
            preview_resp.status(),
            StatusCode::OK,
            "{label} material preview"
        );

        // Filter achievements by department_id and teacher_id
        let ach_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri("/api/v1/achievements?scope=all&department_id=91001&teacher_id=91001")
                .cookie(cookie.clone())
                .to_request(),
        )
        .await;
        assert_eq!(ach_resp.status(), StatusCode::OK);
        let ach_body: Value = test::read_body_json(ach_resp).await;
        assert_eq!(ach_body["data"]["total"], 1);
        assert_eq!(ach_body["data"]["items"][0]["title"], "全校示范公开课");
    }
}
