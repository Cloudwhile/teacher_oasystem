use crate::{
    api::error::ApiError, config::AuthConfig, domain::identity::CurrentUser,
    repositories::identity, runtime::AppState,
};
use actix_web::{
    HttpRequest,
    cookie::{Cookie, SameSite, time::Duration},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

pub const SESSION_COOKIE: &str = "teacher_oa_session";
pub const OAUTH_COOKIE: &str = "teacher_oa_oauth";
pub const OAUTH_TTL: u64 = 600;

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: i64,
    pub csrf_token: String,
    pub expires_at: i64,
}

pub fn random_token() -> String {
    rand::random::<[u8; 32]>()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn valid_token(token: &str) -> bool {
    token.len() == 64 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn cookie(config: &AuthConfig, name: &'static str, value: String, ttl: u64) -> Cookie<'static> {
    Cookie::build(name, value)
        .path("/")
        .http_only(true)
        .secure(config.cookie_secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(i64::try_from(ttl).unwrap_or(0)))
        .finish()
}

pub fn clear_cookie(config: &AuthConfig, name: &'static str) -> Cookie<'static> {
    let mut cookie = cookie(config, name, String::new(), 0);
    cookie.make_removal();
    cookie
}

pub fn ensure_origin(request: &HttpRequest, config: &AuthConfig) -> Result<(), ApiError> {
    let origin = request
        .headers()
        .get("origin")
        .and_then(|value| value.to_str().ok());
    if origin != Some(config.public_origin.as_str()) {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

pub fn ensure_csrf(
    request: &HttpRequest,
    config: &AuthConfig,
    session: &Session,
) -> Result<(), ApiError> {
    ensure_origin(request, config)?;
    let token = request
        .headers()
        .get("x-csrf-token")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Forbidden)?;
    if token
        .as_bytes()
        .ct_eq(session.csrf_token.as_bytes())
        .unwrap_u8()
        != 1
    {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

pub async fn load_session(request: &HttpRequest, state: &AppState) -> Result<Session, ApiError> {
    let token = request
        .cookie(SESSION_COOKIE)
        .map(|cookie| cookie.value().to_string())
        .ok_or(ApiError::Unauthorized)?;
    if !valid_token(&token) {
        return Err(ApiError::Unauthorized);
    }
    let bytes = state
        .auth_store
        .get(&format!("teacher_oa:session:{token}"))
        .await?
        .ok_or(ApiError::Unauthorized)?;
    let session: Session = serde_json::from_slice(&bytes).map_err(|_| ApiError::Unauthorized)?;
    if session.expires_at <= Utc::now().timestamp() {
        return Err(ApiError::Unauthorized);
    }
    Ok(session)
}

pub async fn authenticate(
    request: &HttpRequest,
    state: &AppState,
) -> Result<(Session, CurrentUser), ApiError> {
    let session = load_session(request, state).await?;
    // Use the writer so a disabled account or changed permission is effective immediately.
    let user = identity::current_user(state.db_handles.writer(), session.user_id).await?;
    Ok((session, user))
}

pub async fn audit(
    state: &AppState,
    user_id: i64,
    action: &str,
    target: Option<i64>,
) -> Result<(), ApiError> {
    aster_forge_db::audit_log::create_audit_log_row(
        state.db_handles.writer(),
        aster_forge_db::audit_log::AuditLogCreate {
            user_id,
            action: action.into(),
            entity_type: "identity".into(),
            entity_id: target,
            entity_name: None,
            details: None,
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        },
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "authentication audit write failed");
        ApiError::Internal
    })?;
    Ok(())
}

pub async fn create_session(
    request: &HttpRequest,
    state: &AppState,
    user: CurrentUser,
) -> Result<(Cookie<'static>, Session), ApiError> {
    let token = random_token();
    let ttl = state.config.auth.session_ttl_seconds;
    let session = Session {
        user_id: user.id,
        csrf_token: random_token(),
        expires_at: Utc::now().timestamp() + i64::try_from(ttl).map_err(|_| ApiError::Internal)?,
    };
    let key = format!("teacher_oa:session:{token}");
    state
        .auth_store
        .put(
            &key,
            serde_json::to_vec(&session).map_err(|_| ApiError::Internal)?,
            ttl,
        )
        .await?;
    if let Err(error) = audit(state, user.id, "auth.login", Some(user.id)).await {
        state.auth_store.delete(&key).await?;
        return Err(error);
    }
    if let Some(old) = request
        .cookie(SESSION_COOKIE)
        .filter(|cookie| valid_token(cookie.value()))
        && let Err(error) = state
            .auth_store
            .delete(&format!("teacher_oa:session:{}", old.value()))
            .await
    {
        state.auth_store.delete(&key).await?;
        return Err(error);
    }
    Ok((
        cookie(&state.config.auth, SESSION_COOKIE, token, ttl),
        session,
    ))
}

pub async fn consume_state(
    state: &AppState,
    value: &str,
    browser_binding: &str,
) -> Result<(), ApiError> {
    if !valid_token(value) || !valid_token(browser_binding) {
        return Err(ApiError::Invalid("登录已失效，请重新登录"));
    }
    let expected = state
        .auth_store
        .take(&format!("teacher_oa:oauth:{value}"))
        .await?
        .ok_or(ApiError::Invalid("登录已失效，请重新登录"))?;
    if expected.ct_eq(browser_binding.as_bytes()).unwrap_u8() != 1 {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}
