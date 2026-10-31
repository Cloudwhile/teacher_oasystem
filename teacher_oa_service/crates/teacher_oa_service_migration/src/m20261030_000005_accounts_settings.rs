use crate::schema_helpers::*;
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(n("password_credentials"))
                .col(text("username", 64).primary_key())
                .col(bigint("user_id").unique_key())
                .col(text("password_hash", 512))
                .col(integer("version").default(1))
                .col(integer("failed_attempts").default(0))
                .col(ColumnDef::new(n("locked_until")).timestamp_with_time_zone())
                .col(
                    ColumnDef::new(n("must_change"))
                        .boolean()
                        .not_null()
                        .default(true),
                )
                .foreign_key(&mut fk("password_credentials", "user_id", "users", "id"))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(n("login_settings"))
                .col(integer("id").primary_key())
                .col(ColumnDef::new(n("enabled")).boolean().not_null())
                .col(text("corp_id", 128))
                .col(text("agent_id", 64))
                .col(text("secret_ciphertext", 4096))
                .col(integer("version").default(1))
                .to_owned(),
        )
        .await?;
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(n("login_settings")).to_owned())
            .await?;
        m.drop_table(Table::drop().table(n("password_credentials")).to_owned())
            .await
    }
}
