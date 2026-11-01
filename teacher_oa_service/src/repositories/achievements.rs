use super::{
    achievement_entities::{
        achievement, achievement_file, achievement_type, dictionary, file, participant,
        review_event, schema_version, term,
    },
    entities::teacher,
};
use crate::{
    api::error::ApiError,
    domain::{
        achievements::*,
        identity::{CurrentUser, DataScope},
    },
};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait,
    sea_query::{Expr, Query},
};

pub fn has_permission(user: &CurrentUser, permission: &str) -> bool {
    user.grants.iter().any(|g| g.permission == permission)
}

pub fn scope(user: &CurrentUser) -> Condition {
    scope_for(user, "achievements:read")
}

pub fn personal_scope(user: &CurrentUser) -> Condition {
    let mut personal = user.clone();
    personal.grants = vec![crate::domain::identity::PermissionGrant {
        permission: "achievements:read".into(),
        scope: DataScope::SelfOnly,
    }];
    scope(&personal)
}

/// Drafts stay private to the owner and participants; wider scopes only see submitted records.
pub fn scope_for(user: &CurrentUser, permission: &str) -> Condition {
    let mut condition = Condition::any();
    for grant in &user.grants {
        if grant.permission != permission {
            continue;
        }
        match grant.scope {
            DataScope::All => {
                condition = condition.add(achievement::Column::Status.ne("draft"));
            }
            DataScope::SelfOnly => {
                condition = condition
                    .add(achievement::Column::TeacherId.eq(user.teacher.id))
                    .add(
                        achievement::Column::Id.in_subquery(
                            Query::select()
                                .column(participant::Column::AchievementId)
                                .from(participant::Entity)
                                .and_where(
                                    Expr::col(participant::Column::TeacherId).eq(user.teacher.id),
                                )
                                .to_owned(),
                        ),
                    );
            }
            DataScope::Department => {
                condition = condition.add(
                    Condition::all()
                        .add(achievement::Column::Status.ne("draft"))
                        .add(
                            achievement::Column::TeacherId.in_subquery(
                                Query::select()
                                    .column(teacher::Column::Id)
                                    .from(teacher::Entity)
                                    .and_where(
                                        Expr::col(teacher::Column::DepartmentId)
                                            .eq(user.teacher.department_id),
                                    )
                                    .to_owned(),
                            ),
                        ),
                );
            }
        }
    }
    if !has_permission(user, permission) {
        condition = condition.add(Expr::cust("1=0"));
    }
    condition
}
pub async fn accessible(
    db: &impl ConnectionTrait,
    user: &CurrentUser,
    id: i64,
) -> Result<achievement::Model, ApiError> {
    achievement::Entity::find_by_id(id)
        .filter(scope(user))
        .one(db)
        .await?
        .ok_or(ApiError::Forbidden)
}
pub fn editable(user: &CurrentUser, record: &achievement::Model) -> bool {
    record.teacher_id == user.teacher.id
        && has_permission(user, "achievements:write")
        && matches!(record.status.as_str(), "draft" | "returned")
}
pub async fn schema(
    db: &impl ConnectionTrait,
    code: &str,
    version: i32,
) -> Result<AchievementSchema, ApiError> {
    let row = schema_version::Entity::find_by_id((code.to_string(), version))
        .one(db)
        .await?
        .ok_or(ApiError::NotFound)?;
    serde_json::from_value(row.schema_document).map_err(|_| ApiError::Internal)
}
pub async fn current_schema(
    db: &impl ConnectionTrait,
    code: &str,
) -> Result<AchievementSchema, ApiError> {
    let kind = achievement_type::Entity::find_by_id(code)
        .filter(achievement_type::Column::Published.eq(true))
        .one(db)
        .await?
        .ok_or(ApiError::NotFound)?;
    schema(db, code, kind.current_version).await
}
pub async fn types(db: &impl ConnectionTrait) -> Result<Vec<AchievementType>, ApiError> {
    Ok(achievement_type::Entity::find()
        .filter(achievement_type::Column::Published.eq(true))
        .order_by_asc(achievement_type::Column::Code)
        .all(db)
        .await?
        .into_iter()
        .map(|r| AchievementType {
            code: r.code,
            name: r.name,
            category: r.category,
            current_version: r.current_version,
        })
        .collect())
}
pub async fn terms(db: &impl ConnectionTrait) -> Result<Vec<AcademicTerm>, ApiError> {
    Ok(term::Entity::find()
        .order_by_desc(term::Column::StartsOn)
        .all(db)
        .await?
        .into_iter()
        .map(|r| AcademicTerm {
            id: r.id,
            academic_year: r.academic_year,
            term_number: r.term_number,
            name: r.name,
            starts_on: r.starts_on,
            ends_on: r.ends_on,
            enabled: r.enabled,
        })
        .collect())
}
pub async fn dictionaries(db: &impl ConnectionTrait) -> Result<Vec<DictionaryItem>, ApiError> {
    Ok(dictionary::Entity::find()
        .filter(dictionary::Column::Enabled.eq(true))
        .order_by_asc(dictionary::Column::GroupCode)
        .order_by_asc(dictionary::Column::SortOrder)
        .all(db)
        .await?
        .into_iter()
        .map(|r| DictionaryItem {
            group_code: r.group_code,
            code: r.code,
            name: r.name,
            sort_order: r.sort_order,
        })
        .collect())
}
pub fn metadata(r: file::Model) -> FileMetadata {
    FileMetadata {
        id: r.id,
        filename: r.filename,
        size_bytes: r.size_bytes,
        mime_type: r.mime_type,
        sha256: r.sha256,
        uploaded_by: r.uploaded_by,
        upload_status: r.upload_status,
        created_at: r.created_at,
        association: None,
    }
}

pub async fn file_association(
    db: &impl ConnectionTrait,
    user: &CurrentUser,
    id: i64,
) -> Result<FileAssociation, ApiError> {
    let visible = achievement::Entity::find()
        .select_only()
        .column(achievement::Column::Id)
        .filter(scope(user))
        .filter(scope_for(user, "files:read"))
        .into_query();
    if achievement_file::Entity::find()
        .filter(achievement_file::Column::FileId.eq(id))
        .filter(achievement_file::Column::AchievementId.in_subquery(visible.clone()))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(FileAssociation::Linked);
    }
    use super::achievement_entities::revision_file;
    if revision_file::Entity::find()
        .filter(revision_file::Column::FileId.eq(id))
        .filter(revision_file::Column::AchievementId.in_subquery(visible))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(FileAssociation::Historical);
    }
    Ok(FileAssociation::Unlinked)
}

pub async fn item(
    db: &impl ConnectionTrait,
    r: &achievement::Model,
) -> Result<AchievementItem, ApiError> {
    let person = teacher::Entity::find_by_id(r.teacher_id)
        .one(db)
        .await?
        .ok_or(ApiError::Internal)?;
    let kind = achievement_type::Entity::find_by_id(&r.type_code)
        .one(db)
        .await?
        .ok_or(ApiError::Internal)?;
    Ok(AchievementItem {
        id: r.id,
        teacher_id: r.teacher_id,
        teacher_name: person.name,
        title: r.title.clone(),
        type_code: r.type_code.clone(),
        type_name: kind.name,
        subject_code: r.subject_code.clone(),
        academic_term_id: r.academic_term_id,
        occurred_at: r.occurred_at,
        level_code: r.level_code.clone(),
        award_grade_code: r.award_grade_code.clone(),
        status: r.status.clone(),
        version: r.version,
        created_at: r.created_at,
        updated_at: r.updated_at,
    })
}
pub async fn input(
    db: &impl ConnectionTrait,
    r: &achievement::Model,
) -> Result<AchievementInput, ApiError> {
    let participants = participant::Entity::find()
        .filter(participant::Column::AchievementId.eq(r.id))
        .order_by_asc(participant::Column::SortOrder)
        .all(db)
        .await?
        .into_iter()
        .map(|p| ParticipantInput {
            teacher_id: p.teacher_id,
            external_name: p.external_name,
            role_code: p.role_code,
            sort_order: p.sort_order,
        })
        .collect();
    let file_ids = achievement_file::Entity::find()
        .filter(achievement_file::Column::AchievementId.eq(r.id))
        .order_by_asc(achievement_file::Column::SortOrder)
        .all(db)
        .await?
        .into_iter()
        .map(|r| r.file_id)
        .collect();
    Ok(AchievementInput {
        type_code: r.type_code.clone(),
        schema_version: r.schema_version,
        title: r.title.clone(),
        subject_code: r.subject_code.clone(),
        academic_term_id: r.academic_term_id,
        occurred_at: r.occurred_at,
        level_code: r.level_code.clone(),
        award_grade_code: r.award_grade_code.clone(),
        attributes: serde_json::from_value(r.attributes.clone()).map_err(|_| ApiError::Internal)?,
        remarks: r.remarks.clone(),
        participants,
        file_ids,
        term_mismatch_confirmed: r.term_mismatch_confirmed,
    })
}
pub async fn detail(
    db: &impl ConnectionTrait,
    user: &CurrentUser,
    r: &achievement::Model,
) -> Result<AchievementDetail, ApiError> {
    let data = input(db, r).await?;
    let mut files = Vec::new();
    for id in &data.file_ids {
        files.push(metadata(
            file::Entity::find_by_id(*id)
                .one(db)
                .await?
                .ok_or(ApiError::Internal)?,
        ));
    }
    let rows = review_event::Entity::find()
        .filter(review_event::Column::AchievementId.eq(r.id))
        .order_by_asc(review_event::Column::CreatedAt)
        .all(db)
        .await?;
    let mut events = Vec::new();
    for e in rows {
        let reviewer = super::entities::user::Entity::find_by_id(e.reviewer_id)
            .one(db)
            .await?;
        events.push(ReviewEvent {
            id: e.id,
            reviewer_id: e.reviewer_id,
            reviewer_name: reviewer
                .map(|u| u.display_name)
                .unwrap_or_else(|| "原办理人员".into()),
            action: e.action,
            old_status: e.old_status,
            new_status: e.new_status,
            opinion: e.opinion,
            created_at: e.created_at,
        });
    }
    Ok(AchievementDetail {
        record: item(db, r).await?,
        schema: schema(db, &r.type_code, r.schema_version).await?,
        data,
        files,
        events,
        can_edit: editable(user, r),
        can_review: r.status == "pending" && reviewable(db, user, r.id).await?,
        can_archive: r.status == "approved" && reviewable(db, user, r.id).await?,
    })
}
pub fn review_scope(user: &CurrentUser) -> Condition {
    Condition::all()
        .add(scope(user))
        .add(scope_for(user, "reviews:write"))
        .add(achievement::Column::TeacherId.ne(user.teacher.id))
        .add(achievement::Column::CreatedBy.ne(user.id))
}

pub async fn reviewable(
    db: &impl ConnectionTrait,
    user: &CurrentUser,
    id: i64,
) -> Result<bool, ApiError> {
    Ok(achievement::Entity::find_by_id(id)
        .filter(review_scope(user))
        .one(db)
        .await?
        .is_some())
}
pub fn file_scope(user: &CurrentUser) -> Condition {
    let ids = achievement::Entity::find()
        .select_only()
        .column(achievement::Column::Id)
        .filter(scope(user))
        .filter(scope_for(user, "files:read"))
        .into_query();
    let linked = Query::select()
        .column(achievement_file::Column::FileId)
        .from(achievement_file::Entity)
        .and_where(Expr::col(achievement_file::Column::AchievementId).in_subquery(ids.clone()))
        .to_owned();
    let historical = Query::select()
        .column(super::achievement_entities::revision_file::Column::FileId)
        .from(super::achievement_entities::revision_file::Entity)
        .and_where(
            Expr::col(super::achievement_entities::revision_file::Column::AchievementId)
                .in_subquery(ids),
        )
        .to_owned();
    Condition::any()
        .add(file::Column::UploadedBy.eq(user.id))
        .add(file::Column::Id.in_subquery(linked))
        .add(file::Column::Id.in_subquery(historical))
}
pub async fn file_access(
    db: &impl ConnectionTrait,
    user: &CurrentUser,
    id: i64,
) -> Result<file::Model, ApiError> {
    if !has_permission(user, "files:read") {
        return Err(ApiError::Forbidden);
    }
    file::Entity::find_by_id(id)
        .filter(file_scope(user))
        .filter(file::Column::UploadStatus.eq("ready"))
        .one(db)
        .await?
        .ok_or(ApiError::Forbidden)
}
