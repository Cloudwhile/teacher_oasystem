//! Explicit, non-production fixtures. Migrations never create login accounts.
use crate::repositories::entities::{department, identity, teacher, user, user_role};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};

pub async fn seed(db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let txn = db.begin().await?;
    for (id, name) in [(91001, "数学教研组"), (91002, "语文教研组")] {
        if department::Entity::find_by_id(id)
            .one(&txn)
            .await?
            .is_none()
        {
            department::ActiveModel {
                id: Set(id),
                name: Set(name.into()),
                parent_id: Set(None),
                wecom_department_id: Set(None),
            }
            .insert(&txn)
            .await?;
        }
    }
    for (id, member, label, department_id, subject, role, scope) in [
        (
            91001,
            "teacher",
            "测试教师",
            91001,
            "math",
            "teacher",
            "SELF",
        ),
        (
            91002,
            "leader",
            "测试组长",
            91001,
            "math",
            "group_leader",
            "DEPARTMENT",
        ),
        (
            91003,
            "reviewer",
            "测试审核员",
            91002,
            "chinese",
            "reviewer",
            "ALL",
        ),
        (
            91004,
            "admin",
            "测试管理员",
            91002,
            "chinese",
            "admin",
            "ALL",
        ),
    ] {
        if user::Entity::find_by_id(id).one(&txn).await?.is_some() {
            continue;
        }
        user::ActiveModel {
            id: Set(id),
            display_name: Set(label.into()),
            status: Set("active".into()),
            created_at: Set(Utc::now()),
        }
        .insert(&txn)
        .await?;
        teacher::ActiveModel {
            id: Set(id),
            user_id: Set(id),
            employee_no: Set(format!("DEMO-{id}")),
            name: Set(label.into()),
            department_id: Set(department_id),
            subject_code: Set(subject.into()),
            employment_status: Set("active".into()),
            created_at: Set(Utc::now()),
        }
        .insert(&txn)
        .await?;
        identity::ActiveModel {
            id: Set(id),
            user_id: Set(id),
            provider: Set("mock".into()),
            corp_id: Set("development".into()),
            provider_userid: Set(member.into()),
            created_at: Set(Utc::now()),
        }
        .insert(&txn)
        .await?;
        user_role::ActiveModel {
            user_id: Set(id),
            role_code: Set("teacher".into()),
            data_scope: Set("SELF".into()),
        }
        .insert(&txn)
        .await?;
        if role != "teacher" {
            user_role::ActiveModel {
                user_id: Set(id),
                role_code: Set(role.into()),
                data_scope: Set(scope.into()),
            }
            .insert(&txn)
            .await?;
        }
    }
    txn.commit().await
}
