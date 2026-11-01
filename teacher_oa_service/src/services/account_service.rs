use crate::{
    api::error::ApiError,
    domain::identity::CurrentUser,
    repositories::{
        entities::{credential, department, identity, teacher, user, user_role},
        identity as identities,
    },
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use chrono::Utc;
use sea_orm::ExprTrait;

pub async fn initialize_admin(
    state: &crate::runtime::AppState,
) -> Result<std::path::PathBuf, ApiError> {
    use sea_orm::PaginatorTrait;
    use std::io::Write;
    if credential::Entity::find()
        .count(state.db_handles.writer())
        .await?
        != 0
    {
        return Err(ApiError::Conflict("已有账号，不能重复初始化管理员"));
    }
    let directory = std::path::Path::new(&state.config.storage.root);
    std::fs::create_dir_all(directory).map_err(|_| ApiError::Internal)?;
    let path = directory.join("initial-admin.txt");
    let password = format!("OA-{}", super::auth_service::random_token());
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&path)
        .map_err(|_| ApiError::Conflict("初始凭据文件已存在或无法创建"))?;
    write!(
        file,
        "账号：admin\n初始密码：{password}\n首次登录后请修改密码，并妥善删除此文件。\n"
    )
    .map_err(|_| ApiError::Internal)?;
    file.sync_all().map_err(|_| ApiError::Internal)?;
    save(
        state.db_handles.writer(),
        None,
        "",
        AccountInput {
            id: None,
            username: "admin".into(),
            name: "系统管理员".into(),
            department: "管理中心".into(),
            role: "admin".into(),
            active: true,
            password: Some(password),
            wecom_userid: None,
        },
    )
    .await?;
    Ok(path.to_path_buf())
}

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait, sea_query::Expr,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PasswordLogin {
    pub username: String,
    pub password: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PasswordChange {
    pub current_password: String,
    pub new_password: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AccountInput {
    pub id: Option<i64>,
    pub username: String,
    pub name: String,
    pub department: String,
    pub role: String,
    pub active: bool,
    pub password: Option<String>,
    pub wecom_userid: Option<String>,
}
#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AccountItem {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub department: String,
    pub role: String,
    pub active: bool,
    pub wecom_userid: Option<String>,
    pub must_change: bool,
}
#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct Accounts {
    pub items: Vec<AccountItem>,
}

pub async fn credential_for(
    db: &impl ConnectionTrait,
    id: i64,
) -> Result<Option<credential::Model>, ApiError> {
    Ok(credential::Entity::find()
        .filter(credential::Column::UserId.eq(id))
        .one(db)
        .await?)
}
pub fn check_password(password: &str) -> Result<(), ApiError> {
    if !(12..=128).contains(&password.chars().count()) || password.trim() != password {
        return Err(ApiError::Invalid("密码须为12至128位，首尾不能有空格"));
    }
    Ok(())
}
pub async fn hash(password: String) -> Result<String, ApiError> {
    check_password(&password)?;
    tokio::task::spawn_blocking(move || {
        let salt =
            SaltString::encode_b64(&rand::random::<[u8; 16]>()).map_err(|_| ApiError::Internal)?;
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|v| v.to_string())
            .map_err(|_| ApiError::Internal)
    })
    .await
    .map_err(|_| ApiError::Internal)?
}
async fn verify(password: String, hash: String) -> Result<bool, ApiError> {
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&hash).map_err(|_| ApiError::Internal)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await
    .map_err(|_| ApiError::Internal)?
}
pub async fn login(
    db: &DatabaseConnection,
    input: PasswordLogin,
) -> Result<(CurrentUser, i32), ApiError> {
    let username = input.username.trim().to_ascii_lowercase();
    if username.len() > 64 || input.password.len() > 512 {
        return Err(ApiError::InvalidCredentials);
    }
    let row = credential::Entity::find_by_id(username).one(db).await?;
    let Some(row) = row else {
        // Equivalent password work without exposing whether an account exists.
        let _ = hash("unknown-account-dummy-password".into()).await?;
        return Err(ApiError::InvalidCredentials);
    };
    if row.locked_until.is_some_and(|until| until > Utc::now()) {
        return Err(ApiError::InvalidCredentials);
    }
    if !verify(input.password, row.password_hash.clone()).await? {
        credential::Entity::update_many()
            .col_expr(
                credential::Column::FailedAttempts,
                Expr::col(credential::Column::FailedAttempts).add(1),
            )
            .filter(credential::Column::Username.eq(&row.username))
            .exec(db)
            .await?;
        if let Some(latest) = credential::Entity::find_by_id(&row.username)
            .one(db)
            .await?
            && latest.failed_attempts >= 5
        {
            credential::Entity::update_many()
                .col_expr(
                    credential::Column::LockedUntil,
                    Expr::value(Some(Utc::now() + chrono::Duration::minutes(15))),
                )
                .col_expr(credential::Column::FailedAttempts, Expr::value(0))
                .filter(credential::Column::Username.eq(&row.username))
                .exec(db)
                .await?;
        }
        return Err(ApiError::InvalidCredentials);
    }
    let user = identities::current_user(db, row.user_id)
        .await
        .map_err(|_| ApiError::InvalidCredentials)?;
    credential::Entity::update_many()
        .col_expr(credential::Column::FailedAttempts, Expr::value(0))
        .col_expr(
            credential::Column::LockedUntil,
            Expr::value(None::<chrono::DateTime<Utc>>),
        )
        .filter(credential::Column::Username.eq(row.username))
        .filter(credential::Column::Version.eq(row.version))
        .exec(db)
        .await?;
    Ok((user, row.version))
}
pub async fn change(
    db: &DatabaseConnection,
    id: i64,
    input: PasswordChange,
) -> Result<i32, ApiError> {
    let row = credential_for(db, id).await?.ok_or(ApiError::Forbidden)?;
    if input.current_password.len() > 512
        || !verify(input.current_password.clone(), row.password_hash.clone()).await?
    {
        return Err(ApiError::Invalid("当前密码不正确"));
    }
    if input.current_password == input.new_password {
        return Err(ApiError::Invalid("新密码不能与当前密码相同"));
    }
    let hash = hash(input.new_password).await?;
    let txn = db.begin().await?;
    let result = credential::Entity::update_many()
        .col_expr(credential::Column::PasswordHash, Expr::value(hash))
        .col_expr(credential::Column::Version, Expr::value(row.version + 1))
        .col_expr(credential::Column::MustChange, Expr::value(false))
        .col_expr(credential::Column::FailedAttempts, Expr::value(0))
        .col_expr(
            credential::Column::LockedUntil,
            Expr::value(None::<chrono::DateTime<Utc>>),
        )
        .filter(credential::Column::UserId.eq(id))
        .filter(credential::Column::Version.eq(row.version))
        .exec(&txn)
        .await?;
    if result.rows_affected != 1 {
        return Err(ApiError::Conflict("账号已更新，请重新登录"));
    }
    super::business_audit::write(&txn, id, "account.password_change", "user", Some(id)).await?;
    txn.commit().await?;
    Ok(row.version + 1)
}
pub async fn accounts(db: &DatabaseConnection) -> Result<Accounts, ApiError> {
    let mut items = Vec::new();
    for row in credential::Entity::find()
        .order_by_asc(credential::Column::Username)
        .all(db)
        .await?
    {
        let person = teacher::Entity::find()
            .filter(teacher::Column::UserId.eq(row.user_id))
            .one(db)
            .await?
            .ok_or(ApiError::Internal)?;
        let department = department::Entity::find_by_id(person.department_id)
            .one(db)
            .await?
            .ok_or(ApiError::Internal)?;
        let account = user::Entity::find_by_id(row.user_id)
            .one(db)
            .await?
            .ok_or(ApiError::Internal)?;
        let roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(row.user_id))
            .all(db)
            .await?;
        let binding = identity::Entity::find()
            .filter(identity::Column::UserId.eq(row.user_id))
            .filter(identity::Column::Provider.eq("wecom"))
            .one(db)
            .await?;
        items.push(AccountItem {
            id: row.user_id,
            username: row.username,
            name: person.name,
            department: department.name,
            role: roles
                .iter()
                .find(|r| r.role_code != "teacher")
                .map(|r| r.role_code.clone())
                .unwrap_or("teacher".into()),
            active: account.status == "active",
            wecom_userid: binding.map(|b| b.provider_userid),
            must_change: row.must_change,
        });
    }
    Ok(Accounts { items })
}
pub async fn save(
    db: &DatabaseConnection,
    actor: Option<i64>,
    corp: &str,
    input: AccountInput,
) -> Result<i64, ApiError> {
    let username = input.username.trim().to_ascii_lowercase();
    if !(3..=64).contains(&username.len())
        || !username
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(ApiError::Invalid(
            "账号须为3至64位字母、数字、点、下划线或短横线",
        ));
    }
    if input.name.trim().is_empty()
        || input.name.chars().count() > 100
        || input.department.trim().is_empty()
        || input.department.chars().count() > 100
    {
        return Err(ApiError::Invalid("请填写有效姓名和部门"));
    }
    if !["teacher", "group_leader", "reviewer", "admin"].contains(&input.role.as_str()) {
        return Err(ApiError::Invalid("角色无效"));
    }
    if input.id.is_some() && input.id == actor && (!input.active || input.role != "admin") {
        return Err(ApiError::Invalid("不能停用自己或移除自己的管理员角色"));
    }
    let member = input.wecom_userid.as_deref().unwrap_or("").trim();
    if member.len() > 128 || member.chars().any(char::is_whitespace) {
        return Err(ApiError::Invalid("企业微信成员账号无效"));
    }
    if !member.is_empty() && corp.is_empty() {
        return Err(ApiError::Invalid("请先设置企业微信企业ID"));
    }
    if input.id.is_some()
        && input.id == actor
        && input.password.as_ref().is_some_and(|p| !p.is_empty())
    {
        return Err(ApiError::Invalid("请在账号安全中修改自己的密码"));
    }
    let password_hash = match input.password.filter(|p| !p.is_empty()) {
        Some(p) => Some(hash(p).await?),
        None => None,
    };
    if input.id.is_none() && password_hash.is_none() {
        return Err(ApiError::Invalid("请设置初始密码"));
    }
    let txn = db.begin().await?;
    let existing = credential::Entity::find_by_id(&username).one(&txn).await?;
    if existing.as_ref().map(|r| r.user_id) != input.id {
        return Err(ApiError::Conflict("账号已存在或账号信息已变更"));
    }
    let dept = match department::Entity::find()
        .filter(department::Column::Name.eq(input.department.trim()))
        .one(&txn)
        .await?
    {
        Some(d) => d,
        None => {
            department::ActiveModel {
                name: Set(input.department.trim().into()),
                ..Default::default()
            }
            .insert(&txn)
            .await?
        }
    };
    let id = if let Some(id) = input.id {
        user::Entity::update_many()
            .col_expr(user::Column::DisplayName, Expr::value(input.name.trim()))
            .col_expr(
                user::Column::Status,
                Expr::value(if input.active { "active" } else { "disabled" }),
            )
            .filter(user::Column::Id.eq(id))
            .exec(&txn)
            .await?;
        teacher::Entity::update_many()
            .col_expr(teacher::Column::Name, Expr::value(input.name.trim()))
            .col_expr(teacher::Column::DepartmentId, Expr::value(dept.id))
            .filter(teacher::Column::UserId.eq(id))
            .exec(&txn)
            .await?;
        let old = existing.ok_or(ApiError::NotFound)?;
        // A new version revokes existing sessions, so re-enabling never revives them.
        let mut update = credential::Entity::update_many().col_expr(
            credential::Column::Version,
            Expr::value(old.version + i32::from(password_hash.is_some() || !input.active)),
        );
        if let Some(hash) = password_hash {
            update = update
                .col_expr(credential::Column::PasswordHash, Expr::value(hash))
                .col_expr(credential::Column::MustChange, Expr::value(true))
                .col_expr(credential::Column::FailedAttempts, Expr::value(0))
                .col_expr(
                    credential::Column::LockedUntil,
                    Expr::value(None::<chrono::DateTime<Utc>>),
                );
        }
        update
            .filter(credential::Column::UserId.eq(id))
            .exec(&txn)
            .await?;
        id
    } else {
        let account = user::ActiveModel {
            display_name: Set(input.name.trim().into()),
            status: Set(if input.active {
                "active".into()
            } else {
                "disabled".into()
            }),
            created_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        teacher::ActiveModel {
            user_id: Set(account.id),
            employee_no: Set(format!("OA-{username}")),
            name: Set(input.name.trim().into()),
            department_id: Set(dept.id),
            subject_code: Set("other".into()),
            employment_status: Set("active".into()),
            created_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        credential::ActiveModel {
            username: Set(username),
            user_id: Set(account.id),
            password_hash: Set(password_hash.ok_or(ApiError::Internal)?),
            version: Set(1),
            failed_attempts: Set(0),
            locked_until: Set(None),
            must_change: Set(true),
        }
        .insert(&txn)
        .await?;
        account.id
    };
    user_role::Entity::delete_many()
        .filter(user_role::Column::UserId.eq(id))
        .exec(&txn)
        .await?;
    user_role::ActiveModel {
        user_id: Set(id),
        role_code: Set("teacher".into()),
        data_scope: Set("SELF".into()),
    }
    .insert(&txn)
    .await?;
    if input.role != "teacher" {
        user_role::ActiveModel {
            user_id: Set(id),
            role_code: Set(input.role.clone()),
            data_scope: Set(if input.role == "group_leader" {
                "DEPARTMENT".into()
            } else {
                "ALL".into()
            }),
        }
        .insert(&txn)
        .await?;
    }
    if !member.is_empty() {
        let taken = identity::Entity::find()
            .filter(identity::Column::Provider.eq("wecom"))
            .filter(identity::Column::CorpId.eq(corp))
            .filter(identity::Column::ProviderUserid.eq(member))
            .one(&txn)
            .await?;
        if taken.is_some_and(|b| b.user_id != id) {
            return Err(ApiError::Conflict("该企业微信成员已绑定其他账号"));
        }
    }
    identity::Entity::delete_many()
        .filter(identity::Column::UserId.eq(id))
        .filter(identity::Column::Provider.eq("wecom"))
        .exec(&txn)
        .await?;
    if !member.is_empty() {
        identity::ActiveModel {
            user_id: Set(id),
            provider: Set("wecom".into()),
            corp_id: Set(corp.into()),
            provider_userid: Set(member.into()),
            created_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
    }
    super::business_audit::write(&txn, actor.unwrap_or(id), "account.save", "user", Some(id))
        .await?;
    txn.commit().await?;
    Ok(id)
}
