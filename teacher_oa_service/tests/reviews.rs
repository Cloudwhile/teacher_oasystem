#[macro_use]
mod common;
use actix_web::{http::StatusCode, test};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde_json::json;
use teacher_oa_service::{
    api::error::ApiError,
    domain::{
        achievements::{AchievementInput, ReviewRequest, UpdateAchievement},
        identity::{DataScope, PermissionGrant},
    },
    repositories::{
        achievement_entities::{review_event, revision},
        achievements as repo, identity,
    },
    services::{
        achievement_service as achievements, development_seed, file_service,
        review_service::{ReviewAction, review},
    },
};

fn request(version: i32, opinion: &str) -> ReviewRequest {
    ReviewRequest {
        version,
        opinion: opinion.into(),
    }
}

fn png() -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(2, 2)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

#[actix_web::test]
async fn review_return_resubmit_archive_preserves_snapshots_and_permissions() {
    let (mut state, _database) = common::setup().await;
    std::sync::Arc::make_mut(&mut state.config)
        .auth
        .mock_enabled = true;
    let db = state.db_handles.writer();
    development_seed::seed(db).await.expect("seed");
    let owner = identity::current_user(db, 91001).await.unwrap();
    let reviewer = identity::current_user(db, 91003).await.unwrap();
    let file = file_service::upload(&state, &owner, "材料.png".into(), "image/png".into(), png())
        .await
        .unwrap();
    let replacement = file_service::upload(
        &state,
        &owner,
        "替换材料.png".into(),
        "image/png".into(),
        png(),
    )
    .await
    .unwrap();
    let unused = file_service::upload(
        &state,
        &owner,
        "未关联.png".into(),
        "image/png".into(),
        png(),
    )
    .await
    .unwrap();
    let input: AchievementInput = serde_json::from_value(json!({"type_code":"open_class","schema_version":1,"title":"审核闭环","file_ids":[file.id]})).unwrap();
    let draft = achievements::create(db, &owner, input.clone())
        .await
        .unwrap();
    let id = draft.record.id;
    assert!(matches!(
        review(db, &reviewer, id, request(1, ""), ReviewAction::Approve).await,
        Err(ApiError::Conflict(_))
    ));
    achievements::submit(db, &owner, id, 1).await.unwrap();
    assert!(matches!(
        review(db, &owner, id, request(2, ""), ReviewAction::Approve).await,
        Err(ApiError::Forbidden)
    ));
    let mut self_reviewer = owner.clone();
    self_reviewer.grants.push(PermissionGrant {
        permission: "reviews:write".into(),
        scope: DataScope::All,
    });
    assert!(matches!(
        review(
            db,
            &self_reviewer,
            id,
            request(2, ""),
            ReviewAction::Approve
        )
        .await,
        Err(ApiError::Forbidden)
    ));
    let mut department_reviewer = reviewer.clone();
    for grant in &mut department_reviewer.grants {
        if grant.permission == "reviews:write" {
            grant.scope = DataScope::Department;
        }
    }
    assert!(repo::accessible(db, &department_reviewer, id).await.is_ok());
    assert!(matches!(
        review(
            db,
            &department_reviewer,
            id,
            request(2, ""),
            ReviewAction::Approve
        )
        .await,
        Err(ApiError::Forbidden)
    ));
    assert!(matches!(
        review(db, &reviewer, id, request(2, " \n "), ReviewAction::Return).await,
        Err(ApiError::Invalid(_))
    ));
    assert!(matches!(
        review(
            db,
            &reviewer,
            id,
            request(2, &"长".repeat(2001)),
            ReviewAction::Approve
        )
        .await,
        Err(ApiError::Invalid(_))
    ));
    assert!(matches!(
        review(db, &reviewer, id, request(2, ""), ReviewAction::Archive).await,
        Err(ApiError::Conflict(_))
    ));
    let returned = review(
        db,
        &reviewer,
        id,
        request(2, " 请补充材料说明 "),
        ReviewAction::Return,
    )
    .await
    .unwrap();
    assert_eq!(returned.record.status, "returned");
    assert_eq!(returned.events.last().unwrap().opinion, "请补充材料说明");
    assert_eq!(returned.events.last().unwrap().reviewer_id, reviewer.id);
    assert!(matches!(
        review(db, &reviewer, id, request(2, ""), ReviewAction::Approve).await,
        Err(ApiError::Conflict(_))
    ));
    let corrected = achievements::update(
        db,
        &owner,
        id,
        UpdateAchievement {
            version: 3,
            data: AchievementInput {
                file_ids: vec![replacement.id],
                ..input.clone()
            },
        },
    )
    .await
    .unwrap();
    assert_eq!(corrected.record.version, 4);
    assert_eq!(
        repo::file_association(db, &owner, file.id).await.unwrap(),
        teacher_oa_service::domain::achievements::FileAssociation::Historical
    );
    assert_eq!(
        repo::file_association(db, &owner, replacement.id)
            .await
            .unwrap(),
        teacher_oa_service::domain::achievements::FileAssociation::Linked
    );
    assert_eq!(
        repo::file_association(db, &owner, unused.id).await.unwrap(),
        teacher_oa_service::domain::achievements::FileAssociation::Unlinked
    );
    assert!(repo::file_access(db, &owner, file.id).await.is_ok());
    achievements::submit(db, &owner, id, 4).await.unwrap();
    let approved = review(
        db,
        &reviewer,
        id,
        request(5, "核验通过"),
        ReviewAction::Approve,
    )
    .await
    .unwrap();
    assert!(approved.can_archive);
    assert!(!approved.can_review);
    assert!(matches!(
        achievements::update(
            db,
            &owner,
            id,
            UpdateAchievement {
                version: 6,
                data: input
            }
        )
        .await,
        Err(ApiError::Conflict(_))
    ));
    let archived = review(db, &reviewer, id, request(6, ""), ReviewAction::Archive)
        .await
        .unwrap();
    assert_eq!(archived.record.status, "archived");
    assert_eq!(archived.record.version, 7);
    assert!(!archived.can_archive);
    assert_eq!(
        revision::Entity::find()
            .filter(revision::Column::AchievementId.eq(id))
            .count(db)
            .await
            .unwrap(),
        7
    );
    assert_eq!(
        review_event::Entity::find()
            .filter(review_event::Column::AchievementId.eq(id))
            .count(db)
            .await
            .unwrap(),
        5
    );

    let app = create_test_app!(state.clone());
    let login = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/mock/login")
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .set_json(json!({"identity":"reviewer"}))
            .to_request(),
    )
    .await;
    let cookie = login
        .response()
        .cookies()
        .find(|c| c.name() == "teacher_oa_session")
        .unwrap()
        .into_owned();
    let body: serde_json::Value = test::read_body_json(login).await;
    let csrf = body["data"]["csrf_token"].as_str().unwrap();
    for path in ["approve", "return", "archive"] {
        let response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri(&format!("/api/v1/achievements/{id}/{path}"))
                .cookie(cookie.clone())
                .set_json(request(7, "意见"))
                .to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    let draft = achievements::create(db, &owner, serde_json::from_value(json!({"type_code":"open_class","schema_version":1,"title":"接口审核","file_ids":[file.id]})).unwrap()).await.unwrap();
    achievements::submit(db, &owner, draft.record.id, 1)
        .await
        .unwrap();
    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/reviews/pending")
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let page: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(page["data"]["total"], 1);
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!("/api/v1/achievements/{}/approve", draft.record.id))
            .cookie(cookie)
            .insert_header(("Origin", "http://127.0.0.1:3000"))
            .insert_header(("X-CSRF-Token", csrf))
            .set_json(request(2, "通过"))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
}
