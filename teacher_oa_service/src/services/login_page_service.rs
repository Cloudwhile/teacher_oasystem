use crate::{api::error::ApiError, repositories::entities::login_page_settings, runtime::AppState};
use chrono::Utc;
use object_store::ObjectStoreExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait, sea_query::Expr};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const ASSET_PREFIX: &str = "/api/v1/login-page/assets/";

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LoginAnnouncement {
    pub id: String,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub published_at: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LoginPageSettings {
    pub title: String,
    pub background_url: String,
    pub logo_url: String,
    pub footer: String,
    pub announcements: Vec<LoginAnnouncement>,
    pub version: i32,
}

impl Default for LoginPageSettings {
    fn default() -> Self {
        Self {
            title: "TeacherOA · 教师综合 OA".into(),
            background_url: String::new(),
            logo_url: String::new(),
            footer: "TeacherOA · 教师工作空间".into(),
            announcements: Vec::new(),
            version: 0,
        }
    }
}

pub async fn read(state: &AppState) -> Result<LoginPageSettings, ApiError> {
    let row = login_page_settings::Entity::find_by_id(1)
        .one(state.db_handles.writer())
        .await?
        .ok_or(ApiError::Internal)?;
    let mut settings: LoginPageSettings =
        serde_json::from_str(&row.body).map_err(|_| ApiError::Internal)?;
    settings.version = row.version;
    Ok(settings)
}

pub async fn public(state: &AppState) -> Result<LoginPageSettings, ApiError> {
    let mut settings = read(state).await?;
    settings.announcements.retain(|item| item.published);
    settings
        .announcements
        .sort_by(|a, b| b.published_at.cmp(&a.published_at));
    Ok(settings)
}

pub fn asset_mime(key: &str) -> Result<&'static str, ApiError> {
    let (hash, ext) = key.rsplit_once('.').ok_or(ApiError::NotFound)?;
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ApiError::NotFound);
    }
    match ext {
        "png" => Ok("image/png"),
        "jpg" => Ok("image/jpeg"),
        _ => Err(ApiError::NotFound),
    }
}

fn validate(settings: &mut LoginPageSettings) -> Result<(), ApiError> {
    settings.title = settings.title.trim().into();
    settings.footer = settings.footer.trim().into();
    if settings.title.is_empty()
        || settings.title.chars().count() > 80
        || settings.footer.chars().count() > 300
    {
        return Err(ApiError::Invalid("请检查网站名称和页脚内容"));
    }
    for url in [&settings.background_url, &settings.logo_url] {
        if !url.is_empty() {
            let key = url
                .strip_prefix(ASSET_PREFIX)
                .ok_or(ApiError::Invalid("请上传登录页图片"))?;
            asset_mime(key).map_err(|_| ApiError::Invalid("登录页图片地址无效"))?;
        }
    }
    if settings.announcements.len() > 30 {
        return Err(ApiError::Invalid("最多保留30条公告"));
    }
    let mut ids = HashSet::new();
    for item in &mut settings.announcements {
        item.title = item.title.trim().into();
        item.content = item.content.trim().into();
        if item.id.is_empty()
            || item.id.len() > 64
            || !item
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !ids.insert(item.id.clone())
            || item.title.is_empty()
            || item.title.chars().count() > 120
            || item.content.is_empty()
            || item.content.chars().count() > 4000
        {
            return Err(ApiError::Invalid("请检查公告标题和正文"));
        }
    }
    if serde_json::to_vec(settings)
        .map_err(|_| ApiError::Internal)?
        .len()
        > 60000
    {
        return Err(ApiError::Invalid("公告内容过多，请精简后保存"));
    }
    Ok(())
}

pub async fn save(
    state: &AppState,
    actor: i64,
    mut settings: LoginPageSettings,
) -> Result<LoginPageSettings, ApiError> {
    validate(&mut settings)?;
    for url in [&settings.background_url, &settings.logo_url] {
        if let Some(key) = url.strip_prefix(ASSET_PREFIX) {
            let path = object_store::path::Path::from(format!("login-page/{key}"));
            state
                .files
                .head(&path)
                .await
                .map_err(|_| ApiError::Invalid("图片不存在，请重新上传"))?;
        }
    }
    let txn = state.db_handles.writer().begin().await?;
    let row = login_page_settings::Entity::find_by_id(1)
        .one(&txn)
        .await?
        .ok_or(ApiError::Internal)?;
    if row.version != settings.version {
        return Err(ApiError::Conflict("登录页配置已更新，请刷新后重试"));
    }
    let old: LoginPageSettings = serde_json::from_str(&row.body).map_err(|_| ApiError::Internal)?;
    for item in &mut settings.announcements {
        item.published_at = if item.published {
            old.announcements
                .iter()
                .find(|previous| previous.id == item.id && previous.published)
                .and_then(|previous| previous.published_at.clone())
                .or_else(|| Some(Utc::now().to_rfc3339()))
        } else {
            None
        };
    }
    settings.version = settings.version.checked_add(1).ok_or(ApiError::Internal)?;
    let body = serde_json::to_string(&settings).map_err(|_| ApiError::Internal)?;
    let changed = login_page_settings::Entity::update_many()
        .col_expr(login_page_settings::Column::Body, Expr::value(body))
        .col_expr(
            login_page_settings::Column::Version,
            Expr::value(settings.version),
        )
        .filter(login_page_settings::Column::Id.eq(1))
        .filter(login_page_settings::Column::Version.eq(row.version))
        .exec(&txn)
        .await?;
    if changed.rows_affected != 1 {
        return Err(ApiError::Conflict("登录页配置已更新，请刷新后重试"));
    }
    super::business_audit::write(
        &txn,
        actor,
        "settings.login_page",
        "login_page_settings",
        Some(1),
    )
    .await?;
    txn.commit().await?;
    Ok(settings)
}
