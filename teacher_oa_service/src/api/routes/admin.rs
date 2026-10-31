#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::api::error::{Failure, Success};
use crate::{
    api::error::{ApiError, RequestError, success},
    runtime::AppState,
    services::{
        account_service::{self, AccountInput, Accounts},
        auth_service as auth,
        settings_service::{self, SaveWecomSettings, WecomSettings},
    },
};
use actix_web::{HttpRequest, HttpResponse, web};
pub(super) async fn admin(
    request: &HttpRequest,
    state: &AppState,
    write: bool,
) -> Result<i64, ApiError> {
    let (session, user) = auth::authenticate(request, state).await?;
    if !crate::repositories::achievements::has_permission(&user, "admin:manage") {
        return Err(ApiError::Forbidden);
    }
    if write {
        auth::ensure_csrf(request, &state.config.auth, &session)?;
    }
    Ok(user.id)
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/admin/accounts",operation_id="admin_accounts",security(("session"=[])),responses((status=200,body=Success<Accounts>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn accounts(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        admin(&request, &state, false).await?;
        success(
            &request,
            account_service::accounts(state.db_handles.writer()).await?,
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(post,path="/api/v1/admin/accounts",operation_id="admin_save_account",request_body=AccountInput,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<Accounts>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn save_account(
    request: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<AccountInput>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let actor = admin(&request, &state, true).await?;
        let settings = settings_service::read(&state).await?;
        account_service::save(
            state.db_handles.writer(),
            Some(actor),
            &settings.corp_id,
            body.into_inner(),
        )
        .await?;
        success(
            &request,
            account_service::accounts(state.db_handles.writer()).await?,
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/admin/wecom",operation_id="admin_wecom",security(("session"=[])),responses((status=200,body=Success<WecomSettings>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn wecom_settings(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        admin(&request, &state, false).await?;
        success(&request, settings_service::read(&state).await?)
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(put,path="/api/v1/admin/wecom",operation_id="admin_save_wecom",request_body=SaveWecomSettings,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<WecomSettings>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn save_wecom(
    request: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<SaveWecomSettings>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let actor = admin(&request, &state, true).await?;
        success(
            &request,
            settings_service::save(&state, actor, body.into_inner()).await?,
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/admin/accounts", web::get().to(accounts))
        .route("/admin/accounts", web::post().to(save_account))
        .route("/admin/wecom", web::get().to(wecom_settings))
        .route("/admin/wecom", web::put().to(save_wecom));
}
