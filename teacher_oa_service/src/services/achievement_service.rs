use super::business_audit;
use crate::{
    api::error::ApiError,
    domain::{achievement_validation, achievements::*, identity::CurrentUser},
    repositories::{
        achievement_entities::{
            achievement, achievement_file, dictionary, file, participant, review_event, revision,
            term,
        },
        achievements as repo,
    },
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set, TransactionTrait, sea_query::Expr,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub async fn validate_relations(
    db: &impl ConnectionTrait,
    user: &CurrentUser,
    input: &mut AchievementInput,
    schema: &AchievementSchema,
    submitting: bool,
) -> Result<Option<rust_decimal::Decimal>, ApiError> {
    let funding = achievement_validation::validate(input, schema, submitting)?;
    let mut errors = BTreeMap::new();
    let values = achievement_validation::values(input);
    for field in &schema.fields {
        if matches!(field.kind, FieldType::Select | FieldType::MultiSelect)
            && field.key != "academic_term_id"
            && let (Some(group), Some(value)) = (&field.dictionary, values.get(&field.key))
        {
            let selections: Vec<&str> = if field.kind == FieldType::MultiSelect {
                value
                    .as_array()
                    .map(|v| v.iter().filter_map(Value::as_str).collect())
                    .unwrap_or_default()
            } else {
                value
                    .as_str()
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            for code in selections {
                if dictionary::Entity::find_by_id((group.clone(), code.to_string()))
                    .filter(dictionary::Column::Enabled.eq(true))
                    .one(db)
                    .await?
                    .is_none()
                {
                    errors.insert(
                        achievement_validation::error_key(&field.key),
                        "请选择有效字典项".into(),
                    );
                }
            }
        }
    }
    if let Some(term_id) = input.academic_term_id {
        match term::Entity::find_by_id(term_id).one(db).await? {
            None => {
                errors.insert("academic_term_id".into(), "学期不存在".into());
            }
            Some(term) => {
                if submitting && !term.enabled {
                    errors.insert("academic_term_id".into(), "所选学期已停用".into());
                }
                if submitting
                    && input
                        .occurred_at
                        .is_some_and(|date| date < term.starts_on || date > term.ends_on)
                    && !input.term_mismatch_confirmed
                {
                    errors.insert(
                        "academic_term_id".into(),
                        "日期不在所选学期，请核对并确认".into(),
                    );
                }
            }
        }
    }
    for (index, person) in input.participants.iter_mut().enumerate() {
        if let Some(id) = person.teacher_id {
            let profile = crate::repositories::identity::teacher_profile(db, id)
                .await
                .map_err(|error| match error {
                    ApiError::NotFound => ApiError::Invalid("关联教师不存在"),
                    other => other,
                })?;
            if !user.can_read_teacher(&profile) {
                return Err(ApiError::Forbidden);
            }
            if profile.employment_status != "active" {
                errors.insert(
                    format!("participants.{index}.teacher_id"),
                    "关联教师已离职".into(),
                );
            }
            person.external_name = profile.name;
        } else {
            person.external_name = person.external_name.trim().to_string();
        }
    }
    for id in &input.file_ids {
        let own_file = file::Entity::find_by_id(*id)
            .filter(file::Column::UploadedBy.eq(user.id))
            .filter(file::Column::UploadStatus.eq("ready"))
            .one(db)
            .await?;
        if own_file.is_none() {
            return Err(ApiError::Forbidden);
        }
    }
    if errors.is_empty() {
        Ok(funding)
    } else {
        Err(ApiError::Validation(errors))
    }
}

async fn replace_links(
    db: &impl ConnectionTrait,
    id: i64,
    input: &AchievementInput,
) -> Result<(), ApiError> {
    participant::Entity::delete_many()
        .filter(participant::Column::AchievementId.eq(id))
        .exec(db)
        .await?;
    achievement_file::Entity::delete_many()
        .filter(achievement_file::Column::AchievementId.eq(id))
        .exec(db)
        .await?;
    for person in &input.participants {
        participant::ActiveModel {
            achievement_id: Set(id),
            teacher_id: Set(person.teacher_id),
            external_name: Set(person.external_name.clone()),
            role_code: Set(person.role_code.clone()),
            sort_order: Set(person.sort_order),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }
    for (order, file_id) in input.file_ids.iter().enumerate() {
        achievement_file::ActiveModel {
            achievement_id: Set(id),
            file_id: Set(*file_id),
            purpose: Set("evidence".into()),
            sort_order: Set(i32::try_from(order).map_err(|_| ApiError::Internal)?),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}
pub(super) async fn snapshot(
    db: &impl ConnectionTrait,
    actor: i64,
    record: &achievement::Model,
    input: &AchievementInput,
) -> Result<(), ApiError> {
    let revision = revision::ActiveModel {
        achievement_id: Set(record.id),
        version: Set(record.version),
        schema_version: Set(record.schema_version),
        snapshot: Set(json!({"status":record.status,"data":input})),
        modified_by: Set(actor),
        created_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    for file_id in &input.file_ids {
        crate::repositories::achievement_entities::revision_file::ActiveModel {
            revision_id: Set(revision.id),
            achievement_id: Set(record.id),
            file_id: Set(*file_id),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

pub async fn create(
    db: &DatabaseConnection,
    user: &CurrentUser,
    mut input: AchievementInput,
) -> Result<AchievementDetail, ApiError> {
    if !repo::has_permission(user, "achievements:write") {
        return Err(ApiError::Forbidden);
    }
    let txn = db.begin().await?;
    let schema = repo::current_schema(&txn, &input.type_code).await?;
    let funding = validate_relations(&txn, user, &mut input, &schema, false).await?;
    input.title = input.title.trim().to_string();
    let record = achievement::ActiveModel {
        teacher_id: Set(user.teacher.id),
        type_code: Set(input.type_code.clone()),
        schema_version: Set(input.schema_version),
        title: Set(input.title.clone()),
        subject_code: Set(input.subject_code.clone()),
        academic_term_id: Set(input.academic_term_id),
        occurred_at: Set(input.occurred_at),
        level_code: Set(input.level_code.clone()),
        award_grade_code: Set(input.award_grade_code.clone()),
        funding_amount: Set(funding),
        attributes: Set(json!(input.attributes)),
        remarks: Set(input.remarks.clone()),
        status: Set("draft".into()),
        version: Set(1),
        created_by: Set(user.id),
        term_mismatch_confirmed: Set(input.term_mismatch_confirmed),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    replace_links(&txn, record.id, &input).await?;
    snapshot(&txn, user.id, &record, &input).await?;
    business_audit::write(
        &txn,
        user.id,
        "achievement.create",
        "achievement",
        Some(record.id),
    )
    .await?;
    let detail = repo::detail(&txn, user, &record).await?;
    txn.commit().await?;
    Ok(detail)
}

pub async fn update(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: i64,
    request: UpdateAchievement,
) -> Result<AchievementDetail, ApiError> {
    let txn = db.begin().await?;
    let old = repo::accessible(&txn, user, id).await?;
    if old.teacher_id != user.teacher.id || !repo::has_permission(user, "achievements:write") {
        return Err(ApiError::Forbidden);
    }
    if !repo::editable(user, &old) {
        return Err(ApiError::Conflict("当前状态不能修改"));
    }
    if request.version != old.version {
        return Err(ApiError::Conflict("记录已更新，请重新加载后再保存"));
    }
    let mut input = request.data;
    let schema = repo::schema(&txn, &old.type_code, old.schema_version).await?;
    let funding = validate_relations(&txn, user, &mut input, &schema, false).await?;
    let next = old.version.checked_add(1).ok_or(ApiError::Internal)?;
    input.title = input.title.trim().to_string();
    let result = achievement::Entity::update_many()
        .col_expr(achievement::Column::Title, Expr::value(input.title.trim()))
        .col_expr(
            achievement::Column::SubjectCode,
            Expr::value(input.subject_code.clone()),
        )
        .col_expr(
            achievement::Column::AcademicTermId,
            Expr::value(input.academic_term_id),
        )
        .col_expr(
            achievement::Column::OccurredAt,
            Expr::value(input.occurred_at),
        )
        .col_expr(
            achievement::Column::LevelCode,
            Expr::value(input.level_code.clone()),
        )
        .col_expr(
            achievement::Column::AwardGradeCode,
            Expr::value(input.award_grade_code.clone()),
        )
        .col_expr(achievement::Column::FundingAmount, Expr::value(funding))
        .col_expr(
            achievement::Column::Attributes,
            Expr::value(json!(input.attributes)),
        )
        .col_expr(
            achievement::Column::Remarks,
            Expr::value(input.remarks.clone()),
        )
        .col_expr(
            achievement::Column::TermMismatchConfirmed,
            Expr::value(input.term_mismatch_confirmed),
        )
        .col_expr(achievement::Column::Version, Expr::value(next))
        .col_expr(achievement::Column::UpdatedAt, Expr::value(Utc::now()))
        .filter(achievement::Column::Id.eq(id))
        .filter(achievement::Column::Version.eq(request.version))
        .filter(achievement::Column::Status.is_in(["draft", "returned"]))
        .exec(&txn)
        .await?;
    if result.rows_affected != 1 {
        return Err(ApiError::Conflict("记录已更新，请重新加载后再保存"));
    }
    replace_links(&txn, id, &input).await?;
    let record = achievement::Entity::find_by_id(id)
        .one(&txn)
        .await?
        .ok_or(ApiError::Internal)?;
    snapshot(&txn, user.id, &record, &input).await?;
    business_audit::write(&txn, user.id, "achievement.update", "achievement", Some(id)).await?;
    let detail = repo::detail(&txn, user, &record).await?;
    txn.commit().await?;
    Ok(detail)
}

pub async fn submit(
    db: &DatabaseConnection,
    user: &CurrentUser,
    id: i64,
    version: i32,
) -> Result<AchievementDetail, ApiError> {
    let txn = db.begin().await?;
    let old = repo::accessible(&txn, user, id).await?;
    if old.teacher_id != user.teacher.id || !repo::has_permission(user, "achievements:write") {
        return Err(ApiError::Forbidden);
    }
    if !repo::editable(user, &old) || old.version != version {
        return Err(ApiError::Conflict("记录状态或版本已变化，请重新加载"));
    }
    let schema = repo::schema(&txn, &old.type_code, old.schema_version).await?;
    let mut input = repo::input(&txn, &old).await?;
    validate_relations(&txn, user, &mut input, &schema, true).await?;
    let next = version.checked_add(1).ok_or(ApiError::Internal)?;
    let update = achievement::Entity::update_many()
        .col_expr(achievement::Column::Status, Expr::value("pending"))
        .col_expr(achievement::Column::Version, Expr::value(next))
        .col_expr(achievement::Column::UpdatedAt, Expr::value(Utc::now()))
        .filter(achievement::Column::Id.eq(id))
        .filter(achievement::Column::Version.eq(version))
        .filter(achievement::Column::Status.is_in(["draft", "returned"]))
        .exec(&txn)
        .await?;
    if update.rows_affected != 1 {
        return Err(ApiError::Conflict("记录状态或版本已变化，请重新加载"));
    }
    let record = achievement::Entity::find_by_id(id)
        .one(&txn)
        .await?
        .ok_or(ApiError::Internal)?;
    snapshot(&txn, user.id, &record, &input).await?;
    review_event::ActiveModel {
        achievement_id: Set(id),
        reviewer_id: Set(user.id),
        action: Set("submit".into()),
        old_status: Set(old.status),
        new_status: Set("pending".into()),
        opinion: Set(String::new()),
        created_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    business_audit::write(&txn, user.id, "achievement.submit", "achievement", Some(id)).await?;
    let detail = repo::detail(&txn, user, &record).await?;
    txn.commit().await?;
    Ok(detail)
}
