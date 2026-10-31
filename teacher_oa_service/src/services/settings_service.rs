use crate::{
    api::error::ApiError,
    config::AuthConfig,
    integrations::wecom::{self, WeComAdapter},
    repositories::entities::login_settings,
    runtime::AppState,
};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::{io::Write, sync::Arc};

#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct WecomSettings {
    pub enabled: bool,
    pub corp_id: String,
    pub agent_id: String,
    pub callback_url: String,
    pub secret_configured: bool,
    pub version: i32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct SaveWecomSettings {
    pub enabled: bool,
    pub corp_id: String,
    pub agent_id: String,
    pub secret: Option<String>,
    pub version: i32,
}
fn key(state: &AppState) -> Result<Vec<u8>, ApiError> {
    let root = std::path::Path::new(&state.config.storage.root);
    std::fs::create_dir_all(root).map_err(|_| ApiError::Internal)?;
    let path = root.join(".settings-key");
    if !path.exists() {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(mut file) => {
                file.write_all(&rand::random::<[u8; 32]>())
                    .map_err(|_| ApiError::Internal)?;
                file.sync_all().map_err(|_| ApiError::Internal)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err(ApiError::Internal),
        }
    }
    let bytes = std::fs::read(path).map_err(|_| ApiError::Internal)?;
    if bytes.len() != 32 {
        return Err(ApiError::Internal);
    }
    Ok(bytes)
}
fn encrypt(state: &AppState, value: &str) -> Result<String, ApiError> {
    let cipher = Aes256Gcm::new_from_slice(&key(state)?).map_err(|_| ApiError::Internal)?;
    let nonce = rand::random::<[u8; 12]>();
    let ciphertext = cipher
        .encrypt(&Nonce::from(nonce), value.as_bytes())
        .map_err(|_| ApiError::Internal)?;
    Ok(nonce
        .iter()
        .chain(ciphertext.iter())
        .map(|b| format!("{b:02x}"))
        .collect())
}
fn decrypt(state: &AppState, value: &str) -> Result<String, ApiError> {
    if value.len() < 56
        || !value.len().is_multiple_of(2)
        || !value.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(ApiError::Internal);
    }
    // Never generate a replacement key for an existing encrypted secret.
    if !std::path::Path::new(&state.config.storage.root)
        .join(".settings-key")
        .exists()
    {
        return Err(ApiError::Unavailable);
    }
    let bytes: Vec<u8> = (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).map_err(|_| ApiError::Internal))
        .collect::<Result<_, _>>()?;
    let cipher = Aes256Gcm::new_from_slice(&key(state)?).map_err(|_| ApiError::Internal)?;
    let nonce: [u8; 12] = bytes[..12].try_into().map_err(|_| ApiError::Internal)?;
    let plain = cipher
        .decrypt(&Nonce::from(nonce), &bytes[12..])
        .map_err(|_| ApiError::Internal)?;
    String::from_utf8(plain).map_err(|_| ApiError::Internal)
}
pub async fn read(state: &AppState) -> Result<WecomSettings, ApiError> {
    let row = login_settings::Entity::find_by_id(1)
        .one(state.db_handles.writer())
        .await?;
    Ok(match row {
        Some(row) => WecomSettings {
            enabled: row.enabled,
            corp_id: row.corp_id,
            agent_id: row.agent_id,
            callback_url: format!(
                "{}/api/v1/auth/wecom/callback",
                state.config.auth.public_origin
            ),
            secret_configured: !row.secret_ciphertext.is_empty(),
            version: row.version,
        },
        None => WecomSettings {
            enabled: state.wecom.enabled(),
            corp_id: state.config.auth.wecom_corp_id.clone(),
            agent_id: state.config.auth.wecom_agent_id.clone(),
            callback_url: format!(
                "{}/api/v1/auth/wecom/callback",
                state.config.auth.public_origin
            ),
            secret_configured: std::env::var("WECOM_CORP_SECRET").is_ok_and(|s| !s.is_empty()),
            version: 0,
        },
    })
}
pub async fn adapter(state: &AppState) -> Result<(Arc<dyn WeComAdapter>, AuthConfig), ApiError> {
    let Some(row) = login_settings::Entity::find_by_id(1)
        .one(state.db_handles.writer())
        .await?
    else {
        return Ok((state.wecom.clone(), state.config.auth.clone()));
    };
    let mut config = state.config.auth.clone();
    config.wecom_corp_id = row.corp_id;
    config.wecom_agent_id = row.agent_id;
    config.wecom_redirect_uri = format!("{}/api/v1/auth/wecom/callback", config.public_origin);
    let secret = if row.enabled {
        decrypt(state, &row.secret_ciphertext)?
    } else {
        String::new()
    };
    Ok((
        wecom::build_with_secret(&config, secret).map_err(|_| ApiError::Unavailable)?,
        config,
    ))
}
pub async fn save(
    state: &AppState,
    actor: i64,
    input: SaveWecomSettings,
) -> Result<WecomSettings, ApiError> {
    let corp = input.corp_id.trim();
    let agent = input.agent_id.trim();
    if corp.len() > 128
        || agent.len() > 64
        || (!corp.is_empty() && !corp.bytes().all(|b| b.is_ascii_alphanumeric()))
        || (!agent.is_empty() && !agent.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(ApiError::Invalid("企业ID或应用AgentId格式不正确"));
    }
    let txn = state.db_handles.writer().begin().await?;
    let old = login_settings::Entity::find_by_id(1).one(&txn).await?;
    if old.as_ref().map_or(0, |r| r.version) != input.version {
        return Err(ApiError::Conflict("配置已更新，请刷新后重试"));
    }
    if let Some(previous) = &old
        && previous.corp_id != corp
        && input.secret.as_deref().unwrap_or("").is_empty()
    {
        return Err(ApiError::Invalid("更换企业时请重新填写Secret"));
    }
    let ciphertext = match input.secret.filter(|v| !v.is_empty()) {
        Some(secret) => {
            if secret.len() > 2048 || secret.trim() != secret {
                return Err(ApiError::Invalid("Secret格式不正确"));
            }
            encrypt(state, &secret)?
        }
        None => match &old {
            Some(old) => old.secret_ciphertext.clone(),
            None => match std::env::var("WECOM_CORP_SECRET") {
                Ok(secret) if !secret.is_empty() && corp == state.config.auth.wecom_corp_id => {
                    encrypt(state, &secret)?
                }
                _ => String::new(),
            },
        },
    };
    if input.enabled && (corp.is_empty() || agent.is_empty() || ciphertext.is_empty()) {
        return Err(ApiError::Invalid("请完整填写企业ID、AgentId和Secret"));
    }
    let mut model = login_settings::ActiveModel {
        id: Set(1),
        enabled: Set(input.enabled),
        corp_id: Set(corp.into()),
        agent_id: Set(agent.into()),
        secret_ciphertext: Set(ciphertext),
        version: Set(input.version + 1),
    };
    if old.is_some() {
        use sea_orm::{ColumnTrait, QueryFilter, sea_query::Expr};
        let result = login_settings::Entity::update_many()
            .col_expr(login_settings::Column::Enabled, Expr::value(input.enabled))
            .col_expr(login_settings::Column::CorpId, Expr::value(corp))
            .col_expr(login_settings::Column::AgentId, Expr::value(agent))
            .col_expr(
                login_settings::Column::SecretCiphertext,
                Expr::value(model.secret_ciphertext.take().ok_or(ApiError::Internal)?),
            )
            .col_expr(
                login_settings::Column::Version,
                Expr::value(input.version + 1),
            )
            .filter(login_settings::Column::Id.eq(1))
            .filter(login_settings::Column::Version.eq(input.version))
            .exec(&txn)
            .await?;
        if result.rows_affected != 1 {
            return Err(ApiError::Conflict("配置已更新，请刷新后重试"));
        }
    } else {
        model.insert(&txn).await?;
    }
    super::business_audit::write(&txn, actor, "settings.wecom", "login_settings", Some(1)).await?;
    txn.commit().await?;
    read(state).await
}
