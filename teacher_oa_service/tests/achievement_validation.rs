use serde_json::json;
use teacher_oa_service::{
    api::error::ApiError,
    domain::{
        achievement_validation::validate,
        achievements::{AchievementInput, AchievementSchema},
    },
};

fn schema(code: &str) -> AchievementSchema {
    serde_json::from_value(
        teacher_oa_service_migration::catalog::schemas()
            .into_iter()
            .find(|v| v["code"] == code)
            .expect("schema"),
    )
    .expect("typed schema")
}
fn input(code: &str) -> AchievementInput {
    serde_json::from_value(
        json!({"type_code":code,"schema_version":1,"title":"字段校验","attributes":{}}),
    )
    .expect("input")
}

#[test]
fn empty_arrays_cannot_bypass_scalar_field_types_or_text_limits() {
    for (code, key) in [
        ("open_class", "venue"),
        ("research_project", "project_funding"),
        ("research_project", "completed_at"),
    ] {
        let mut data = input(code);
        data.attributes.insert(key.into(), json!([]));
        assert!(
            matches!(
                validate(&data, &schema(code), false),
                Err(ApiError::Validation(_))
            ),
            "{key}"
        );
    }
    let mut data = input("open_class");
    data.remarks = " ".repeat(2001);
    assert!(matches!(
        validate(&data, &schema("open_class"), false),
        Err(ApiError::Validation(_))
    ));
}

#[test]
fn participant_identity_and_contiguous_order_are_enforced() {
    let mut data = input("research_project");
    data.participants = serde_json::from_value(json!([
        {"teacher_id":91001,"external_name":"一个名称","role_code":"host","sort_order":0},
        {"teacher_id":91001,"external_name":"另一个名称","role_code":"participant","sort_order":1}
    ]))
    .expect("participants");
    assert!(matches!(
        validate(&data, &schema("research_project"), false),
        Err(ApiError::Validation(_))
    ));
    data.participants.truncate(1);
    data.participants[0].sort_order = 4;
    assert!(matches!(
        validate(&data, &schema("research_project"), false),
        Err(ApiError::Validation(_))
    ));
    data.participants[0].sort_order = 0;
    data.participants[0].external_name.clear();
    assert!(
        validate(&data, &schema("research_project"), false).is_ok(),
        "internal name comes from the profile"
    );
    data.participants[0].teacher_id = None;
    data.participants[0].external_name = "张\0三".into();
    assert!(matches!(
        validate(&data, &schema("research_project"), false),
        Err(ApiError::Validation(_))
    ));
}

#[test]
fn whitespace_non_text_values_fail_and_hidden_defaults_pass() {
    for (code, key) in [
        ("research_project", "project_funding"),
        ("research_project", "completed_at"),
    ] {
        let mut data = input(code);
        data.attributes.insert(key.into(), json!("   "));
        assert!(
            matches!(
                validate(&data, &schema(code), false),
                Err(ApiError::Validation(_))
            ),
            "{key} whitespace must not bypass validation"
        );
    }
    let mut hidden_schema = schema("open_class");
    for field in &mut hidden_schema.fields {
        if field.key == "remarks" || field.key == "venue" {
            field.visible = false;
        }
    }
    let mut clean = input("open_class");
    assert!(
        validate(&clean, &hidden_schema, false).is_ok(),
        "default empty remarks must not fail when hidden"
    );
    clean.attributes.insert("venue".into(), json!("   "));
    assert!(
        matches!(
            validate(&clean, &hidden_schema, false),
            Err(ApiError::Validation(_))
        ),
        "explicit whitespace on hidden field must fail"
    );
}
