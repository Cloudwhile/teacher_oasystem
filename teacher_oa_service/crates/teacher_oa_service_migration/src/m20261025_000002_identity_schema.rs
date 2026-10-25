use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

fn name(value: &str) -> Alias {
    Alias::new(value)
}
fn id() -> ColumnDef {
    let mut column = ColumnDef::new(name("id"));
    column
        .big_integer()
        .not_null()
        .auto_increment()
        .primary_key();
    column
}
fn text(value: &str, length: u32) -> ColumnDef {
    let mut column = ColumnDef::new(name(value));
    column.string_len(length).not_null();
    column
}
fn reference(value: &str) -> ColumnDef {
    let mut column = ColumnDef::new(name(value));
    column.big_integer().not_null();
    column
}
fn created_at() -> ColumnDef {
    let mut column = ColumnDef::new(name("created_at"));
    column
        .timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp());
    column
}
fn fk(table: &str, column: &str, target: &str, target_column: &str) -> ForeignKeyCreateStatement {
    ForeignKey::create()
        .name(format!("fk_{table}_{column}"))
        .from(name(table), name(column))
        .to(name(target), name(target_column))
        .on_delete(ForeignKeyAction::Restrict)
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(name("users"))
                    .col(id())
                    .col(text("display_name", 100))
                    .col(text("status", 20).default("active"))
                    .col(created_at())
                    .check(Expr::cust("status IN ('active', 'disabled')"))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(name("departments"))
                    .col(id())
                    .col(text("name", 100))
                    .col(ColumnDef::new(name("parent_id")).big_integer())
                    .col(
                        ColumnDef::new(name("wecom_department_id"))
                            .string_len(100)
                            .unique_key(),
                    )
                    .foreign_key(&mut fk("departments", "parent_id", "departments", "id"))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(name("teachers"))
                    .col(id())
                    .col(reference("user_id").unique_key())
                    .col(text("employee_no", 50).unique_key())
                    .col(text("name", 100))
                    .col(reference("department_id"))
                    .col(text("subject_code", 50))
                    .col(text("employment_status", 20).default("active"))
                    .col(created_at())
                    .check(Expr::cust("employment_status IN ('active', 'inactive')"))
                    .foreign_key(&mut fk("teachers", "user_id", "users", "id"))
                    .foreign_key(&mut fk("teachers", "department_id", "departments", "id"))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(name("user_identities"))
                    .col(id())
                    .col(reference("user_id"))
                    .col(text("provider", 32))
                    .col(text("corp_id", 100))
                    .col(text("provider_userid", 100))
                    .col(created_at())
                    .index(
                        Index::create()
                            .name("uq_identity_provider_corp_member")
                            .unique()
                            .col(name("provider"))
                            .col(name("corp_id"))
                            .col(name("provider_userid")),
                    )
                    .foreign_key(&mut fk("user_identities", "user_id", "users", "id"))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(name("roles"))
                    .col(text("code", 32).primary_key())
                    .col(text("name", 100))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(name("permissions"))
                    .col(text("code", 64).primary_key())
                    .col(text("name", 100))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(name("role_permissions"))
                    .col(text("role_code", 32))
                    .col(text("permission_code", 64))
                    .primary_key(
                        Index::create()
                            .col(name("role_code"))
                            .col(name("permission_code")),
                    )
                    .foreign_key(&mut fk("role_permissions", "role_code", "roles", "code"))
                    .foreign_key(&mut fk(
                        "role_permissions",
                        "permission_code",
                        "permissions",
                        "code",
                    ))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(name("user_roles"))
                    .col(reference("user_id"))
                    .col(text("role_code", 32))
                    .col(text("data_scope", 16))
                    .primary_key(Index::create().col(name("user_id")).col(name("role_code")))
                    .check(Expr::cust("data_scope IN ('SELF', 'DEPARTMENT', 'ALL')"))
                    .foreign_key(&mut fk("user_roles", "user_id", "users", "id"))
                    .foreign_key(&mut fk("user_roles", "role_code", "roles", "code"))
                    .to_owned(),
            )
            .await?;
        for (table, column) in [
            ("teachers", "department_id"),
            ("user_identities", "user_id"),
        ] {
            manager
                .create_index(
                    Index::create()
                        .name(format!("idx_{table}_{column}"))
                        .table(name(table))
                        .col(name(column))
                        .to_owned(),
                )
                .await?;
        }
        let connection = manager.get_connection();
        for (table, rows) in [
            (
                "roles",
                vec![
                    ("teacher", "教师"),
                    ("group_leader", "教研组长"),
                    ("reviewer", "审核人员"),
                    ("admin", "管理员"),
                ],
            ),
            (
                "permissions",
                vec![
                    ("teachers:read", "查看教师档案"),
                    ("achievements:write", "填报成果"),
                    ("reviews:write", "审核成果"),
                    ("admin:manage", "管理系统"),
                ],
            ),
        ] {
            for (code, label) in rows {
                let statement = Query::insert()
                    .into_table(name(table))
                    .columns([name("code"), name("name")])
                    .values([code.into(), label.into()])
                    .map_err(|error| DbErr::Custom(error.to_string()))?
                    .to_owned();
                connection.execute(&statement).await?;
            }
        }
        for (role, permissions) in [
            ("teacher", vec!["teachers:read", "achievements:write"]),
            ("group_leader", vec!["teachers:read"]),
            ("reviewer", vec!["teachers:read", "reviews:write"]),
            ("admin", vec!["teachers:read", "admin:manage"]),
        ] {
            for permission in permissions {
                let statement = Query::insert()
                    .into_table(name("role_permissions"))
                    .columns([name("role_code"), name("permission_code")])
                    .values([role.into(), permission.into()])
                    .map_err(|error| DbErr::Custom(error.to_string()))?
                    .to_owned();
                connection.execute(&statement).await?;
            }
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in [
            "user_roles",
            "role_permissions",
            "permissions",
            "roles",
            "user_identities",
            "teachers",
            "departments",
            "users",
        ] {
            manager
                .drop_table(Table::drop().table(name(table)).to_owned())
                .await?;
        }
        Ok(())
    }
}
