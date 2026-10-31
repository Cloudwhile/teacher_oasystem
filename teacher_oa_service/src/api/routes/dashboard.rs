#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::api::error::{Failure, Success};
use crate::{
    api::error::{ApiError, RequestError, success},
    domain::achievements::AchievementItem,
    repositories::{achievement_entities::achievement, achievements as repo},
    runtime::AppState,
    services::auth_service as auth,
};
use actix_web::{HttpRequest, HttpResponse, web};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Serialize;
#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct Dashboard {
    pub counts: std::collections::BTreeMap<String, u64>,
    pub review_pending: u64,
    pub recent: Vec<AchievementItem>,
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/dashboard",operation_id="dashboard",security(("session"=[])),responses((status=200,body=Success<Dashboard>),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn dashboard(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        let db = state.db_handles.writer();
        let own = achievement::Entity::find()
            .filter(repo::scope(&user))
            .filter(repo::personal_scope(&user));
        let mut counts = std::collections::BTreeMap::new();
        for status in ["draft", "pending", "returned", "approved", "archived"] {
            counts.insert(
                status.into(),
                own.clone()
                    .filter(achievement::Column::Status.eq(status))
                    .count(db)
                    .await?,
            );
        }
        let review_pending = achievement::Entity::find()
            .filter(repo::review_scope(&user))
            .filter(achievement::Column::Status.eq("pending"))
            .count(db)
            .await?;
        let mut recent = Vec::new();
        for row in own
            .order_by_desc(achievement::Column::UpdatedAt)
            .order_by_desc(achievement::Column::Id)
            .limit(5)
            .all(db)
            .await?
        {
            recent.push(repo::item(db, &row).await?);
        }
        success(
            &request,
            Dashboard {
                counts,
                review_pending,
                recent,
            },
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
