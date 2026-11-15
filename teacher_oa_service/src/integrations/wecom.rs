use crate::{api::error::ApiError, config::AuthConfig};
use async_trait::async_trait;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoginMode {
    Web,
    H5,
}

pub struct WeComIdentity {
    pub corp_id: String,
    pub member_id: String,
}

#[async_trait]
pub trait WeComAdapter: Send + Sync {
    fn enabled(&self) -> bool;
    fn authorization_url(&self, state: &str, mode: LoginMode) -> Result<String, ApiError>;
    async fn exchange(&self, code: &str) -> Result<WeComIdentity, ApiError>;
}

struct DisabledWeComAdapter;

#[async_trait]
impl WeComAdapter for DisabledWeComAdapter {
    fn enabled(&self) -> bool {
        false
    }
    fn authorization_url(&self, _state: &str, _mode: LoginMode) -> Result<String, ApiError> {
        Err(ApiError::Invalid("企业微信登录未启用"))
    }
    async fn exchange(&self, _code: &str) -> Result<WeComIdentity, ApiError> {
        Err(ApiError::Unavailable)
    }
}

pub fn disabled() -> Arc<dyn WeComAdapter> {
    Arc::new(DisabledWeComAdapter)
}

struct RealWeComAdapter {
    config: AuthConfig,
    secret: String,
    client: reqwest::Client,
    cached_token: tokio::sync::Mutex<Option<(String, Instant)>>,
}

#[derive(Deserialize)]
struct TokenResponse {
    #[serde(default)]
    errcode: i64,
    access_token: Option<String>,
    expires_in: Option<u64>,
}
#[derive(Deserialize)]
struct MemberResponse {
    #[serde(default)]
    errcode: i64,
    #[serde(alias = "UserId")]
    userid: Option<String>,
}

impl RealWeComAdapter {
    async fn access_token(&self) -> Result<String, ApiError> {
        let mut cached = self.cached_token.lock().await;
        if let Some((token, expires_at)) = &*cached
            && Instant::now() < *expires_at
        {
            return Ok(token.clone());
        }
        // Never propagate reqwest errors: they may include URLs containing credentials.
        let token: TokenResponse = self
            .client
            .get("https://qyapi.weixin.qq.com/cgi-bin/gettoken")
            .query(&[
                ("corpid", self.config.wecom_corp_id.as_str()),
                ("corpsecret", self.secret.as_str()),
            ])
            .send()
            .await
            .map_err(|_| ApiError::Unavailable)?
            .error_for_status()
            .map_err(|_| ApiError::Unavailable)?
            .json()
            .await
            .map_err(|_| ApiError::Unavailable)?;
        if token.errcode != 0 {
            return Err(ApiError::Unavailable);
        }
        let access_token = token
            .access_token
            .filter(|value| !value.is_empty())
            .ok_or(ApiError::Unavailable)?;
        let ttl_secs = token.expires_in.unwrap_or(7200).saturating_sub(120).max(30);
        *cached = Some((
            access_token.clone(),
            Instant::now() + Duration::from_secs(ttl_secs),
        ));
        Ok(access_token)
    }
}

#[async_trait]
impl WeComAdapter for RealWeComAdapter {
    fn enabled(&self) -> bool {
        !self.config.wecom_corp_id.is_empty()
            && !self.config.wecom_agent_id.is_empty()
            && !self.config.wecom_redirect_uri.is_empty()
            && !self.secret.is_empty()
    }
    fn authorization_url(&self, state: &str, mode: LoginMode) -> Result<String, ApiError> {
        if !self.enabled() {
            return Err(ApiError::Unavailable);
        }
        let base = match mode {
            LoginMode::Web => "https://open.work.weixin.qq.com/wwopen/sso/qrConnect",
            LoginMode::H5 => "https://open.weixin.qq.com/connect/oauth2/authorize",
        };
        let mut url = url::Url::parse(base).map_err(|_| ApiError::Internal)?;
        url.query_pairs_mut()
            .append_pair("appid", &self.config.wecom_corp_id)
            .append_pair("agentid", &self.config.wecom_agent_id)
            .append_pair("redirect_uri", &self.config.wecom_redirect_uri)
            .append_pair("state", state);
        if matches!(mode, LoginMode::H5) {
            url.query_pairs_mut()
                .append_pair("response_type", "code")
                .append_pair("scope", "snsapi_base");
            url.set_fragment(Some("wechat_redirect"));
        }
        Ok(url.into())
    }
    async fn exchange(&self, code: &str) -> Result<WeComIdentity, ApiError> {
        if !self.enabled() {
            return Err(ApiError::Unavailable);
        }
        let token = self.access_token().await?;
        let member: MemberResponse = self
            .client
            .get("https://qyapi.weixin.qq.com/cgi-bin/auth/getuserinfo")
            .query(&[("access_token", token.as_str()), ("code", code)])
            .send()
            .await
            .map_err(|_| ApiError::Unavailable)?
            .error_for_status()
            .map_err(|_| ApiError::Unavailable)?
            .json()
            .await
            .map_err(|_| ApiError::Unavailable)?;
        if matches!(member.errcode, 40014 | 42001) {
            *self.cached_token.lock().await = None;
        }
        if member.errcode != 0 {
            return Err(ApiError::Unauthorized);
        }
        let member_id = member
            .userid
            .filter(|value| !value.is_empty())
            .ok_or(ApiError::Forbidden)?;
        Ok(WeComIdentity {
            corp_id: self.config.wecom_corp_id.clone(),
            member_id,
        })
    }
}

pub fn build(config: &AuthConfig) -> crate::errors::Result<Arc<dyn WeComAdapter>> {
    build_with_secret(
        config,
        std::env::var("WECOM_CORP_SECRET").unwrap_or_default(),
    )
}

pub fn build_with_secret(
    config: &AuthConfig,
    secret: String,
) -> crate::errors::Result<Arc<dyn WeComAdapter>> {
    if let Ok(allowed) = std::env::var("WECOM_ALLOWED_CORP_ID")
        && !allowed.is_empty()
        && allowed != config.wecom_corp_id
    {
        return Err(crate::errors::AppError::Config(
            "WeCom corp ID is outside the allowed organization".into(),
        ));
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| {
            crate::errors::AppError::Config("cannot initialize WeCom HTTP client".into())
        })?;
    Ok(Arc::new(RealWeComAdapter {
        config: config.clone(),
        secret,
        client,
        cached_token: tokio::sync::Mutex::new(None),
    }))
}

type AdapterMap = BTreeMap<[u8; 32], Arc<dyn WeComAdapter>>;

pub fn cached_with_secret(
    config: &AuthConfig,
    secret: String,
) -> crate::errors::Result<Arc<dyn WeComAdapter>> {
    static ADAPTERS: OnceLock<Mutex<AdapterMap>> = OnceLock::new();
    let mut hasher = Sha256::new();
    for part in [
        config.wecom_corp_id.as_str(),
        config.wecom_agent_id.as_str(),
        config.wecom_redirect_uri.as_str(),
        secret.as_str(),
    ] {
        hasher.update(u64::try_from(part.len()).unwrap_or(0).to_le_bytes());
        hasher.update(part.as_bytes());
    }
    let fingerprint: [u8; 32] = hasher.finalize().into();
    let cache = ADAPTERS.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Ok(guard) = cache.lock()
        && let Some(existing) = guard.get(&fingerprint)
    {
        return Ok(existing.clone());
    }
    let built = build_with_secret(config, secret)?;
    if let Ok(mut guard) = cache.lock() {
        if guard.len() >= 16 {
            guard.clear();
        }
        guard.insert(fingerprint, built.clone());
    }
    Ok(built)
}
