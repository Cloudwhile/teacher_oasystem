use super::achievements::{AchievementInput, AchievementSchema, FieldType};
use crate::api::error::ApiError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

pub const COMMON_KEYS: [&str; 9] = [
    "title",
    "subject_code",
    "academic_term_id",
    "occurred_at",
    "level_code",
    "award_grade_code",
    "remarks",
    "participants",
    "files",
];

pub fn values(input: &AchievementInput) -> BTreeMap<String, Value> {
    let mut map = input.attributes.clone();
    map.insert("title".into(), json!(input.title));
    map.insert("remarks".into(), json!(input.remarks));
    for (key, value) in [
        ("subject_code", json!(input.subject_code)),
        ("academic_term_id", json!(input.academic_term_id)),
        ("occurred_at", json!(input.occurred_at)),
        ("level_code", json!(input.level_code)),
        ("award_grade_code", json!(input.award_grade_code)),
    ] {
        if !value.is_null() {
            map.insert(key.into(), value);
        }
    }
    map.insert("participants".into(), json!(input.participants));
    map.insert("files".into(), json!(input.file_ids));
    map
}
pub fn error_key(key: &str) -> String {
    if key == "files" {
        "file_ids".into()
    } else if COMMON_KEYS.contains(&key) {
        key.into()
    } else {
        format!("attributes.{key}")
    }
}

pub fn validate(
    input: &AchievementInput,
    schema: &AchievementSchema,
    submitting: bool,
) -> Result<Option<Decimal>, ApiError> {
    let mut errors = BTreeMap::new();
    if input.type_code != schema.code || input.schema_version != schema.version {
        return Err(ApiError::Conflict("成果类型或字段版本已变化，请重新加载"));
    }
    let map = values(input);
    let defined: BTreeSet<&str> = schema
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect();
    for key in input.attributes.keys() {
        if COMMON_KEYS.contains(&key.as_str()) || !defined.contains(key.as_str()) {
            errors.insert(error_key(key), "字段不属于当前成果类型".into());
        }
    }
    for key in [
        "subject_code",
        "academic_term_id",
        "occurred_at",
        "level_code",
        "award_grade_code",
    ] {
        if map.contains_key(key) && !defined.contains(key) {
            errors.insert(key.into(), "字段不属于当前成果类型".into());
        }
    }
    let mut funding = None;
    for field in &schema.fields {
        let value = map.get(&field.key);
        let empty = value.is_none_or(|v| {
            v.is_null()
                || v.as_str().is_some_and(|v| v.trim().is_empty())
                || (matches!(
                    field.kind,
                    FieldType::MultiSelect | FieldType::ParticipantList | FieldType::FileList
                ) && v.as_array().is_some_and(Vec::is_empty))
        });
        if field.required && (submitting || field.key == "title") && empty {
            errors.insert(error_key(&field.key), format!("请填写{}", field.label));
            continue;
        }
        if empty
            && value.is_none_or(|v| {
                v.is_null() || !matches!(field.kind, FieldType::Text | FieldType::Textarea)
            })
        {
            continue;
        }
        let Some(value) = value else { continue };
        if !field.visible {
            errors.insert(error_key(&field.key), "该字段当前不可填写".into());
            continue;
        }
        let valid = match field.kind {
            FieldType::Text | FieldType::Textarea => value.as_str().is_some_and(|s| {
                s.chars().count() <= field.max_length.unwrap_or(2000) && !s.contains('\0')
            }),
            FieldType::Date => value
                .as_str()
                .is_some_and(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()),
            FieldType::Select if field.key == "academic_term_id" => {
                value.as_i64().is_some_and(|id| id > 0)
            }
            FieldType::Select => value.as_str().is_some_and(|s| {
                s.len() <= 64 && (field.options.is_empty() || field.options.iter().any(|v| v == s))
            }),
            FieldType::MultiSelect => value.as_array().is_some_and(|items| {
                items.len() <= 50
                    && items.iter().all(|v| {
                        v.as_str().is_some_and(|s| {
                            s.len() <= 64
                                && (field.options.is_empty()
                                    || field.options.iter().any(|v| v == s))
                        })
                    })
                    && items
                        .iter()
                        .map(Value::to_string)
                        .collect::<BTreeSet<_>>()
                        .len()
                        == items.len()
            }),
            FieldType::Number if field.format.as_deref() == Some("decimal") => {
                funding = value.as_str().and_then(|s| {
                    let valid = s.split('.').count() <= 2
                        && s.split_once('.').is_none_or(|(_, fraction)| {
                            !fraction.is_empty() && fraction.len() <= 2
                        })
                        && s.split('.')
                            .next()
                            .is_some_and(|v| !v.is_empty() && v.len() <= 14)
                        && s.bytes().all(|b| b.is_ascii_digit() || b == b'.');
                    if !valid {
                        return None;
                    }
                    Decimal::from_str(s)
                        .ok()
                        .filter(|v| v.scale() <= 2 && *v >= Decimal::ZERO)
                });
                funding.is_some()
            }
            FieldType::Number => value.as_u64().is_some_and(|v| v <= 1_000_000),
            FieldType::ParticipantList => input.participants.len() <= 50,
            FieldType::FileList => input.file_ids.len() <= 20,
        };
        if !valid {
            errors.insert(
                error_key(&field.key),
                format!("{}格式不正确或超出允许长度", field.label),
            );
        }
    }
    let participant_field = schema
        .fields
        .iter()
        .find(|f| f.kind == FieldType::ParticipantList);
    if !input.participants.is_empty() && participant_field.is_none() {
        errors.insert("participants".into(), "当前类型不支持相关人员".into());
    }
    let mut order = BTreeSet::new();
    let mut people = BTreeSet::new();
    let mut teachers = BTreeSet::new();
    for (index, person) in input.participants.iter().enumerate() {
        if person.teacher_id.is_none()
            && (person.external_name.trim().is_empty()
                || person.external_name.chars().count() > 100
                || person.external_name.contains('\0'))
        {
            errors.insert(
                format!("participants.{index}.external_name"),
                "请填写有效姓名，最多100字".into(),
            );
        }
        if usize::try_from(person.sort_order).ok() != Some(index)
            || !order.insert(person.sort_order)
        {
            errors.insert(
                format!("participants.{index}.sort_order"),
                "署名顺序必须从0开始连续编号".into(),
            );
        }
        if !people.insert((
            person.teacher_id,
            person.external_name.trim(),
            person.role_code.as_str(),
        )) {
            errors.insert(format!("participants.{index}"), "相关人员重复".into());
        }
        if let Some(id) = person.teacher_id
            && !teachers.insert(id)
        {
            errors.insert(
                format!("participants.{index}.teacher_id"),
                "同一教师不能重复关联".into(),
            );
        }
        if participant_field.is_some_and(|f| !f.roles.contains(&person.role_code)) {
            errors.insert(
                format!("participants.{index}.role_code"),
                "人员角色不适用于当前类型".into(),
            );
        }
        if person.teacher_id.is_some_and(|id| id <= 0) {
            errors.insert(
                format!("participants.{index}.teacher_id"),
                "关联教师无效".into(),
            );
        }
    }
    if input.file_ids.iter().any(|id| *id <= 0)
        || input.file_ids.iter().collect::<BTreeSet<_>>().len() != input.file_ids.len()
    {
        errors.insert("file_ids".into(), "材料关联重复或无效".into());
    }
    for key in ["completed_at", "ended_at", "valid_until"] {
        if let (Some(start), Some(end)) = (
            input.occurred_at,
            input
                .attributes
                .get(key)
                .and_then(Value::as_str)
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
        ) && end < start
        {
            errors.insert(error_key(key), "结束日期不能早于开始日期".into());
        }
    }
    if errors.is_empty() {
        Ok(funding)
    } else {
        Err(ApiError::Validation(errors))
    }
}
