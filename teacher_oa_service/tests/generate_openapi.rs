#![cfg(all(debug_assertions, feature = "openapi"))]
//! OpenAPI generation test.

use std::fs;

use teacher_oa_service::api::openapi::ApiDoc;
use utoipa::OpenApi;

#[test]
fn generate_openapi() {
    let mut doc = ApiDoc::openapi();
    doc.info.title = "TeacherOA".to_string();
    doc.info.description = Some("教师综合 OA 接口".to_string());
    let json = format!(
        "{}\n",
        serde_json::to_string_pretty(&doc).expect("serialize openapi document")
    );

    fs::create_dir_all("./frontend-panel/generated").expect("create frontend generated directory");
    fs::write("./frontend-panel/generated/openapi.json", json)
        .expect("write frontend OpenAPI spec");
}

#[test]
fn generated_openapi_contains_health_route() {
    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).expect("openapi json value");

    assert!(value["paths"].get("/health").is_some());
    assert!(value["paths"].get("/health/ready").is_some());
    assert!(
        value["components"]["schemas"]
            .get("StatusResponse")
            .is_some()
    );
    assert!(
        value["components"]["schemas"]
            .get("ErrorResponse")
            .is_some()
    );
}

#[test]
fn collection_responses_keep_distinct_item_types() {
    let value = serde_json::to_value(ApiDoc::openapi()).expect("OpenAPI JSON");
    for (path, collection, item) in [
        (
            "/api/v1/achievement-types",
            "AchievementTypes",
            "AchievementType",
        ),
        ("/api/v1/academic-terms", "AcademicTerms", "AcademicTerm"),
        ("/api/v1/dictionaries", "DictionaryItems", "DictionaryItem"),
        ("/api/v1/teachers", "TeacherOptions", "TeacherOption"),
    ] {
        let reference = value["paths"][path]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"]
            .as_str()
            .expect("response schema");
        let envelope = reference.rsplit('/').next().expect("schema name");
        assert!(envelope.contains(collection));
        let base = value["components"]["schemas"][envelope]["allOf"][0]["$ref"]
            .as_str()
            .expect("Forge response envelope");
        let envelope = base.rsplit('/').next().expect("base envelope");
        let data = &value["components"]["schemas"][envelope]["properties"]["data"];
        assert_eq!(data["type"], "array");
        assert_eq!(
            data["items"]["$ref"],
            format!("#/components/schemas/{item}")
        );
    }
}
