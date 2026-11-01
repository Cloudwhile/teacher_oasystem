use crate::api::collections::PermissionGrants;
#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::api::error::{Failure, Success};
use crate::{
    api::error::{ApiError, RequestError, success},
    domain::identity::CurrentUser,
    integrations::wecom::LoginMode,
    runtime::AppState,
    services::auth_service as auth,
};
use actix_web::{HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};

pub fn safe_next(value: Option<&str>) -> String {
    let value = value.unwrap_or("/dashboard");
    if value.len() <= 2048
        && !value.contains('\\')
        && !value.chars().any(char::is_control)
        && [
            "/dashboard",
            "/achievements",
            "/reviews",
            "/files",
            "/teachers",
            "/admin",
            "/account",
        ]
        .iter()
        .any(|prefix| {
            value == *prefix
                || value.starts_with(&format!("{prefix}/"))
                || value.starts_with(&format!("{prefix}?"))
        })
    {
        value.into()
    } else {
        "/dashboard".into()
    }
}

/// Attempts allowed per minute from one client address. Teachers behind a shared proxy all use
/// this budget, so it only stops a single flooding client; account limits do the real work.
const IP_ATTEMPTS: u32 = 300;
/// Attempts allowed per minute against one account.
const ACCOUNT_ATTEMPTS: u32 = 10;

fn peer_ip(request: &HttpRequest) -> String {
    request
        .peer_addr()
        .map(|p| p.ip().to_string())
        .unwrap_or_default()
}

/// Counts attempts in one-minute windows per `bucket:key`.
fn throttle(bucket: &str, key: &str, limit: u32) -> Result<(), ApiError> {
    use std::{
        collections::BTreeMap,
        sync::{Mutex, OnceLock},
    };
    static ATTEMPTS: OnceLock<Mutex<BTreeMap<String, (i64, u32)>>> = OnceLock::new();
    let mut attempts = ATTEMPTS
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .map_err(|_| ApiError::Internal)?;
    let now = chrono::Utc::now().timestamp();
    attempts.retain(|_, (start, _)| now - *start < 60);
    let key = format!("{bucket}:{key}");
    if !attempts.contains_key(&key) && attempts.len() >= 50_000 {
        return Err(ApiError::TooManyAttempts);
    }
    let entry = attempts.entry(key).or_insert((now, 0));
    entry.1 += 1;
    if entry.1 > limit {
        return Err(ApiError::TooManyAttempts);
    }
    Ok(())
}
#[aster_forge_api_docs_macros::path(post,path="/api/v1/auth/login",operation_id="auth_password_login",request_body=crate::services::account_service::PasswordLogin,responses((status=200,body=Success<AuthResponse>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=429,body=Failure),(status=500,body=Failure)))]
pub async fn password_login(
    request: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<crate::services::account_service::PasswordLogin>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        auth::ensure_origin(&request, &state.config.auth)?;
        throttle("login-ip", &peer_ip(&request), IP_ATTEMPTS)?;
        let account: String = body
            .username
            .trim()
            .to_ascii_lowercase()
            .chars()
            .take(64)
            .collect();
        throttle("login-account", &account, ACCOUNT_ATTEMPTS)?;
        let (user, verified_version) =
            crate::services::account_service::login(state.db_handles.writer(), body.into_inner())
                .await?;
        let must_change_password =
            crate::services::account_service::credential_for(state.db_handles.writer(), user.id)
                .await?
                .is_some_and(|c| c.must_change);
        let (cookie, session) = auth::create_session(&request, &state, user.clone()).await?;
        if session.credential_version != Some(verified_version) {
            state
                .auth_store
                .delete(&format!("teacher_oa:session:{}", cookie.value()))
                .await?;
            return Err(ApiError::InvalidCredentials);
        }
        let mut response = success(
            &request,
            AuthResponse {
                user,
                csrf_token: session.csrf_token,
                must_change_password,
            },
        )?;
        response
            .add_cookie(&cookie)
            .map_err(|_| ApiError::Internal)?;
        Ok(response)
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(post,path="/api/v1/auth/password",operation_id="auth_change_password",request_body=crate::services::account_service::PasswordChange,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<AuthResponse>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn change_password(
    request: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<crate::services::account_service::PasswordChange>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (session, user) = auth::authenticate(&request, &state).await?;
        auth::ensure_csrf(&request, &state.config.auth, &session)?;
        throttle("password-user", &user.id.to_string(), ACCOUNT_ATTEMPTS)?;
        let verified_version = crate::services::account_service::change(
            state.db_handles.writer(),
            user.id,
            body.into_inner(),
        )
        .await?;
        let (cookie, session) = auth::create_session(&request, &state, user.clone()).await?;
        if session.credential_version != Some(verified_version) {
            state
                .auth_store
                .delete(&format!("teacher_oa:session:{}", cookie.value()))
                .await?;
            return Err(ApiError::InvalidCredentials);
        }
        let mut response = success(
            &request,
            AuthResponse {
                user,
                csrf_token: session.csrf_token,
                must_change_password: false,
            },
        )?;
        response
            .add_cookie(&cookie)
            .map_err(|_| ApiError::Internal)?;
        Ok(response)
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}

#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LoginOptions {
    pub password_enabled: bool,
    pub wecom_enabled: bool,
    pub mock_enabled: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct MockLogin {
    pub identity: String,
}

#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AuthResponse {
    pub user: CurrentUser,
    pub csrf_token: String,
    pub must_change_password: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginQuery {
    pub mode: Option<LoginMode>,
    pub next: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[aster_forge_api_docs_macros::path(get, path = "/api/v1/auth/options", operation_id = "auth_options", responses((status = 200, body = Success<LoginOptions>)))]
pub async fn options(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    success(
        &request,
        LoginOptions {
            password_enabled: true,
            wecom_enabled: crate::services::settings_service::read(&state)
                .await
                .map_err(|e| e.at(&request))?
                .enabled,
            mock_enabled: state.config.auth.mock_enabled
                && state.config.auth.environment != "production",
        },
    )
    .map_err(|error| error.at(&request))
}

#[aster_forge_api_docs_macros::path(post, path = "/api/v1/auth/mock/login", operation_id = "auth_mock_login", request_body = MockLogin, responses((status = 200, body = Success<AuthResponse>), (status = 400, body = Failure), (status = 403, body = Failure), (status = 503, body = Failure), (status = 500, body = Failure)))]
pub async fn mock_login(
    request: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<MockLogin>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        if !state.config.auth.mock_enabled || state.config.auth.environment == "production" {
            return Err(ApiError::Forbidden);
        }
        auth::ensure_origin(&request, &state.config.auth)?;
        if !["teacher", "leader", "reviewer", "admin"].contains(&body.identity.as_str()) {
            return Err(ApiError::Invalid("请选择有效身份"));
        }
        let user = crate::repositories::identity::find_identity(
            state.db_handles.writer(),
            "mock",
            "development",
            &body.identity,
        )
        .await?;
        let (cookie, session) = auth::create_session(&request, &state, user.clone()).await?;
        let mut response = success(
            &request,
            AuthResponse {
                user,
                csrf_token: session.csrf_token,
                must_change_password: false,
            },
        )?;
        response
            .add_cookie(&cookie)
            .map_err(|_| ApiError::Internal)?;
        Ok(response)
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
}

#[aster_forge_api_docs_macros::path(get, path = "/api/v1/auth/me", operation_id = "auth_me", security(("session" = [])), responses((status = 200, body = Success<AuthResponse>), (status = 401, body = Failure), (status = 403, body = Failure), (status = 503, body = Failure), (status = 500, body = Failure)))]
pub async fn me(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (session, user) = auth::authenticate(&request, &state).await?;
        success(
            &request,
            AuthResponse {
                must_change_password: crate::services::account_service::credential_for(
                    state.db_handles.writer(),
                    user.id,
                )
                .await?
                .is_some_and(|c| c.must_change),
                user,
                csrf_token: session.csrf_token,
            },
        )
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
}

#[aster_forge_api_docs_macros::path(get, path = "/api/v1/auth/permissions", operation_id = "auth_permissions", security(("session" = [])), responses((status = 200, body = Success<PermissionGrants>), (status = 401, body = Failure), (status = 403, body = Failure), (status = 503, body = Failure), (status = 500, body = Failure)))]
pub async fn permissions(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        success(&request, PermissionGrants(user.grants))
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
}

#[aster_forge_api_docs_macros::path(post, path = "/api/v1/auth/logout", operation_id = "auth_logout", security(("session" = [], "csrf" = [])), responses((status = 204), (status = 401, body = Failure), (status = 403, body = Failure), (status = 503, body = Failure), (status = 500, body = Failure)))]
pub async fn logout(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let session = auth::load_session(&request, &state).await?;
        auth::ensure_csrf(&request, &state.config.auth, &session)?;
        let cookie = request
            .cookie(auth::SESSION_COOKIE)
            .ok_or(ApiError::Unauthorized)?;
        state
            .auth_store
            .delete(&format!("teacher_oa:session:{}", cookie.value()))
            .await?;
        auth::audit(
            &state,
            session.user_id,
            "auth.logout",
            Some(session.user_id),
        )
        .await?;
        Ok(HttpResponse::NoContent()
            .insert_header(("Cache-Control", "no-store"))
            .cookie(auth::clear_cookie(&state.config.auth, auth::SESSION_COOKIE))
            .finish())
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
}

#[aster_forge_api_docs_macros::path(get, path = "/api/v1/auth/wecom/login", operation_id = "auth_wecom_login", params(("mode" = Option<String>, Query, description = "web 或 h5")), responses((status = 302), (status = 400, body = Failure), (status = 503, body = Failure), (status = 500, body = Failure)))]
pub async fn wecom_login(
    request: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<LoginQuery>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        // Each call stores two state records for OAUTH_TTL; bound how fast one client adds them.
        throttle("wecom-ip", &peer_ip(&request), IP_ATTEMPTS)?;
        let value = auth::random_token();
        let binding = auth::random_token();
        let (adapter, _) = crate::services::settings_service::adapter(&state).await?;
        let url = adapter.authorization_url(&value, query.mode.unwrap_or(LoginMode::Web))?;
        state
            .auth_store
            .put(
                &format!("teacher_oa:oauth-next:{value}"),
                safe_next(query.next.as_deref()).into_bytes(),
                auth::OAUTH_TTL,
            )
            .await?;
        state
            .auth_store
            .put(
                &format!("teacher_oa:oauth:{value}"),
                binding.as_bytes().to_vec(),
                auth::OAUTH_TTL,
            )
            .await?;
        Ok(HttpResponse::Found()
            .insert_header(("Location", url))
            .insert_header(("Cache-Control", "no-store"))
            .cookie(auth::cookie(
                &state.config.auth,
                auth::OAUTH_COOKIE,
                binding,
                auth::OAUTH_TTL,
            ))
            .finish())
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
}

#[aster_forge_api_docs_macros::path(get, path = "/api/v1/auth/wecom/callback", operation_id = "auth_wecom_callback", params(("code" = Option<String>, Query), ("state" = Option<String>, Query)), responses((status = 303, description = "跳转工作台或登录错误页"), (status = 400, body = Failure)))]
pub async fn wecom_callback(
    request: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<CallbackQuery>,
) -> HttpResponse {
    let result = async {
        if query.error.is_some() {
            return Err(ApiError::Unauthorized);
        }
        let value = query.state.as_deref().ok_or(ApiError::Unauthorized)?;
        let binding = request
            .cookie(auth::OAUTH_COOKIE)
            .ok_or(ApiError::Unauthorized)?;
        auth::consume_state(&state, value, binding.value()).await?;
        let code = query
            .code
            .as_deref()
            .filter(|code| !code.is_empty() && code.len() <= 512)
            .ok_or(ApiError::Unauthorized)?;
        let (adapter, config) = crate::services::settings_service::adapter(&state).await?;
        let member = adapter.exchange(code).await?;
        if member.corp_id != config.wecom_corp_id {
            return Err(ApiError::Forbidden);
        }
        let user = crate::repositories::identity::find_identity(
            state.db_handles.writer(),
            "wecom",
            &member.corp_id,
            &member.member_id,
        )
        .await?;
        let (cookie, _) = auth::create_session(&request, &state, user).await?;
        let next = state
            .auth_store
            .take(&format!("teacher_oa:oauth-next:{value}"))
            .await?
            .and_then(|v| String::from_utf8(v).ok())
            .unwrap_or_else(|| "/dashboard".into());
        Ok((cookie, safe_next(Some(&next))))
    }
    .await;
    let mut response = HttpResponse::SeeOther();
    response
        .insert_header(("Cache-Control", "no-store"))
        .cookie(auth::clear_cookie(&state.config.auth, auth::OAUTH_COOKIE));
    match result {
        Ok((cookie, next)) => response
            .insert_header(("Location", next))
            .cookie(cookie)
            .finish(),
        Err(_error) => response
            .insert_header(("Location", "/login?error=authorization_failed"))
            .finish(),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/auth/options", web::get().to(options))
        .route("/auth/login", web::post().to(password_login))
        .route("/auth/password", web::post().to(change_password))
        .route("/auth/mock/login", web::post().to(mock_login))
        .route("/auth/me", web::get().to(me))
        .route("/auth/permissions", web::get().to(permissions))
        .route("/auth/logout", web::post().to(logout))
        .route("/auth/wecom/login", web::get().to(wecom_login))
        .route("/auth/wecom/callback", web::get().to(wecom_callback));
}
