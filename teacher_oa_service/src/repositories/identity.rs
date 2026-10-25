use super::entities::{department, identity, role_permission, teacher, user, user_role};
use crate::{
    api::error::ApiError,
    domain::identity::{CurrentUser, DataScope, PermissionGrant, TeacherProfile},
};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

pub async fn teacher_profile(
    db: &impl ConnectionTrait,
    id: i64,
) -> Result<TeacherProfile, ApiError> {
    let teacher = teacher::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::NotFound)?;
    let department = department::Entity::find_by_id(teacher.department_id)
        .one(db)
        .await?
        .ok_or(ApiError::Internal)?;
    Ok(TeacherProfile {
        id: teacher.id,
        user_id: teacher.user_id,
        employee_no: teacher.employee_no,
        name: teacher.name,
        department_id: teacher.department_id,
        department_name: department.name,
        subject_code: teacher.subject_code,
        employment_status: teacher.employment_status,
    })
}

pub async fn current_user(db: &impl ConnectionTrait, id: i64) -> Result<CurrentUser, ApiError> {
    let user = user::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if user.status != "active" {
        return Err(ApiError::Forbidden);
    }
    let teacher = teacher::Entity::find()
        .filter(teacher::Column::UserId.eq(id))
        .one(db)
        .await?
        .ok_or(ApiError::Forbidden)?;
    if teacher.employment_status != "active" {
        return Err(ApiError::Forbidden);
    }
    let teacher = teacher_profile(db, teacher.id).await?;
    let roles = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(id))
        .all(db)
        .await?;
    let mut grants = Vec::new();
    for role in &roles {
        let scope = DataScope::parse(&role.data_scope).ok_or(ApiError::Internal)?;
        let permissions = role_permission::Entity::find()
            .filter(role_permission::Column::RoleCode.eq(&role.role_code))
            .all(db)
            .await?;
        for permission in permissions {
            let grant = PermissionGrant {
                permission: permission.permission_code,
                scope: scope.clone(),
            };
            if !grants.contains(&grant) {
                grants.push(grant);
            }
        }
    }
    Ok(CurrentUser {
        id,
        display_name: user.display_name,
        teacher,
        roles: roles.into_iter().map(|role| role.role_code).collect(),
        grants,
    })
}

pub async fn find_identity(
    db: &impl ConnectionTrait,
    provider: &str,
    corp: &str,
    member: &str,
) -> Result<CurrentUser, ApiError> {
    let identity = identity::Entity::find()
        .filter(identity::Column::Provider.eq(provider))
        .filter(identity::Column::CorpId.eq(corp))
        .filter(identity::Column::ProviderUserid.eq(member))
        .one(db)
        .await?
        .ok_or(ApiError::Forbidden)?;
    current_user(db, identity.user_id).await
}
