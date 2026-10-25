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

#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LoginOptions {
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
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginQuery {
    pub mode: Option<LoginMode>,
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
            wecom_enabled: state.wecom.enabled(),
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
        let value = auth::random_token();
        let binding = auth::random_token();
        let url = state
            .wecom
            .authorization_url(&value, query.mode.unwrap_or(LoginMode::Web))?;
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
        let member = state.wecom.exchange(code).await?;
        if member.corp_id != state.config.auth.wecom_corp_id {
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
        Ok(cookie)
    }
    .await;
    let mut response = HttpResponse::SeeOther();
    response
        .insert_header(("Cache-Control", "no-store"))
        .cookie(auth::clear_cookie(&state.config.auth, auth::OAUTH_COOKIE));
    match result {
        Ok(cookie) => response
            .insert_header(("Location", "/dashboard"))
            .cookie(cookie)
            .finish(),
        Err(_error) => response
            .insert_header(("Location", "/login?error=authorization_failed"))
            .finish(),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/auth/options", web::get().to(options))
        .route("/auth/mock/login", web::post().to(mock_login))
        .route("/auth/me", web::get().to(me))
        .route("/auth/permissions", web::get().to(permissions))
        .route("/auth/logout", web::post().to(logout))
        .route("/auth/wecom/login", web::get().to(wecom_login))
        .route("/auth/wecom/callback", web::get().to(wecom_callback));
}
