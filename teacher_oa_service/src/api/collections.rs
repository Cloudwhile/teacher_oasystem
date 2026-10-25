//! Named payloads keep generic collection schemas distinct in OpenAPI.
use crate::domain::{achievements::*, identity::PermissionGrant};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AchievementTypes(pub Vec<AchievementType>);
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AcademicTerms(pub Vec<AcademicTerm>);
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct DictionaryItems(pub Vec<DictionaryItem>);
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct TeacherOptions(pub Vec<TeacherOption>);
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AchievementRevisions(pub Vec<RevisionItem>);
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PermissionGrants(pub Vec<PermissionGrant>);
