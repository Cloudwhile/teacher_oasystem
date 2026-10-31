use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub enum FieldType {
    Text,
    Textarea,
    Number,
    Date,
    Select,
    MultiSelect,
    ParticipantList,
    FileList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct FieldDefinition {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: FieldType,
    pub required: bool,
    pub visible: bool,
    pub order: u32,
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dictionary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub help: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AchievementSchema {
    pub code: String,
    pub name: String,
    pub category: String,
    pub version: i32,
    pub fields: Vec<FieldDefinition>,
    pub json_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AchievementType {
    pub code: String,
    pub name: String,
    pub category: String,
    pub current_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ParticipantInput {
    pub teacher_id: Option<i64>,
    pub external_name: String,
    pub role_code: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AchievementInput {
    pub type_code: String,
    pub schema_version: i32,
    pub title: String,
    pub subject_code: Option<String>,
    pub academic_term_id: Option<i64>,
    pub occurred_at: Option<NaiveDate>,
    pub level_code: Option<String>,
    pub award_grade_code: Option<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
    #[serde(default)]
    pub remarks: String,
    #[serde(default)]
    pub participants: Vec<ParticipantInput>,
    #[serde(default)]
    pub file_ids: Vec<i64>,
    #[serde(default)]
    pub term_mismatch_confirmed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct UpdateAchievement {
    pub version: i32,
    pub data: AchievementInput,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct VersionRequest {
    pub version: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ReviewRequest {
    pub version: i32,
    #[serde(default)]
    pub opinion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct DictionaryItem {
    pub group_code: String,
    pub code: String,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AcademicTerm {
    pub id: i64,
    pub academic_year: String,
    pub term_number: i32,
    pub name: String,
    pub starts_on: NaiveDate,
    pub ends_on: NaiveDate,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct FileMetadata {
    pub id: i64,
    pub filename: String,
    pub size_bytes: i64,
    pub mime_type: String,
    pub sha256: String,
    pub uploaded_by: i64,
    pub upload_status: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub association: Option<FileAssociation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub enum FileAssociation {
    Linked,
    Historical,
    Unlinked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AchievementItem {
    pub id: i64,
    pub teacher_id: i64,
    pub teacher_name: String,
    pub title: String,
    pub type_code: String,
    pub type_name: String,
    pub subject_code: Option<String>,
    pub academic_term_id: Option<i64>,
    pub occurred_at: Option<NaiveDate>,
    pub level_code: Option<String>,
    pub award_grade_code: Option<String>,
    pub status: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ReviewEvent {
    pub id: i64,
    pub reviewer_id: i64,
    pub reviewer_name: String,
    pub action: String,
    pub old_status: String,
    pub new_status: String,
    pub opinion: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AchievementDetail {
    pub record: AchievementItem,
    pub data: AchievementInput,
    pub schema: AchievementSchema,
    pub files: Vec<FileMetadata>,
    pub events: Vec<ReviewEvent>,
    pub can_edit: bool,
    pub can_review: bool,
    pub can_archive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct RevisionItem {
    pub files: Vec<FileMetadata>,
    pub version: i32,
    pub schema_version: i32,
    pub snapshot: AchievementSnapshot,
    pub modified_by: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AchievementSnapshot {
    pub status: String,
    pub data: AchievementInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AchievementPage {
    pub items: Vec<AchievementItem>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct FilePage {
    pub items: Vec<FileMetadata>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct TeacherOption {
    pub id: i64,
    pub name: String,
    pub department_id: i64,
}
