use crate::api::error::ApiError;
use sea_orm::ConnectionTrait;
pub async fn write(
    db: &impl ConnectionTrait,
    user_id: i64,
    action: &str,
    entity_type: &str,
    id: Option<i64>,
) -> Result<(), ApiError> {
    aster_forge_db::audit_log::create_audit_log_row(
        db,
        aster_forge_db::audit_log::AuditLogCreate {
            user_id,
            action: action.into(),
            entity_type: entity_type.into(),
            entity_id: id,
            entity_name: None,
            details: None,
            ip_address: None,
            user_agent: None,
            created_at: chrono::Utc::now(),
        },
    )
    .await
    .map_err(|e| {
        tracing::error!(error=%e,"business audit failed");
        ApiError::Internal
    })?;
    Ok(())
}
