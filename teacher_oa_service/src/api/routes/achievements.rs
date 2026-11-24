use crate::api::collections::AchievementRevisions;
#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::api::error::{Failure, Success};
use crate::{
    api::error::{ApiError, RequestError, success},
    domain::achievements::*,
    repositories::{
        achievement_entities::{achievement, participant, revision},
        achievements as repo,
        entities::teacher,
    },
    runtime::AppState,
    services::{achievement_service as service, auth_service as auth, business_audit},
};
use actix_web::{HttpRequest, HttpResponse, web};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, ExprTrait, PaginatorTrait, QueryFilter, QueryOrder,
    sea_query::{Expr, LikeExpr, Query},
};
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
    pub teacher_id: Option<i64>,
    pub department_id: Option<i64>,
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

#[aster_forge_api_docs_macros::path(get,path="/api/v1/achievements",operation_id="achievements_list",params(("page"=Option<u64>,Query),("page_size"=Option<u64>,Query),("search"=Option<String>,Query),("type_code"=Option<String>,Query),("academic_term_id"=Option<i64>,Query),("status"=Option<String>,Query),("level_code"=Option<String>,Query),("teacher_id"=Option<i64>,Query),("department_id"=Option<i64>,Query),("sort"=Option<String>,Query),("scope"=Option<String>,Query)),security(("session"=[])),responses((status=200,body=Success<AchievementPage>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
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
        let is_all_scope = match query.scope.as_deref().unwrap_or("all") {
            "mine" => {
                select = select.filter(repo::personal_scope(&user));
                false
            }
            "all" => true,
            _ => return Err(ApiError::Invalid("成果范围无效")),
        };
        if let Some(search) = &query.search {
            if search.chars().count() > 100 {
                return Err(ApiError::Invalid("搜索内容过长"));
            }
            let escaped = search
                .replace('!', "!!")
                .replace('%', "!%")
                .replace('_', "!_");
            let pattern = format!("%{escaped}%");
            if is_all_scope {
                select =
                    select.filter(
                        Condition::any()
                            .add(
                                achievement::Column::Title
                                    .like(LikeExpr::new(pattern.clone()).escape('!')),
                            )
                            .add(
                                achievement::Column::TeacherId.in_subquery(
                                    Query::select()
                                        .column(teacher::Column::Id)
                                        .from(teacher::Entity)
                                        .cond_where(
                                            Condition::any()
                                                .add(Expr::col(teacher::Column::Name).like(
                                                    LikeExpr::new(pattern.clone()).escape('!'),
                                                ))
                                                .add(
                                                    Expr::col(teacher::Column::EmployeeNo)
                                                        .like(LikeExpr::new(pattern).escape('!')),
                                                ),
                                        )
                                        .to_owned(),
                                ),
                            ),
                    );
            } else {
                select = select
                    .filter(achievement::Column::Title.like(LikeExpr::new(pattern).escape('!')));
            }
        }
        if let Some(dept_id) = query.department_id {
            select = select.filter(
                achievement::Column::TeacherId.in_subquery(
                    Query::select()
                        .column(teacher::Column::Id)
                        .from(teacher::Entity)
                        .and_where(Expr::col(teacher::Column::DepartmentId).eq(dept_id))
                        .to_owned(),
                ),
            );
        }
        if let Some(tid) = query.teacher_id {
            select = select.filter(
                Condition::any()
                    .add(achievement::Column::TeacherId.eq(tid))
                    .add(
                        achievement::Column::Id.in_subquery(
                            Query::select()
                                .column(participant::Column::AchievementId)
                                .from(participant::Entity)
                                .and_where(Expr::col(participant::Column::TeacherId).eq(tid))
                                .to_owned(),
                        ),
                    ),
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
            "occurred_at_desc" => select.order_by_with_nulls(
                achievement::Column::OccurredAt,
                sea_orm::Order::Desc,
                sea_orm::sea_query::NullOrdering::Last,
            ),
            _ => return Err(ApiError::Invalid("排序方式无效")),
        };
        let pager = select
            .order_by_desc(achievement::Column::Id)
            .paginate(state.db_handles.writer(), page_size);
        let total = pager.num_items().await?;
        let page = std::cmp::Ord::min(page, std::cmp::Ord::max(total.div_ceil(page_size), 1));
        let rows = pager.fetch_page(page - 1).await?;
        let items = repo::items(state.db_handles.writer(), &rows).await?;
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
        let mut parsed = Vec::with_capacity(rows.len());
        let mut teacher_ids = std::collections::BTreeSet::new();
        for r in rows {
            let snapshot: AchievementSnapshot =
                serde_json::from_value(r.snapshot).map_err(|_| ApiError::Internal)?;
            for person in &snapshot.data.participants {
                if let Some(tid) = person.teacher_id {
                    teacher_ids.insert(tid);
                }
            }
            parsed.push((
                r.version,
                r.schema_version,
                r.modified_by,
                r.created_at,
                snapshot,
            ));
        }
        let names = repo::teacher_names(state.db_handles.writer(), teacher_ids).await?;
        let mut revisions = Vec::with_capacity(parsed.len());
        for (version, schema_version, modified_by, created_at, mut snapshot) in parsed {
            for person in &mut snapshot.data.participants {
                if person.teacher_name.is_none() {
                    person.teacher_name =
                        person.teacher_id.and_then(|tid| names.get(&tid).cloned());
                }
            }
            let mut files = Vec::new();
            for file_id in &snapshot.data.file_ids {
                if let Ok(model) =
                    repo::file_access(state.db_handles.writer(), &user, *file_id).await
                {
                    files.push(repo::metadata(model));
                }
            }
            snapshot.data.file_ids = files.iter().map(|f| f.id).collect();
            revisions.push(RevisionItem {
                version,
                schema_version,
                snapshot,
                files,
                modified_by,
                created_at,
            });
        }
        success(&request, AchievementRevisions(revisions))
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
pub fn configure(cfg: &mut web::ServiceConfig) {
    use crate::api::common::{get_resource, method_not_allowed, post_resource};

    cfg.service(
        web::resource("/achievements")
            .route(web::get().to(list))
            .route(web::head().to(list))
            .route(web::post().to(create))
            .default_service(method_not_allowed("GET, HEAD, POST")),
    )
    .service(
        web::resource("/achievements/{id}")
            .route(web::get().to(detail))
            .route(web::head().to(detail))
            .route(web::patch().to(update))
            .default_service(method_not_allowed("GET, HEAD, PATCH")),
    )
    .service(post_resource("/achievements/{id}/submit", submit))
    .service(get_resource("/achievements/{id}/revisions", revisions));
}
