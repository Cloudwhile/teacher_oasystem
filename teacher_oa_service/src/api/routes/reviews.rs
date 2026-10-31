#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::api::error::{Failure, Success};
use crate::{
    api::error::{ApiError, RequestError, success},
    domain::achievements::{AchievementDetail, AchievementPage, ReviewRequest},
    repositories::{achievement_entities::achievement, achievements as repo},
    runtime::AppState,
    services::{
        auth_service as auth,
        review_service::{self, ReviewAction},
    },
};
use actix_web::{HttpRequest, HttpResponse, web};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueueQuery {
    pub search: Option<String>,
    pub type_code: Option<String>,
    pub academic_term_id: Option<i64>,
    pub teacher: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub status: Option<String>,
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/reviews/pending",operation_id="reviews_pending",params(("page"=Option<u64>,Query),("page_size"=Option<u64>,Query),("status"=Option<String>,Query),("search"=Option<String>,Query),("teacher"=Option<String>,Query),("type_code"=Option<String>,Query),("academic_term_id"=Option<i64>,Query)),security(("session"=[])),responses((status=200,body=Success<AchievementPage>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn pending(
    request: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<QueueQuery>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        if !repo::has_permission(&user, "reviews:write") {
            return Err(ApiError::Forbidden);
        }
        let status = query.status.as_deref().unwrap_or("pending");
        if !["pending", "approved", "returned", "archived", "history"].contains(&status) {
            return Err(ApiError::Invalid("审核列表状态无效"));
        }
        let (page, page_size) = super::achievements::pagination(query.page, query.page_size)?;
        let mut select = achievement::Entity::find().filter(repo::review_scope(&user));
        if status == "history" {
            use crate::repositories::achievement_entities::review_event;
            use sea_orm::{QuerySelect, QueryTrait};
            let ids = review_event::Entity::find()
                .select_only()
                .column(review_event::Column::AchievementId)
                .filter(review_event::Column::ReviewerId.eq(user.id))
                .filter(review_event::Column::Action.is_in(["approve", "return", "archive"]))
                .into_query();
            select = select.filter(achievement::Column::Id.in_subquery(ids));
        } else {
            select = select.filter(achievement::Column::Status.eq(status));
        }
        if let Some(search) = &query.search {
            if search.chars().count() > 100 {
                return Err(ApiError::Invalid("搜索内容过长"));
            }
            let text = search
                .replace('!', "!!")
                .replace('%', "!%")
                .replace('_', "!_");
            select = select.filter(
                achievement::Column::Title
                    .like(sea_orm::sea_query::LikeExpr::new(format!("%{text}%")).escape('!')),
            );
        }
        if let Some(name) = &query.teacher {
            use crate::repositories::entities::teacher;
            use sea_orm::{QuerySelect, QueryTrait};
            if name.chars().count() > 100 {
                return Err(ApiError::Invalid("搜索内容过长"));
            }
            let text = name
                .replace('!', "!!")
                .replace('%', "!%")
                .replace('_', "!_");
            let ids = teacher::Entity::find()
                .select_only()
                .column(teacher::Column::Id)
                .filter(
                    teacher::Column::Name
                        .like(sea_orm::sea_query::LikeExpr::new(format!("%{text}%")).escape('!')),
                )
                .into_query();
            select = select.filter(achievement::Column::TeacherId.in_subquery(ids));
        }
        if let Some(code) = &query.type_code {
            select = select.filter(achievement::Column::TypeCode.eq(code));
        }
        if let Some(term) = query.academic_term_id {
            select = select.filter(achievement::Column::AcademicTermId.eq(term));
        }
        let pager = select
            .order_by_asc(achievement::Column::UpdatedAt)
            .order_by_asc(achievement::Column::Id)
            .paginate(state.db_handles.writer(), page_size);
        let total = pager.num_items().await?;
        let page = page.min(total.div_ceil(page_size).max(1));
        let mut items = Vec::new();
        for row in pager.fetch_page(page - 1).await? {
            items.push(repo::item(state.db_handles.writer(), &row).await?);
        }
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

async fn mutate(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<ReviewRequest>,
    action: ReviewAction,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (session, user) = auth::authenticate(&request, &state).await?;
        auth::ensure_csrf(&request, &state.config.auth, &session)?;
        success(
            &request,
            review_service::review(
                state.db_handles.writer(),
                &user,
                *id,
                body.into_inner(),
                action,
            )
            .await?,
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
#[aster_forge_api_docs_macros::path(post,path="/api/v1/achievements/{id}/approve",operation_id="achievement_approve",params(("id"=i64,Path)),request_body=ReviewRequest,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<AchievementDetail>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn approve(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<ReviewRequest>,
) -> Result<HttpResponse, RequestError> {
    mutate(request, state, id, body, ReviewAction::Approve).await
}

#[aster_forge_api_docs_macros::path(post,path="/api/v1/achievements/{id}/return",operation_id="achievement_return",params(("id"=i64,Path)),request_body=ReviewRequest,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<AchievementDetail>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn return_for_correction(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<ReviewRequest>,
) -> Result<HttpResponse, RequestError> {
    mutate(request, state, id, body, ReviewAction::Return).await
}

#[aster_forge_api_docs_macros::path(post,path="/api/v1/achievements/{id}/archive",operation_id="achievement_archive",params(("id"=i64,Path)),request_body=ReviewRequest,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<AchievementDetail>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn archive(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
    body: web::Json<ReviewRequest>,
) -> Result<HttpResponse, RequestError> {
    mutate(request, state, id, body, ReviewAction::Archive).await
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/reviews/pending", web::get().to(pending))
        .route("/achievements/{id}/approve", web::post().to(approve))
        .route(
            "/achievements/{id}/return",
            web::post().to(return_for_correction),
        )
        .route("/achievements/{id}/archive", web::post().to(archive));
}
