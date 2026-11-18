use crate::schema_helpers::{insert, n};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        insert(
            m,
            "roles",
            &["code", "name"],
            vec!["school_leader".into(), "校级行政".into()],
        )
        .await?;
        insert(
            m,
            "permissions",
            &["code", "name"],
            vec!["teachers:directory".into(), "查阅全校教师资料".into()],
        )
        .await?;
        for (role, permission) in [
            ("school_leader", "teachers:read"),
            ("school_leader", "teachers:directory"),
            ("school_leader", "achievements:read"),
            ("school_leader", "files:read"),
            ("admin", "teachers:directory"),
        ] {
            insert(
                m,
                "role_permissions",
                &["role_code", "permission_code"],
                vec![role.into(), permission.into()],
            )
            .await?;
        }
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute(
            &Query::delete()
                .from_table(n("user_roles"))
                .and_where(Expr::col(n("role_code")).eq("school_leader"))
                .to_owned(),
        )
        .await?;
        conn.execute(
            &Query::delete()
                .from_table(n("role_permissions"))
                .cond_where(
                    Condition::any()
                        .add(Expr::col(n("role_code")).eq("school_leader"))
                        .add(Expr::col(n("permission_code")).eq("teachers:directory")),
                )
                .to_owned(),
        )
        .await?;
        conn.execute(
            &Query::delete()
                .from_table(n("permissions"))
                .and_where(Expr::col(n("code")).eq("teachers:directory"))
                .to_owned(),
        )
        .await?;
        conn.execute(
            &Query::delete()
                .from_table(n("roles"))
                .and_where(Expr::col(n("code")).eq("school_leader"))
                .to_owned(),
        )
        .await?;
        Ok(())
    }
}
