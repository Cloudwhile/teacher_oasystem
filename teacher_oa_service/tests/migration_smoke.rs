//! Migration smoke tests.

use aster_forge_test::temp::SqliteTestDatabase;
use sea_orm::Database;
use sea_orm_migration::prelude::MigratorTrait;
use sea_orm_migration::prelude::SchemaManager;

#[test]
fn migrations_preserve_registered_names_and_order() {
    let migrations = teacher_oa_service_migration::Migrator::migrations();
    let names: Vec<_> = migrations
        .iter()
        .map(|migration| migration.name())
        .collect();

    assert_eq!(
        names,
        [
            "m20260627_000001_forge_foundation_schema",
            "m20261025_000002_identity_schema",
            "m20261025_000003_achievement_catalog",
            "m20261025_000004_achievement_records",
            "m20261030_000005_accounts_settings",
            "m20261031_000006_login_page",
            "m20261115_000007_identity_constraints",
        ]
    );
}

#[tokio::test]
async fn foundation_migration_applies_and_rolls_back_on_sqlite() {
    let database = SqliteTestDatabase::new("foundation-migration");
    let db = Database::connect(database.url())
        .await
        .expect("connect sqlite migration test database");

    teacher_oa_service_migration::Migrator::up(&db, None)
        .await
        .expect("apply foundation migration");
    let manager = SchemaManager::new(&db);

    for table in [
        "runtime_leases",
        "scheduled_tasks",
        "system_config",
        "mail_outbox",
        "audit_logs",
        "users",
        "departments",
        "teachers",
        "user_identities",
        "roles",
        "permissions",
        "role_permissions",
        "user_roles",
    ] {
        assert!(
            manager
                .has_table(table)
                .await
                .expect("query migrated table"),
            "expected {table} to be created"
        );
    }

    teacher_oa_service_migration::Migrator::down(&db, None)
        .await
        .expect("roll back foundation migration");
    for table in [
        "runtime_leases",
        "scheduled_tasks",
        "system_config",
        "mail_outbox",
        "audit_logs",
        "users",
        "departments",
        "teachers",
        "user_identities",
        "roles",
        "permissions",
        "role_permissions",
        "user_roles",
    ] {
        assert!(
            !manager
                .has_table(table)
                .await
                .expect("query rolled back table"),
            "expected {table} to be dropped"
        );
    }
    db.close()
        .await
        .expect("close sqlite migration test database");
}
