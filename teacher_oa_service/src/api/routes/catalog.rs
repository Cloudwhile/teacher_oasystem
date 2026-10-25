use crate::api::collections::*;
#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::api::error::{Failure, Success};
use crate::{
    api::error::{ApiError, RequestError, success},
    domain::{achievements::*, identity::DataScope},
    repositories::{achievements as repo, entities::teacher},
    runtime::AppState,
    services::auth_service as auth,
};
use actix_web::{HttpRequest, HttpResponse, web};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;

async fn authorized(request: &HttpRequest, state: &AppState) -> Result<(), ApiError> {
    let (_, user) = auth::authenticate(request, state).await?;
    if !repo::has_permission(&user, "achievements:read") {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}
#[derive(Deserialize)]
pub struct SchemaQuery {
    pub version: Option<i32>,
}

#[aster_forge_api_docs_macros::path(get,path="/api/v1/achievement-types",operation_id="achievement_types",security(("session"=[])),responses((status=200,body=Success<AchievementTypes>),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn types(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        authorized(&request, &state).await?;
        success(
            &request,
            AchievementTypes(repo::types(state.db_handles.writer()).await?),
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/achievement-types/{code}/schema",operation_id="achievement_schema",params(("code"=String,Path),("version"=Option<i32>,Query)),security(("session"=[])),responses((status=200,body=Success<AchievementSchema>),(status=401,body=Failure),(status=403,body=Failure),(status=404,body=Failure),(status=500,body=Failure)))]
pub async fn schema(
    request: HttpRequest,
    state: web::Data<AppState>,
    code: web::Path<String>,
    query: web::Query<SchemaQuery>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        authorized(&request, &state).await?;
        let schema = if let Some(version) = query.version {
            repo::schema(state.db_handles.writer(), &code, version).await?
        } else {
            repo::current_schema(state.db_handles.writer(), &code).await?
        };
        success(&request, schema)
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/academic-terms",operation_id="academic_terms",security(("session"=[])),responses((status=200,body=Success<AcademicTerms>),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn terms(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        authorized(&request, &state).await?;
        success(
            &request,
            AcademicTerms(repo::terms(state.db_handles.writer()).await?),
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/dictionaries",operation_id="dictionaries",security(("session"=[])),responses((status=200,body=Success<DictionaryItems>),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn dictionaries(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        authorized(&request, &state).await?;
        success(
            &request,
            DictionaryItems(repo::dictionaries(state.db_handles.writer()).await?),
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/teachers",operation_id="teacher_options",security(("session"=[])),responses((status=200,body=Success<TeacherOptions>),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn teachers(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        let mut scope = Condition::any().add(teacher::Column::Id.eq(user.teacher.id));
        for grant in &user.grants {
            if grant.permission == "teachers:read" {
                match grant.scope {
                    DataScope::SelfOnly => {}
                    DataScope::Department => {
                        scope =
                            scope.add(teacher::Column::DepartmentId.eq(user.teacher.department_id))
                    }
                    DataScope::All => {
                        scope = Condition::all();
                        break;
                    }
                }
            }
        }
        let rows = teacher::Entity::find()
            .filter(scope)
            .filter(teacher::Column::EmploymentStatus.eq("active"))
            .order_by_asc(teacher::Column::Name)
            .limit(300)
            .all(state.db_handles.writer())
            .await?;
        success(
            &request,
            TeacherOptions(
                rows.into_iter()
                    .map(|r| TeacherOption {
                        id: r.id,
                        name: r.name,
                        department_id: r.department_id,
                    })
                    .collect::<Vec<_>>(),
            ),
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/achievement-types", web::get().to(types))
        .route("/achievement-types/{code}/schema", web::get().to(schema))
        .route("/academic-terms", web::get().to(terms))
        .route("/dictionaries", web::get().to(dictionaries))
        .route("/teachers", web::get().to(teachers));
}
