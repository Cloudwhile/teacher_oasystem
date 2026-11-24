use crate::{
    api::error::{ApiError, RequestError, success},
    repositories::identity::DirectoryFilter,
    runtime::AppState,
    services::{auth_service as auth, business_audit},
};
#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::{
    api::error::{Failure, Success},
    domain::identity::{TeacherDirectoryPage, TeacherDossier, TeacherProfile},
};
use actix_web::{HttpRequest, HttpResponse, web};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DirectoryQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub search: Option<String>,
    pub department_id: Option<i64>,
    pub subject_code: Option<String>,
    pub employment_status: Option<String>,
}

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

#[aster_forge_api_docs_macros::path(get, path = "/api/v1/teachers/directory", operation_id = "teachers_directory", params(("page" = Option<u64>, Query), ("page_size" = Option<u64>, Query), ("search" = Option<String>, Query), ("department_id" = Option<i64>, Query), ("subject_code" = Option<String>, Query), ("employment_status" = Option<String>, Query)), security(("session" = [])), responses((status = 200, body = Success<TeacherDirectoryPage>), (status = 400, body = Failure), (status = 401, body = Failure), (status = 403, body = Failure), (status = 500, body = Failure)))]
pub async fn directory(
    request: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<DirectoryQuery>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        if !user.can_read_teacher_directory() {
            return Err(ApiError::Forbidden);
        }
        let (page, page_size) = super::achievements::pagination(query.page, query.page_size)?;
        let result = crate::repositories::identity::teacher_directory(
            state.db_handles.writer(),
            DirectoryFilter {
                page,
                page_size,
                search: query.search.as_deref(),
                department_id: query.department_id,
                subject_code: query.subject_code.as_deref(),
                employment_status: query.employment_status.as_deref(),
            },
        )
        .await?;
        business_audit::write(
            state.db_handles.writer(),
            user.id,
            "teacher.directory_list",
            "teacher",
            None,
        )
        .await?;
        success(&request, result)
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
}

#[aster_forge_api_docs_macros::path(get, path = "/api/v1/teachers/{id}/dossier", operation_id = "teachers_dossier", params(("id" = i64, Path)), security(("session" = [])), responses((status = 200, body = Success<TeacherDossier>), (status = 401, body = Failure), (status = 403, body = Failure), (status = 404, body = Failure), (status = 500, body = Failure)))]
pub async fn dossier(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        if !user.can_read_teacher_directory() {
            return Err(ApiError::Forbidden);
        }
        let dossier =
            crate::repositories::identity::teacher_dossier(state.db_handles.writer(), &user, *id)
                .await?;
        business_audit::write(
            state.db_handles.writer(),
            user.id,
            "teacher.dossier_read",
            "teacher",
            Some(dossier.profile.id),
        )
        .await?;
        success(&request, dossier)
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
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
    use crate::api::common::get_resource;

    cfg.service(get_resource("/teachers/me", me))
        .service(get_resource("/teachers/directory", directory))
        .service(get_resource("/teachers/{id}/dossier", dossier))
        .service(get_resource("/teachers/{id}", detail));
}
