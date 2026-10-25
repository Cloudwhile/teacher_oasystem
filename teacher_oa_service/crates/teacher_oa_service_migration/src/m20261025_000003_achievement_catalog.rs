use crate::schema_helpers::*;
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(n("academic_terms"))
                .col(id())
                .col(text("academic_year", 9))
                .col(integer("term_number"))
                .col(text("name", 100).unique_key())
                .col(ColumnDef::new(n("starts_on")).date().not_null())
                .col(ColumnDef::new(n("ends_on")).date().not_null())
                .col(
                    ColumnDef::new(n("enabled"))
                        .boolean()
                        .not_null()
                        .default(true),
                )
                .index(
                    Index::create()
                        .unique()
                        .col(n("academic_year"))
                        .col(n("term_number")),
                )
                .check(Expr::cust("ends_on >= starts_on AND term_number IN (1, 2)"))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(n("dictionary_items"))
                .col(text("group_code", 64))
                .col(text("code", 64))
                .col(text("name", 100))
                .col(integer("sort_order"))
                .col(
                    ColumnDef::new(n("enabled"))
                        .boolean()
                        .not_null()
                        .default(true),
                )
                .col(ColumnDef::new(n("parent_code")).string_len(64))
                .primary_key(Index::create().col(n("group_code")).col(n("code")))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(n("achievement_types"))
                .col(text("code", 64).primary_key())
                .col(text("name", 100))
                .col(text("category", 32))
                .col(integer("current_version"))
                .col(
                    ColumnDef::new(n("published"))
                        .boolean()
                        .not_null()
                        .default(true),
                )
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(n("achievement_type_versions"))
                .col(text("type_code", 64))
                .col(integer("version"))
                .col(
                    ColumnDef::new(n("schema_document"))
                        .json_binary()
                        .not_null(),
                )
                .col(time("published_at"))
                .primary_key(Index::create().col(n("type_code")).col(n("version")))
                .foreign_key(&mut fk(
                    "achievement_type_versions",
                    "type_code",
                    "achievement_types",
                    "code",
                ))
                .to_owned(),
        )
        .await?;
        for (number, label, start, end) in [
            (1, "2026—2027 第一学期", "2026-09-01", "2027-01-31"),
            (2, "2026—2027 第二学期", "2027-02-01", "2027-08-31"),
        ] {
            insert(
                m,
                "academic_terms",
                &[
                    "academic_year",
                    "term_number",
                    "name",
                    "starts_on",
                    "ends_on",
                ],
                vec![
                    "2026-2027".into(),
                    number.into(),
                    label.into(),
                    chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d")
                        .map_err(|e| DbErr::Custom(e.to_string()))?
                        .into(),
                    chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d")
                        .map_err(|e| DbErr::Custom(e.to_string()))?
                        .into(),
                ],
            )
            .await?;
        }
        for (group, items) in crate::catalog::dictionaries() {
            for (order, (code, label)) in items.into_iter().enumerate() {
                insert(
                    m,
                    "dictionary_items",
                    &["group_code", "code", "name", "sort_order"],
                    vec![
                        group.into(),
                        code.into(),
                        label.into(),
                        <i32 as std::convert::TryFrom<usize>>::try_from(order)
                            .map_err(|e| DbErr::Custom(e.to_string()))?
                            .into(),
                    ],
                )
                .await?;
            }
        }
        for schema in crate::catalog::schemas() {
            let code = schema["code"]
                .as_str()
                .ok_or_else(|| DbErr::Custom("seed code missing".into()))?;
            insert(
                m,
                "achievement_types",
                &["code", "name", "category", "current_version"],
                vec![
                    code.into(),
                    schema["name"].as_str().unwrap_or_default().into(),
                    schema["category"].as_str().unwrap_or_default().into(),
                    1.into(),
                ],
            )
            .await?;
            insert(
                m,
                "achievement_type_versions",
                &["type_code", "version", "schema_document"],
                vec![code.into(), 1.into(), schema.clone().into()],
            )
            .await?;
        }
        for (code, label) in [
            ("achievements:read", "查看成果"),
            ("files:read", "查看材料"),
            ("files:write", "上传材料"),
        ] {
            insert(
                m,
                "permissions",
                &["code", "name"],
                vec![code.into(), label.into()],
            )
            .await?;
        }
        for role in ["teacher", "group_leader", "reviewer", "admin"] {
            for permission in ["achievements:read", "files:read"] {
                insert(
                    m,
                    "role_permissions",
                    &["role_code", "permission_code"],
                    vec![role.into(), permission.into()],
                )
                .await?;
            }
        }
        insert(
            m,
            "role_permissions",
            &["role_code", "permission_code"],
            vec!["teacher".into(), "files:write".into()],
        )
        .await?;
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for permission in ["achievements:read", "files:read", "files:write"] {
            m.get_connection()
                .execute(
                    &Query::delete()
                        .from_table(n("role_permissions"))
                        .and_where(Expr::col(n("permission_code")).eq(permission))
                        .to_owned(),
                )
                .await?;
            m.get_connection()
                .execute(
                    &Query::delete()
                        .from_table(n("permissions"))
                        .and_where(Expr::col(n("code")).eq(permission))
                        .to_owned(),
                )
                .await?;
        }
        for table in [
            "achievement_type_versions",
            "achievement_types",
            "dictionary_items",
            "academic_terms",
        ] {
            m.drop_table(Table::drop().table(n(table)).to_owned())
                .await?;
        }
        Ok(())
    }
}
