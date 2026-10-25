//! Run explicitly against a disposable database whose name ends in `_test`.
mod support;
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
use sea_orm_migration::{MigratorTrait, SchemaManager};
use teacher_oa_service::{
    repositories::entities::{identity, teacher},
    services::development_seed,
};
use teacher_oa_service_migration::Migrator;

#[tokio::test]
#[ignore = "requires ASTER_TEST_DATABASE_URL pointing to a disposable PostgreSQL *_test database"]
async fn postgres_migrations_constraints_seed_and_rollback_are_real() {
    let endpoint = support::database_url();
    let parsed = url::Url::parse(&endpoint).expect("database URL");
    assert!(
        parsed.path().ends_with("_test"),
        "refuse to reset a database without the _test suffix"
    );
    assert!(matches!(parsed.scheme(), "postgres" | "postgresql"));
    let db = Database::connect(endpoint)
        .await
        .expect("connect PostgreSQL");
    Migrator::fresh(&db).await.expect("rebuild database");
    Migrator::up(&db, None).await.expect("second up is a no-op");
    development_seed::seed(&db).await.expect("seed");
    development_seed::seed(&db)
        .await
        .expect("seed is idempotent");
    assert_eq!(
        teacher::Entity::find()
            .all(&db)
            .await
            .expect("teachers")
            .len(),
        4
    );
    let duplicate = identity::ActiveModel {
        user_id: Set(91002),
        provider: Set("mock".into()),
        corp_id: Set("development".into()),
        provider_userid: Set("teacher".into()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    }
    .insert(&db)
    .await;
    assert!(duplicate.is_err(), "identity triple must be unique");
    let invalid_reference = identity::ActiveModel {
        user_id: Set(999999),
        provider: Set("wecom".into()),
        corp_id: Set("test".into()),
        provider_userid: Set("missing".into()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    }
    .insert(&db)
    .await;
    assert!(invalid_reference.is_err(), "user FK must be enforced");
    let invalid_status = teacher::Entity::update_many()
        .col_expr(
            teacher::Column::EmploymentStatus,
            sea_orm::sea_query::Expr::value("invalid"),
        )
        .filter(teacher::Column::Id.eq(91001))
        .exec(&db)
        .await;
    assert!(invalid_status.is_err(), "status check must be enforced");
    let teacher = teacher_oa_service::repositories::identity::current_user(&db, 91001)
        .await
        .expect("read seeded identity");
    assert_eq!(teacher.teacher.department_name, "数学教研组");
    assert!(
        teacher
            .grants
            .iter()
            .all(|grant| grant.scope == teacher_oa_service::domain::identity::DataScope::SelfOnly)
    );
    Migrator::down(&db, None)
        .await
        .expect("rollback all migrations");
    let schema = SchemaManager::new(&db);
    for table in [
        "users",
        "teachers",
        "user_identities",
        "user_roles",
        "audit_logs",
    ] {
        assert!(!schema.has_table(table).await.expect("inspect rollback"));
    }
    Migrator::up(&db, None)
        .await
        .expect("reapply after rollback");
    assert!(
        teacher::Entity::find()
            .all(&db)
            .await
            .expect("empty teachers")
            .is_empty()
    );
    db.close().await.expect("close PostgreSQL");
}
