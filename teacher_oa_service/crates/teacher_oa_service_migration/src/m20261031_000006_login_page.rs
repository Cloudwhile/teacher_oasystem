use crate::schema_helpers::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(n("login_page_settings"))
                .col(integer("id").primary_key())
                .col(ColumnDef::new(n("body")).text().not_null())
                .col(integer("version").default(0))
                .to_owned(),
        )
        .await?;
        insert(
            m,
            "login_page_settings",
            &["id", "body", "version"],
            vec![1.into(), "{}".into(), 0.into()],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(n("login_page_settings")).to_owned())
            .await
    }
}
