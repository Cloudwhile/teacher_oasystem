use crate::api::collections::AchievementRevisions;
#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::api::error::{Failure, Success};
use crate::{
    api::error::{ApiError, RequestError, success},
    domain::achievements::*,
    repositories::{
        achievement_entities::{achievement, revision},
        achievements as repo,
    },
    runtime::AppState,
    services::{achievement_service as service, auth_service as auth, business_audit},
};
use actix_web::{HttpRequest, HttpResponse, web};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub search: Option<String>,
    pub type_code: Option<String>,
    pub academic_term_id: Option<i64>,
    pub status: Option<String>,
    pub level_code: Option<String>,
    pub sort: Option<String>,
    pub scope: Option<String>,
}
pub fn pagination(page: Option<u64>, size: Option<u64>) -> Result<(u64, u64), ApiError> {
    let page = page.unwrap_or(1);
    let size = size.unwrap_or(20);
    if page == 0 || page > 1_000_000 || !(1..=100).contains(&size) {
        return Err(ApiError::Invalid("分页参数无效"));
    }
    Ok((page, size))
}

#[aster_forge_api_docs_macros::path(get,path="/api/v1/achievements",operation_id="achievements_list",params(("page"=Option<u64>,Query),("page_size"=Option<u64>,Query),("search"=Option<String>,Query),("type_code"=Option<String>,Query),("academic_term_id"=Option<i64>,Query),("status"=Option<String>,Query),("level_code"=Option<String>,Query),("sort"=Option<String>,Query),("scope"=Option<String>,Query)),security(("session"=[])),responses((status=200,body=Success<AchievementPage>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn list(
    request: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        if !repo::has_permission(&user, "achievements:read") {
            return Err(ApiError::Forbidden);
        }
        let (page, page_size) = pagination(query.page, query.page_size)?;
        let mut select = achievement::Entity::find().filter(repo::scope(&user));
        match query.scope.as_deref().unwrap_or("all") {
            "mine" => select = select.filter(repo::personal_scope(&user)),
            "all" => {}
            _ => return Err(ApiError::Invalid("成果范围无效")),
        }
        if let Some(search) = &query.search {
            if search.chars().count() > 100 {
                return Err(ApiError::Invalid("搜索内容过长"));
            }
            let escaped = search
                .replace('!', "!!")
                .replace('%', "!%")
                .replace('_', "!_");
            select = select.filter(
                achievement::Column::Title
                    .like(sea_orm::sea_query::LikeExpr::new(format!("%{escaped}%")).escape('!')),
            );
        }
        if let Some(code) = &query.type_code {
            select = select.filter(achievement::Column::TypeCode.eq(code));
        }
        if let Some(term) = query.academic_term_id {
            select = select.filter(achievement::Column::AcademicTermId.eq(term));
        }
        if let Some(status) = &query.status {
            if !["draft", "pending", "returned", "approved", "archived"].contains(&status.as_str())
            {
                return Err(ApiError::Invalid("成果状态无效"));
            }
            select = select.filter(achievement::Column::Status.eq(status));
        }
        if let Some(level) = &query.level_code {
            select = select.filter(achievement::Column::LevelCode.eq(level));
        }
        select = match query.sort.as_deref().unwrap_or("updated_at_desc") {
            "updated_at_desc" => select.order_by_desc(achievement::Column::UpdatedAt),
            "occurred_at_desc" => select.order_by_desc(achievement::Column::OccurredAt),
            _ => return Err(ApiError::Invalid("排序方式无效")),
        };
        let pager = select
            .order_by_desc(achievement::Column::Id)
            .paginate(state.db_handles.writer(), page_size);
        let total = pager.num_items().await?;
        let page = page.min(total.div_ceil(page_size).max(1));
        let mut items = Vec::new();
        for row in pager.fetch_page(page - 1).await? {
            items.push(repo::item(state.db_handles.writer(), &row).await?);
        }
        business_audit::write(
            state.db_handles.writer(),
            user.id,
            "achievement.list",
            "achievement",
            None,
        )
        .await?;
        success(
            &request,
            AchievementPage {
                items,
                page,
                page_size,
                total,
            },
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(post,path="/api/v1/achievements",operation_id="achievement_create",request_body=AchievementInput,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<AchievementDetail>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=422,body=Failure),(status=500,body=Failure)))]
pub async fn create(
    request: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<AchievementInput>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (session, user) = auth::authenticate(&request, &state).await?;
        auth::ensure_csrf(&request, &state.config.auth, &session)?;
        success(
            &request,
            service::create(state.db_handles.writer(), &user, body.into_inner()).await?,
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/achievements/{id}",operation_id="achievement_detail",params(("id"=i64,Path)),security(("session"=[])),responses((status=200,body=Success<AchievementDetail>),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn detail(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        let row = repo::accessible(state.db_handles.writer(), &user, *id).await?;
        let detail = repo::detail(state.db_handles.writer(), &user, &row).await?;
        business_audit::write(
            state.db_handles.writer(),
            user.id,
            "achievement.read",
            "achievement",
            Some(*id),
        )
        .await?;
        success(&request, detail)
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(patch,path="/api/v1/achievements/{id}",operation_id="achievement_update",params(("id"=i64,Path)),request_body=UpdateAchievement,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<AchievementDetail>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=422,body=Failure),(status=500,body=Failure)))]
pub async fn update(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<UpdateAchievement>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (session, user) = auth::authenticate(&request, &state).await?;
        auth::ensure_csrf(&request, &state.config.auth, &session)?;
        success(
            &request,
            service::update(state.db_handles.writer(), &user, *id, body.into_inner()).await?,
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(post,path="/api/v1/achievements/{id}/submit",operation_id="achievement_submit",params(("id"=i64,Path)),request_body=VersionRequest,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<AchievementDetail>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=422,body=Failure),(status=500,body=Failure)))]
pub async fn submit(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<VersionRequest>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (session, user) = auth::authenticate(&request, &state).await?;
        auth::ensure_csrf(&request, &state.config.auth, &session)?;
        success(
            &request,
            service::submit(state.db_handles.writer(), &user, *id, body.version).await?,
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/achievements/{id}/revisions",operation_id="achievement_revisions",params(("id"=i64,Path)),security(("session"=[])),responses((status=200,body=Success<AchievementRevisions>),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn revisions(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        repo::accessible(state.db_handles.writer(), &user, *id).await?;
        let rows = revision::Entity::find()
            .filter(revision::Column::AchievementId.eq(*id))
            .order_by_desc(revision::Column::Version)
            .all(state.db_handles.writer())
            .await?;
        business_audit::write(
            state.db_handles.writer(),
            user.id,
            "achievement.revisions",
            "achievement",
            Some(*id),
        )
        .await?;
        let mut revisions = Vec::new();
        for r in rows {
            let snapshot: AchievementSnapshot =
                serde_json::from_value(r.snapshot).map_err(|_| ApiError::Internal)?;
            let mut files = Vec::new();
            for file_id in &snapshot.data.file_ids {
                files.push(repo::metadata(
                    repo::file_access(state.db_handles.writer(), &user, *file_id).await?,
                ));
            }
            revisions.push(RevisionItem {
                version: r.version,
                schema_version: r.schema_version,
                snapshot,
                files,
                modified_by: r.modified_by,
                created_at: r.created_at,
            });
        }
        success(&request, AchievementRevisions(revisions))
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/achievements", web::get().to(list))
        .route("/achievements", web::post().to(create))
        .route("/achievements/{id}", web::get().to(detail))
        .route("/achievements/{id}", web::patch().to(update))
        .route("/achievements/{id}/submit", web::post().to(submit))
        .route("/achievements/{id}/revisions", web::get().to(revisions));
}
