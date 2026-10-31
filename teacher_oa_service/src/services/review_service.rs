use crate::{
    api::error::ApiError,
    domain::{
        achievements::{AchievementDetail, ReviewRequest},
        identity::CurrentUser,
    },
    repositories::{
        achievement_entities::{achievement, review_event},
        achievements as repo,
    },
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait, sea_query::Expr,
};

#[derive(Clone, Copy)]
pub enum ReviewAction {
    Approve,
    Return,
    Archive,
}

impl ReviewAction {
    fn transition(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Approve => ("pending", "approved", "approve"),
            Self::Return => ("pending", "returned", "return"),
            Self::Archive => ("approved", "archived", "archive"),
        }
    }
}

pub async fn review(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: i64,
    request: ReviewRequest,
    action: ReviewAction,
) -> Result<AchievementDetail, ApiError> {
    let txn = db.begin().await?;
    let old = achievement::Entity::find_by_id(id)
        .filter(repo::review_scope(user))
        .one(&txn)
        .await?
        .ok_or(ApiError::Forbidden)?;
    let (from, to, action_code) = action.transition();
    if old.status != from || old.version != request.version {
        return Err(ApiError::Conflict("记录状态或版本已变化，请重新加载"));
    }
    let opinion = request.opinion.trim();
    if matches!(action, ReviewAction::Return) && opinion.is_empty() {
        return Err(ApiError::Invalid("请填写退回意见"));
    }
    if opinion.chars().count() > 2000 {
        return Err(ApiError::Invalid("审核意见不能超过2000字"));
    }
    let next = old.version.checked_add(1).ok_or(ApiError::Internal)?;
    let now = Utc::now();
    let updated = achievement::Entity::update_many()
        .col_expr(achievement::Column::Status, Expr::value(to))
        .col_expr(achievement::Column::Version, Expr::value(next))
        .col_expr(achievement::Column::UpdatedAt, Expr::value(now))
        .filter(achievement::Column::Id.eq(id))
        .filter(achievement::Column::Version.eq(request.version))
        .filter(achievement::Column::Status.eq(from))
        .filter(repo::review_scope(user))
        .exec(&txn)
        .await?;
    if updated.rows_affected != 1 {
        return Err(ApiError::Conflict("记录状态或版本已变化，请重新加载"));
    }
    let record = achievement::Entity::find_by_id(id)
        .one(&txn)
        .await?
        .ok_or(ApiError::Internal)?;
    let input = repo::input(&txn, &record).await?;
    super::achievement_service::snapshot(&txn, user.id, &record, &input).await?;
    review_event::ActiveModel {
        achievement_id: Set(id),
        reviewer_id: Set(user.id),
        action: Set(action_code.into()),
        old_status: Set(from.into()),
        new_status: Set(to.into()),
        opinion: Set(opinion.into()),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    super::business_audit::write(
        &txn,
        user.id,
        &format!("achievement.{action_code}"),
        "achievement",
        Some(id),
    )
    .await?;
    let detail = repo::detail(&txn, user, &record).await?;
    txn.commit().await?;
    Ok(detail)
}
