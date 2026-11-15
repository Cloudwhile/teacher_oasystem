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
    Set, TransactionTrait, sea_query::Expr,
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
    if !(12..=128).contains(&password.chars().count())
        || password.trim() != password
        || password.contains('\0')
    {
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
    let corp = crate::repositories::entities::login_settings::Entity::find_by_id(1)
        .one(db)
        .await?
        .map(|row| row.corp_id)
        .unwrap_or_default();
    accounts_for_corp(db, &corp).await
}

pub async fn accounts_for_corp(db: &DatabaseConnection, corp: &str) -> Result<Accounts, ApiError> {
    use std::collections::BTreeMap;
    let teachers: BTreeMap<i64, teacher::Model> = teacher::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|row| (row.user_id, row))
        .collect();
    let departments: BTreeMap<i64, String> = department::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|row| (row.id, row.name))
        .collect();
    let users: BTreeMap<i64, user::Model> = user::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|row| (row.id, row))
        .collect();
    let mut roles: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for row in user_role::Entity::find().all(db).await? {
        roles.entry(row.user_id).or_default().push(row.role_code);
    }
    let credentials: BTreeMap<i64, credential::Model> = credential::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|row| (row.user_id, row))
        .collect();
    let mut bindings: BTreeMap<i64, String> = BTreeMap::new();
    if !corp.trim().is_empty() {
        for row in identity::Entity::find()
            .filter(identity::Column::Provider.eq("wecom"))
            .filter(identity::Column::CorpId.eq(corp.trim()))
            .all(db)
            .await?
        {
            bindings.insert(row.user_id, row.provider_userid);
        }
    }
    let mut items = Vec::new();
    for (&user_id, person) in &teachers {
        let account = users.get(&user_id).ok_or(ApiError::Internal)?;
        let department = departments
            .get(&person.department_id)
            .cloned()
            .ok_or(ApiError::Internal)?;
        let cred = credentials.get(&user_id);
        let username = cred
            .map(|row| row.username.clone())
            .unwrap_or_else(|| person.employee_no.to_ascii_lowercase());
        let role = roles
            .get(&user_id)
            .and_then(|list| list.iter().find(|code| code.as_str() != "teacher"))
            .cloned()
            .unwrap_or_else(|| "teacher".into());
        items.push(AccountItem {
            id: user_id,
            username,
            name: person.name.clone(),
            department,
            role,
            active: account.status == "active",
            wecom_userid: bindings.get(&user_id).cloned(),
            must_change: cred.is_some_and(|row| row.must_change),
        });
    }
    items.sort_by(|left, right| {
        left.username
            .cmp(&right.username)
            .then_with(|| left.id.cmp(&right.id))
    });
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
        || input.name.contains('\0')
        || input.department.trim().is_empty()
        || input.department.chars().count() > 100
        || input.department.contains('\0')
    {
        return Err(ApiError::Invalid("请填写有效姓名和部门"));
    }
    if !["teacher", "group_leader", "reviewer", "admin"].contains(&input.role.as_str()) {
        return Err(ApiError::Invalid("角色无效"));
    }
    if input.id.is_some() && input.id == actor && (!input.active || input.role != "admin") {
        return Err(ApiError::Invalid("不能停用自己或移除自己的管理员角色"));
    }
    let corp = corp.trim();
    let member = input.wecom_userid.as_deref().unwrap_or("").trim();
    if member.len() > 128 || member.chars().any(|c| c.is_whitespace() || c.is_control()) {
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
    let dept_name = input.department.trim();
    let dept = match department::Entity::find()
        .filter(department::Column::Name.eq(dept_name))
        .one(db)
        .await?
    {
        Some(d) => d,
        None => match (department::ActiveModel {
            name: Set(dept_name.into()),
            ..Default::default()
        })
        .insert(db)
        .await
        {
            Ok(d) => d,
            Err(_) => department::Entity::find()
                .filter(department::Column::Name.eq(dept_name))
                .one(db)
                .await?
                .ok_or(ApiError::Internal)?,
        },
    };
    let txn = db.begin().await?;
    let existing = credential::Entity::find_by_id(&username).one(&txn).await?;
    if existing
        .as_ref()
        .is_some_and(|row| Some(row.user_id) != input.id)
    {
        return Err(ApiError::Conflict("账号已存在或账号信息已变更"));
    }
    let id = if let Some(id) = input.id {
        if user::Entity::find_by_id(id).one(&txn).await?.is_none() {
            return Err(ApiError::NotFound);
        }
        let current_cred = credential_for(&txn, id).await?;
        if current_cred
            .as_ref()
            .is_some_and(|cred| cred.username != username)
        {
            return Err(ApiError::Conflict("账号已存在或账号信息已变更"));
        }
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
        if current_cred.is_some() {
            // Increment the persisted version atomically; ordinary profile saves leave it alone.
            if password_hash.is_some() || !input.active {
                let mut update = credential::Entity::update_many().col_expr(
                    credential::Column::Version,
                    Expr::col(credential::Column::Version).add(1),
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
            }
        } else if let Some(hash) = password_hash {
            credential::ActiveModel {
                username: Set(username),
                user_id: Set(id),
                password_hash: Set(hash),
                version: Set(1),
                failed_attempts: Set(0),
                locked_until: Set(None),
                must_change: Set(true),
            }
            .insert(&txn)
            .await?;
        }
        id
    } else {
        if teacher::Entity::find()
            .filter(teacher::Column::EmployeeNo.eq(format!("OA-{username}")))
            .one(&txn)
            .await?
            .is_some()
        {
            return Err(ApiError::Conflict("账号已存在或账号信息已变更"));
        }
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
