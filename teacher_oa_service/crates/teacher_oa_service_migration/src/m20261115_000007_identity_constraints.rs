use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();
        // Deduplicate any concurrent duplicate department names before adding the unique index.
        connection
            .execute_unprepared(
                "UPDATE teachers SET department_id = (
                    SELECT MIN(d2.id) FROM departments d2
                    JOIN departments d1 ON d1.name = d2.name
                    WHERE d1.id = teachers.department_id
                ) WHERE department_id NOT IN (
                    SELECT MIN(id) FROM departments GROUP BY name
                )",
            )
            .await?;
        connection
            .execute_unprepared(
                "UPDATE departments SET parent_id = (
                    SELECT MIN(d2.id) FROM departments d2
                    JOIN departments d1 ON d1.name = d2.name
                    WHERE d1.id = departments.parent_id
                ) WHERE parent_id IS NOT NULL AND parent_id NOT IN (
                    SELECT MIN(id) FROM departments GROUP BY name
                )",
            )
            .await?;
        connection
            .execute_unprepared(
                "DELETE FROM departments WHERE id NOT IN (
                    SELECT MIN(id) FROM departments GROUP BY name
                )",
            )
            .await?;

        if manager.get_database_backend() == sea_orm::DbBackend::Postgres {
            for statement in [
                "ALTER TABLE teachers ALTER COLUMN employee_no TYPE VARCHAR(128)",
                "ALTER TABLE user_identities ALTER COLUMN corp_id TYPE VARCHAR(128)",
                "ALTER TABLE user_identities ALTER COLUMN provider_userid TYPE VARCHAR(128)",
            ] {
                connection.execute_unprepared(statement).await?;
            }
        }

        manager
            .create_index(
                Index::create()
                    .name("uq_departments_name")
                    .table(Alias::new("departments"))
                    .unique()
                    .col(Alias::new("name"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq_departments_name")
                    .table(Alias::new("departments"))
                    .to_owned(),
            )
            .await?;

        if manager.get_database_backend() == sea_orm::DbBackend::Postgres {
            let connection = manager.get_connection();
            for statement in [
                "ALTER TABLE teachers ALTER COLUMN employee_no TYPE VARCHAR(50) USING LEFT(employee_no, 50)",
                "ALTER TABLE user_identities ALTER COLUMN corp_id TYPE VARCHAR(100) USING LEFT(corp_id, 100)",
                "ALTER TABLE user_identities ALTER COLUMN provider_userid TYPE VARCHAR(100) USING LEFT(provider_userid, 100)",
            ] {
                connection.execute_unprepared(statement).await?;
            }
        }
        Ok(())
    }
}
