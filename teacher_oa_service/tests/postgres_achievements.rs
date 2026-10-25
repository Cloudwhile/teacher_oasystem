mod support;
use sea_orm::{ColumnTrait, Database, EntityTrait, PaginatorTrait, QueryFilter};
use sea_orm_migration::MigratorTrait;
use serde_json::json;
use teacher_oa_service::{
    api::error::ApiError,
    config::AppConfig,
    domain::achievements::AchievementInput,
    repositories::{
        achievement_entities::{achievement, revision},
        identity,
    },
    services::{achievement_service, file_service},
};
use teacher_oa_service_migration::Migrator;

#[tokio::test]
#[ignore = "requires a disposable PostgreSQL *_test database; uses private local files only"]
async fn postgres_decimal_jsonb_private_files_and_concurrent_submit() {
    let url = support::database_url();
    let parsed = url::Url::parse(&url).expect("database URL");
    assert!(parsed.path().ends_with("_test"));
    assert!(matches!(parsed.scheme(), "postgres" | "postgresql"));
    let db = Database::connect(url.clone()).await.expect("connect");
    Migrator::fresh(&db).await.expect("fresh migration");
    let directory = aster_forge_test::temp::TestTempDir::new("p1-postgres-storage");
    let mut config = AppConfig::default();
    config.database.url = url.into();
    config.auth.environment = "test".into();
    config.auth.mock_enabled = true;
    config.auth.mock_seed = true;
    config.storage.root = directory.join("objects").to_string_lossy().into_owned();
    let state = teacher_oa_service::runtime::assembly::prepare_state(config)
        .await
        .expect("state");
    let user = identity::current_user(state.db_handles.writer(), 91001)
        .await
        .expect("user");
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(2, 2)
        .write_to(&mut buffer, image::ImageFormat::Png)
        .expect("PNG");
    let bytes = buffer.into_inner();
    let material = file_service::upload(
        &state,
        &user,
        "证明材料.png".into(),
        "image/png".into(),
        bytes.clone(),
    )
    .await
    .expect("private upload");
    let (_, download) = file_service::read(&state, &user, material.id)
        .await
        .expect("private read");
    assert_eq!(download.as_ref(), bytes);
    let input:AchievementInput=serde_json::from_value(json!({"type_code":"research_project","schema_version":1,"title":"PostgreSQL真实课题","attributes":{"project_funding":"12345678901234.56","project_number":"PG-P1"},"file_ids":[material.id]})).expect("input");
    let record = achievement_service::create(state.db_handles.writer(), &user, input)
        .await
        .expect("create");
    let row = achievement::Entity::find_by_id(record.record.id)
        .one(&db)
        .await
        .expect("row")
        .expect("exists");
    assert_eq!(
        row.funding_amount.expect("funding").to_string(),
        "12345678901234.56"
    );
    assert_eq!(row.attributes["project_funding"], "12345678901234.56");
    let (left, right) = tokio::join!(
        achievement_service::submit(state.db_handles.writer(), &user, row.id, 1),
        achievement_service::submit(state.db_handles.writer(), &user, row.id, 1)
    );
    assert_eq!(usize::from(left.is_ok()) + usize::from(right.is_ok()), 1);
    assert!(
        matches!(left, Err(ApiError::Conflict(_))) || matches!(right, Err(ApiError::Conflict(_)))
    );
    assert_eq!(
        revision::Entity::find()
            .filter(revision::Column::AchievementId.eq(row.id))
            .count(&db)
            .await
            .expect("revision count"),
        2
    );
    let row = achievement::Entity::find_by_id(row.id)
        .one(&db)
        .await
        .expect("query")
        .expect("record");
    assert_eq!(row.status, "pending");
    assert_eq!(row.version, 2);
    db.close().await.expect("close");
}
