use crate::{
    api::error::{ApiError, RequestError, success},
    runtime::AppState,
    services::auth_service as auth,
};
#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::{
    api::error::{Failure, Success},
    domain::identity::TeacherProfile,
};
use actix_web::{HttpRequest, HttpResponse, web};

async fn profile(
    request: &HttpRequest,
    state: &AppState,
    id: Option<i64>,
) -> Result<HttpResponse, ApiError> {
    let (_, user) = auth::authenticate(request, state).await?;
    // A forbidden teacher ID gets the same response whether it exists or not.
    let target_id = id.unwrap_or(user.teacher.id);
    if target_id != user.teacher.id
        && !user.grants.iter().any(|grant| {
            grant.permission == "teachers:read"
                && grant.scope != crate::domain::identity::DataScope::SelfOnly
        })
    {
        return Err(ApiError::Forbidden);
    }
    let target =
        crate::repositories::identity::teacher_profile(state.db_handles.writer(), target_id)
            .await?;
    if !user.can_read_teacher(&target) {
        return Err(ApiError::Forbidden);
    }
    auth::audit(state, user.id, "teacher.read", Some(target.id)).await?;
    success(request, target)
}

#[aster_forge_api_docs_macros::path(get, path = "/api/v1/teachers/me", operation_id = "teachers_me", security(("session" = [])), responses((status = 200, body = Success<TeacherProfile>), (status = 401, body = Failure), (status = 403, body = Failure), (status = 503, body = Failure), (status = 500, body = Failure)))]
pub async fn me(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    profile(&request, &state, None)
        .await
        .map_err(|error| error.at(&request))
}

#[aster_forge_api_docs_macros::path(get, path = "/api/v1/teachers/{id}", operation_id = "teachers_detail", params(("id" = i64, Path)), security(("session" = [])), responses((status = 200, body = Success<TeacherProfile>), (status = 401, body = Failure), (status = 403, body = Failure), (status = 404, body = Failure), (status = 503, body = Failure), (status = 500, body = Failure)))]
pub async fn detail(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse, RequestError> {
    profile(&request, &state, Some(*id))
        .await
        .map_err(|error| error.at(&request))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/teachers/me", web::get().to(me))
        .route("/teachers/{id}", web::get().to(detail));
}
