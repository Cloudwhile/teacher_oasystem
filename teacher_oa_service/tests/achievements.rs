#[macro_use]
mod common;
use actix_web::{http::StatusCode, test};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, sea_query::Expr};
use serde_json::{Value, json};
use teacher_oa_service::{
    api::error::ApiError,
    domain::{
        achievement_validation,
        achievements::{AchievementInput, AchievementSchema},
    },
    repositories::achievement_entities::{achievement, achievement_type, schema_version},
    services::auth_service::SESSION_COOKIE,
};

async fn setup() -> (
    teacher_oa_service::runtime::AppState,
    aster_forge_test::temp::SqliteTestDatabase,
) {
    let (mut state, db) = common::setup().await;
    std::sync::Arc::make_mut(&mut state.config)
        .auth
        .mock_enabled = true;
    teacher_oa_service::services::development_seed::seed(state.db_handles.writer())
        .await
        .expect("seed");
    (state, db)
}
fn png() -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(2, 2)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .expect("test PNG");
    bytes.into_inner()
}
fn multipart(filename: &str, mime: &str, bytes: &[u8]) -> Vec<u8> {
    let mut body=format!("--proof\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {mime}\r\n\r\n").into_bytes();
    body.extend(bytes);
    body.extend(b"\r\n--proof--\r\n");
    body
}
macro_rules! login{($app:expr,$identity:expr)=>{{
    let response=test::call_service(&$app,test::TestRequest::post().uri("/api/v1/auth/mock/login").insert_header(("Origin","http://127.0.0.1:3000")).set_json(json!({"identity":$identity})).to_request()).await;
    assert_eq!(response.status(),StatusCode::OK);
    let cookie=response.response().cookies().find(|c|c.name()==SESSION_COOKIE).expect("cookie").into_owned();
    let body:Value=test::read_body_json(response).await;(cookie,body["data"]["csrf_token"].as_str().expect("csrf").to_string())
}};}
macro_rules! mutation {
    ($app:expr,$method:ident,$uri:expr,$cookie:expr,$csrf:expr,$body:expr) => {{
        test::call_service(
            &$app,
            test::TestRequest::$method()
                .uri($uri)
                .cookie($cookie.clone())
                .insert_header(("Origin", "http://127.0.0.1:3000"))
                .insert_header(("X-CSRF-Token", $csrf.as_str()))
                .set_json($body)
                .to_request(),
        )
        .await
    }};
}
macro_rules! upload {
    ($app:expr,$cookie:expr,$csrf:expr) => {{
        let response = test::call_service(
            &$app,
            test::TestRequest::post()
                .uri("/api/v1/files/upload")
                .cookie($cookie.clone())
                .insert_header(("Origin", "http://127.0.0.1:3000"))
                .insert_header(("X-CSRF-Token", $csrf.as_str()))
                .insert_header(("Content-Type", "multipart/form-data; boundary=proof"))
                .set_payload(multipart("证明.png", "image/png", &png()))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = test::read_body_json(response).await;
        assert!(body["data"].get("object_key").is_none());
        body["data"]["id"].as_i64().expect("file id")
    }};
}
fn draft(code: &str, files: Vec<i64>) -> Value {
    json!({"type_code":code,"schema_version":1,"title":format!("测试{code}"),"attributes":{},"file_ids":files})
}

#[actix_web::test]
async fn all_fifteen_schemas_enforce_submit_and_scalar_types() {
    let schemas = teacher_oa_service_migration::catalog::schemas();
    assert_eq!(schemas.len(), 15);
    for schema in schemas {
        let schema: AchievementSchema = serde_json::from_value(schema).expect("schema");
        let mut input: AchievementInput =
            serde_json::from_value(draft(&schema.code, vec![])).expect("input");
        assert!(achievement_validation::validate(&input, &schema, false).is_ok());
        assert!(matches!(
            achievement_validation::validate(&input, &schema, true),
            Err(ApiError::Validation(_))
        ));
        input.file_ids = vec![1];
        assert!(achievement_validation::validate(&input, &schema, true).is_ok());
        input.attributes.insert("unknown".into(), json!("value"));
        assert!(achievement_validation::validate(&input, &schema, false).is_err());
    }
    let schema: AchievementSchema = serde_json::from_value(
        teacher_oa_service_migration::catalog::schemas()
            .into_iter()
            .find(|s| s["code"] == "research_project")
            .expect("research schema"),
    )
    .expect("schema");
    for invalid in [
        json!(0.1),
        json!("0.001"),
        json!("1e3"),
        json!("-1"),
        json!("12."),
        json!("NaN"),
    ] {
        let mut input: AchievementInput =
            serde_json::from_value(draft("research_project", vec![])).expect("input");
        input.attributes.insert("project_funding".into(), invalid);
        assert!(achievement_validation::validate(&input, &schema, false).is_err());
    }
}

#[actix_web::test]
async fn fifteen_types_can_create_and_submit_and_two_core_types_round_trip() {
    let (state, _database) = setup().await;
    let app = create_test_app!(state.clone());
    let (cookie, csrf) = login!(app, "teacher");
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/achievement-types")
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    let types: Value = test::read_body_json(response).await;
    assert_eq!(types["data"].as_array().expect("types").len(), 15);
    let file_id = upload!(app, cookie, csrf);
    for kind in types["data"].as_array().expect("types") {
        let code = kind["code"].as_str().expect("code");
        let mut data = draft(code, vec![file_id]);
        if code == "open_class" {
            data["subject_code"] = json!("math");
            data["occurred_at"] = json!("2026-10-18");
            data["attributes"] = json!({"venue":"综合楼402","evaluation":"excellent"});
        }
        if code == "research_project" {
            data["attributes"] = json!({"project_funding":"123456789.01","project_number":"P-001","completed_at":"2027-06-01"});
            data["participants"] = json!([{"teacher_id":91001,"external_name":"测试教师","role_code":"host","sort_order":0},{"teacher_id":null,"external_name":"校外合作者","role_code":"participant","sort_order":1}]);
        }
        let response = mutation!(app, post, "/api/v1/achievements", cookie, csrf, &data);
        assert_eq!(response.status(), StatusCode::OK, "create {code}");
        let body: Value = test::read_body_json(response).await;
        let id = body["data"]["record"]["id"].as_i64().expect("id");
        let mut version = 1;
        if code == "open_class" {
            data["attributes"]["venue"] = json!("学术报告厅");
            let response = mutation!(
                app,
                patch,
                &format!("/api/v1/achievements/{id}"),
                cookie,
                csrf,
                json!({"version":1,"data":data})
            );
            assert_eq!(response.status(), StatusCode::OK);
            version = 2;
            let response = mutation!(
                app,
                patch,
                &format!("/api/v1/achievements/{id}"),
                cookie,
                csrf,
                json!({"version":1,"data":data})
            );
            assert_eq!(response.status(), StatusCode::CONFLICT);
        }
        let response = mutation!(
            app,
            post,
            &format!("/api/v1/achievements/{id}/submit"),
            cookie,
            csrf,
            json!({"version":version})
        );
        assert_eq!(response.status(), StatusCode::OK, "submit {code}");
        let result: Value = test::read_body_json(response).await;
        assert_eq!(result["data"]["record"]["status"], "pending");
        assert_eq!(result["data"]["can_edit"], false);
        if code == "research_project" {
            assert_eq!(
                result["data"]["data"]["attributes"]["project_funding"],
                "123456789.01"
            );
            assert_eq!(
                result["data"]["data"]["participants"][1]["external_name"],
                "校外合作者"
            );
            assert_eq!(
                achievement::Entity::find_by_id(id)
                    .one(state.db_handles.writer())
                    .await
                    .expect("funding row")
                    .expect("row")
                    .funding_amount
                    .expect("decimal")
                    .to_string(),
                "123456789.01"
            );
        }
        let response = mutation!(
            app,
            post,
            &format!("/api/v1/achievements/{id}/submit"),
            cookie,
            csrf,
            json!({"version":version+1})
        );
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&format!("/api/v1/achievements/{id}/revisions"))
                .cookie(cookie.clone())
                .to_request(),
        )
        .await;
        let revisions: Value = test::read_body_json(response).await;
        assert_eq!(
            revisions["data"].as_array().expect("revisions").len(),
            usize::try_from(version + 1).expect("length")
        );
    }
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/achievements?page_size=5&status=pending")
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    let body: Value = test::read_body_json(response).await;
    assert_eq!(body["data"]["total"], 15);
    assert_eq!(body["data"]["items"].as_array().expect("items").len(), 5);
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/files/{file_id}/preview"))
            .cookie(cookie)
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(test::read_body(response).await.as_ref(), png());
}

#[actix_web::test]
async fn file_and_achievement_access_is_scoped_and_mutations_need_csrf() {
    use teacher_oa_service::repositories::entities::user_role;
    let (state, _database) = setup().await;
    user_role::Entity::delete_by_id((91003, "reviewer".to_string()))
        .exec(state.db_handles.writer())
        .await
        .expect("remove broad role");
    let app = create_test_app!(state.clone());
    let (owner, csrf) = login!(app, "teacher");
    let id = upload!(app, owner, csrf);
    let (other, other_csrf) = login!(app, "reviewer");
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/files/{id}/download"))
            .cookie(other.clone())
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = mutation!(
        app,
        post,
        "/api/v1/achievements",
        other,
        other_csrf,
        draft("open_class", vec![id])
    );
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/achievements")
            .cookie(owner.clone())
            .set_json(draft("open_class", vec![id]))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = mutation!(
        app,
        post,
        "/api/v1/achievements",
        owner,
        csrf,
        draft("open_class", vec![id])
    );
    let record: Value = test::read_body_json(response).await;
    let record_id = record["data"]["record"]["id"].as_i64().expect("record");
    // Drafts and their proofs stay private to the owner until submitted.
    let (leader, leader_csrf) = login!(app, "leader");
    for uri in [
        format!("/api/v1/achievements/{record_id}"),
        format!("/api/v1/files/{id}/download"),
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&uri)
                .cookie(leader.clone())
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{uri}");
    }
    let response = mutation!(
        app,
        post,
        &format!("/api/v1/achievements/{record_id}/submit"),
        owner,
        csrf,
        json!({"version":1})
    );
    assert_eq!(response.status(), StatusCode::OK);
    // Broader achievement access must not broaden an unrelated file grant.
    let mut restricted =
        teacher_oa_service::repositories::identity::current_user(state.db_handles.writer(), 91003)
            .await
            .expect("restricted reader");
    restricted
        .grants
        .push(teacher_oa_service::domain::identity::PermissionGrant {
            permission: "achievements:read".into(),
            scope: teacher_oa_service::domain::identity::DataScope::All,
        });
    assert!(
        teacher_oa_service::repositories::achievements::accessible(
            state.db_handles.writer(),
            &restricted,
            record_id,
        )
        .await
        .is_ok()
    );
    assert!(matches!(
        teacher_oa_service::services::file_service::read(&state, &restricted, id).await,
        Err(ApiError::Forbidden)
    ));
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/achievements/{record_id}"))
            .cookie(other.clone())
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/files/{id}/download"))
            .cookie(leader.clone())
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let response = mutation!(
        app,
        patch,
        &format!("/api/v1/achievements/{record_id}"),
        leader,
        leader_csrf,
        json!({"version":1,"data":draft("open_class",vec![id])})
    );
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let mut returner = restricted.clone();
    returner
        .grants
        .push(teacher_oa_service::domain::identity::PermissionGrant {
            permission: "reviews:write".into(),
            scope: teacher_oa_service::domain::identity::DataScope::All,
        });
    teacher_oa_service::services::review_service::review(
        state.db_handles.writer(),
        &returner,
        record_id,
        teacher_oa_service::domain::achievements::ReviewRequest {
            version: 2,
            opinion: "请更换材料".into(),
        },
        teacher_oa_service::services::review_service::ReviewAction::Return,
    )
    .await
    .expect("return for correction");
    let response = mutation!(
        app,
        patch,
        &format!("/api/v1/achievements/{record_id}"),
        owner,
        csrf,
        json!({"version":3,"data":draft("open_class",vec![])})
    );
    assert_eq!(response.status(), StatusCode::OK);
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/files/{id}/download"))
            .cookie(leader.clone())
            .to_request(),
    )
    .await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "historical proof remains accessible"
    );
    let mut joint = draft("research_project", vec![]);
    joint["participants"] = json!([{"teacher_id":91001,"external_name":"测试教师","role_code":"participant","sort_order":0},{"teacher_id":91002,"external_name":"测试组长","role_code":"host","sort_order":1}]);
    let response = mutation!(
        app,
        post,
        "/api/v1/achievements",
        leader,
        leader_csrf,
        joint
    );
    assert_eq!(response.status(), StatusCode::OK);
    let joint: Value = test::read_body_json(response).await;
    let joint_id = joint["data"]["record"]["id"].as_i64().expect("joint id");
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/achievements/{joint_id}"))
            .cookie(owner)
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[actix_web::test]
async fn validations_reject_incomplete_invalid_files_and_term_mismatch() {
    let (state, _database) = setup().await;
    let app = create_test_app!(state);
    let (cookie, csrf) = login!(app, "teacher");
    let response = mutation!(
        app,
        post,
        "/api/v1/achievements",
        cookie,
        csrf,
        draft("open_class", vec![])
    );
    let body: Value = test::read_body_json(response).await;
    let id = body["data"]["record"]["id"].as_i64().expect("id");
    let response = mutation!(
        app,
        post,
        &format!("/api/v1/achievements/{id}/submit"),
        cookie,
        csrf,
        json!({"version":1})
    );
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = test::read_body_json(response).await;
    assert!(body["error"]["diagnostic"]["fields"]["file_ids"].is_string());
    for (filename, mime, bytes) in [
        (
            "proof.png",
            "image/png",
            b"\x89PNG\r\n\x1a\ninvalid".to_vec(),
        ),
        ("proof.pdf", "application/pdf", png()),
        ("proof.html", "text/html", b"<script>bad</script>".to_vec()),
    ] {
        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/files/upload")
                .cookie(cookie.clone())
                .insert_header(("Origin", "http://127.0.0.1:3000"))
                .insert_header(("X-CSRF-Token", csrf.as_str()))
                .insert_header(("Content-Type", "multipart/form-data; boundary=proof"))
                .set_payload(multipart(filename, mime, &bytes))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    let file = upload!(app, cookie, csrf);
    let mut data = draft("open_class", vec![file]);
    data["academic_term_id"] = json!(1);
    data["occurred_at"] = json!("2025-01-01");
    let response = mutation!(app, post, "/api/v1/achievements", cookie, csrf, &data);
    let record: Value = test::read_body_json(response).await;
    let id = record["data"]["record"]["id"].as_i64().expect("id");
    assert_eq!(
        mutation!(
            app,
            post,
            &format!("/api/v1/achievements/{id}/submit"),
            cookie,
            csrf,
            json!({"version":1})
        )
        .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    data["term_mismatch_confirmed"] = json!(true);
    assert_eq!(
        mutation!(
            app,
            patch,
            &format!("/api/v1/achievements/{id}"),
            cookie,
            csrf,
            json!({"version":1,"data":data})
        )
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        mutation!(
            app,
            post,
            &format!("/api/v1/achievements/{id}/submit"),
            cookie,
            csrf,
            json!({"version":2})
        )
        .status(),
        StatusCode::OK
    );
    let mut invalid = draft("open_class", vec![]);
    invalid["attributes"] = json!({"evaluation":"first"});
    assert_eq!(
        mutation!(app, post, "/api/v1/achievements", cookie, csrf, invalid).status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[actix_web::test]
async fn normalized_participants_and_revision_titles_match_persisted_data() {
    let (state, _database) = setup().await;
    let app = create_test_app!(state);
    let (cookie, csrf) = login!(app, "teacher");
    let mut data = draft("research_project", vec![]);
    data["participants"] =
        json!([{"teacher_id":91001,"external_name":"","role_code":"host","sort_order":0}]);
    let response = mutation!(app, post, "/api/v1/achievements", cookie, csrf, &data);
    assert_eq!(response.status(), StatusCode::OK);
    let created: Value = test::read_body_json(response).await;
    let id = created["data"]["record"]["id"].as_i64().expect("id");
    assert_eq!(
        created["data"]["data"]["participants"][0]["external_name"],
        "测试教师"
    );
    data["title"] = json!("  修订标题  ");
    let response = mutation!(
        app,
        patch,
        &format!("/api/v1/achievements/{id}"),
        cookie,
        csrf,
        json!({"version":1,"data":data})
    );
    assert_eq!(response.status(), StatusCode::OK);
    let updated: Value = test::read_body_json(response).await;
    assert_eq!(updated["data"]["data"]["title"], "修订标题");
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/achievements/{id}/revisions"))
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    let revisions: Value = test::read_body_json(response).await;
    assert_eq!(
        revisions["data"][0]["snapshot"]["data"]["title"],
        "修订标题"
    );
    data["participants"] = json!([
        {"teacher_id":91001,"external_name":"伪造姓名一","role_code":"host","sort_order":0},
        {"teacher_id":91001,"external_name":"伪造姓名二","role_code":"participant","sort_order":1}
    ]);
    let response = mutation!(
        app,
        patch,
        &format!("/api/v1/achievements/{id}"),
        cookie,
        csrf,
        json!({"version":2,"data":data})
    );
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[actix_web::test]
async fn search_treats_wildcards_as_text_and_pagination_stays_in_range() {
    let (state, _database) = setup().await;
    let app = create_test_app!(state);
    let (cookie, csrf) = login!(app, "teacher");
    for title in ["完成100%_!研修", "完成100AB研修", "普通成果"] {
        let mut data = draft("open_class", vec![]);
        data["title"] = json!(title);
        assert_eq!(
            mutation!(app, post, "/api/v1/achievements", cookie, csrf, data).status(),
            StatusCode::OK
        );
    }
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("search", "%_!")
        .finish();
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/achievements?{query}&page=99"))
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    let page: Value = test::read_body_json(response).await;
    assert_eq!(page["data"]["total"], 1);
    assert_eq!(page["data"]["page"], 1);
    assert_eq!(page["data"]["items"][0]["title"], "完成100%_!研修");
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/files?page=99")
            .cookie(cookie)
            .to_request(),
    )
    .await;
    let page: Value = test::read_body_json(response).await;
    assert_eq!(page["data"]["total"], 0);
    assert_eq!(page["data"]["page"], 1);
}

#[actix_web::test]
async fn published_schema_changes_do_not_change_existing_records() {
    let (state, _database) = setup().await;
    let app = create_test_app!(state.clone());
    let (cookie, csrf) = login!(app, "teacher");
    let response = mutation!(
        app,
        post,
        "/api/v1/achievements",
        cookie,
        csrf,
        draft("open_class", vec![])
    );
    let record: Value = test::read_body_json(response).await;
    let id = record["data"]["record"]["id"].as_i64().expect("id");
    let mut document = schema_version::Entity::find_by_id(("open_class".to_string(), 1))
        .one(state.db_handles.writer())
        .await
        .expect("schema")
        .expect("schema")
        .schema_document;
    document["version"] = json!(2);
    for field in document["fields"].as_array_mut().expect("fields") {
        if field["key"] == "venue" {
            field["required"] = json!(true);
        }
    }
    schema_version::ActiveModel {
        type_code: Set("open_class".into()),
        version: Set(2),
        schema_document: Set(document),
        published_at: Set(chrono::Utc::now()),
    }
    .insert(state.db_handles.writer())
    .await
    .expect("publish v2");
    achievement_type::Entity::update_many()
        .col_expr(achievement_type::Column::CurrentVersion, Expr::value(2))
        .filter(achievement_type::Column::Code.eq("open_class"))
        .exec(state.db_handles.writer())
        .await
        .expect("set current");
    assert_eq!(
        mutation!(
            app,
            post,
            "/api/v1/achievements",
            cookie,
            csrf,
            draft("open_class", vec![])
        )
        .status(),
        StatusCode::CONFLICT
    );
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/achievements/{id}"))
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    let old: Value = test::read_body_json(response).await;
    assert_eq!(old["data"]["schema"]["version"], 1);
    assert_eq!(
        mutation!(
            app,
            patch,
            &format!("/api/v1/achievements/{id}"),
            cookie,
            csrf,
            json!({"version":1,"data":draft("open_class",vec![])})
        )
        .status(),
        StatusCode::OK
    );
}
