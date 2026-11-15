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
    #[serde(default)]
    pub credential_version: Option<i32>,
    #[serde(default)]
    pub password_authenticated: bool,
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

fn is_loopback_origin(url: &url::Url) -> bool {
    match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        None => false,
    }
}

pub fn ensure_origin(request: &HttpRequest, config: &AuthConfig) -> Result<(), ApiError> {
    let Some(origin) = request
        .headers()
        .get("origin")
        .and_then(|value| value.to_str().ok())
    else {
        return Err(ApiError::OriginMismatch);
    };
    if origin == config.public_origin {
        return Ok(());
    }
    if config.environment != "production"
        && let (Ok(origin_url), Ok(expected_url)) = (
            url::Url::parse(origin),
            url::Url::parse(&config.public_origin),
        )
        && origin_url.origin().ascii_serialization() == origin
        && ["http", "https"].contains(&origin_url.scheme())
        && (expected_url.scheme() != "https" || origin_url.scheme() == "https")
    {
        let sec_fetch_site = request
            .headers()
            .get("sec-fetch-site")
            .and_then(|value| value.to_str().ok());
        let same_origin_fetch = sec_fetch_site.is_none_or(|site| site == "same-origin");

        if same_origin_fetch && is_loopback_origin(&origin_url) && is_loopback_origin(&expected_url)
        {
            let same_scheme_and_port = origin_url.scheme() == expected_url.scheme()
                && origin_url.port_or_known_default() == expected_url.port_or_known_default();
            let local_dev_peer = config.environment == "development"
                && origin_url.scheme() == "http"
                && request
                    .peer_addr()
                    .is_some_and(|peer| peer.ip().is_loopback());
            if same_scheme_and_port || local_dev_peer {
                return Ok(());
            }
        }

        if config.environment == "development"
            && same_origin_fetch
            && let Some(host) = request
                .headers()
                .get("host")
                .and_then(|value| value.to_str().ok())
            && let Some(authority) = origin
                .strip_prefix("http://")
                .or_else(|| origin.strip_prefix("https://"))
            && authority.eq_ignore_ascii_case(host.trim())
        {
            return Ok(());
        }
    }
    tracing::warn!(
        origin = %origin,
        expected = %config.public_origin,
        environment = %config.environment,
        "rejected request with mismatched Origin header"
    );
    Err(ApiError::OriginMismatch)
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
    let credentials =
        super::account_service::credential_for(state.db_handles.writer(), user.id).await?;
    if credentials.as_ref().map(|c| c.version) != session.credential_version {
        return Err(ApiError::Unauthorized);
    }
    if session.password_authenticated
        && credentials.is_some_and(|c| c.must_change)
        && ![
            "/api/v1/auth/me",
            "/api/v1/auth/password",
            "/api/v1/auth/logout",
        ]
        .contains(&request.path())
    {
        return Err(ApiError::Forbidden);
    }
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
    create_session_inner(request, state, user, false).await
}

pub async fn create_password_session(
    request: &HttpRequest,
    state: &AppState,
    user: CurrentUser,
) -> Result<(Cookie<'static>, Session), ApiError> {
    create_session_inner(request, state, user, true).await
}

async fn create_session_inner(
    request: &HttpRequest,
    state: &AppState,
    user: CurrentUser,
    password_authenticated: bool,
) -> Result<(Cookie<'static>, Session), ApiError> {
    let token = random_token();
    let ttl = state.config.auth.session_ttl_seconds;
    let session = Session {
        user_id: user.id,
        credential_version: super::account_service::credential_for(
            state.db_handles.writer(),
            user.id,
        )
        .await?
        .map(|c| c.version),
        csrf_token: random_token(),
        expires_at: Utc::now().timestamp() + i64::try_from(ttl).map_err(|_| ApiError::Internal)?,
        password_authenticated,
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

#[cfg(test)]
mod tests {
    use super::ensure_origin;
    use crate::config::AuthConfig;
    use actix_web::test::TestRequest;

    #[test]
    fn development_accepts_loopback_aliases_and_same_origin_hosts_while_production_stays_strict() {
        let dev_auth = AuthConfig::default();

        for origin in [
            "http://127.0.0.1:3000",
            "http://localhost:3000",
            "http://[::1]:3000",
        ] {
            let req = TestRequest::post()
                .insert_header(("Origin", origin))
                .to_http_request();
            assert!(ensure_origin(&req, &dev_auth).is_ok(), "{origin}");
        }

        let lan_same_origin = TestRequest::post()
            .insert_header(("Origin", "http://192.168.1.50:3000"))
            .insert_header(("Host", "192.168.1.50:3000"))
            .insert_header(("Sec-Fetch-Site", "same-origin"))
            .to_http_request();
        assert!(ensure_origin(&lan_same_origin, &dev_auth).is_ok());

        for bad in [
            TestRequest::post().to_http_request(),
            TestRequest::post()
                .insert_header(("Origin", "https://evil.example"))
                .insert_header(("Host", "127.0.0.1:3000"))
                .to_http_request(),
            TestRequest::post()
                .insert_header(("Origin", "http://192.168.1.50:3000"))
                .insert_header(("Host", "192.168.1.50:3000"))
                .insert_header(("Sec-Fetch-Site", "cross-site"))
                .to_http_request(),
        ] {
            assert!(ensure_origin(&bad, &dev_auth).is_err());
        }

        let prod_auth = AuthConfig {
            environment: "production".into(),
            public_origin: "https://oa.example.edu.cn".into(),
            cookie_secure: true,
            ..Default::default()
        };
        let prod_mismatch = TestRequest::post()
            .insert_header(("Origin", "https://other.example.edu.cn"))
            .insert_header(("Host", "other.example.edu.cn"))
            .insert_header(("Sec-Fetch-Site", "same-origin"))
            .to_http_request();
        assert!(ensure_origin(&prod_mismatch, &prod_auth).is_err());
    }
}
