//! OpenAPI document assembly.
//!
//! Product routes should add their annotated handlers and DTO schemas here. The module is compiled
//! only for debug builds with the `openapi` feature, keeping normal release binaries free of API
//! documentation generation overhead.

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        description = env!("CARGO_PKG_DESCRIPTION"),
        license(name = "MIT"),
    ),
    paths(
        crate::api::routes::health::health,
        crate::api::routes::health::ready,
        crate::api::routes::auth::options,
        crate::api::routes::auth::mock_login,
        crate::api::routes::auth::me,
        crate::api::routes::auth::permissions,
        crate::api::routes::auth::logout,
        crate::api::routes::auth::wecom_login,
        crate::api::routes::auth::wecom_callback,
        crate::api::routes::teachers::me,
        crate::api::routes::teachers::detail,
        crate::api::routes::catalog::types,
        crate::api::routes::catalog::schema,
        crate::api::routes::catalog::terms,
        crate::api::routes::catalog::dictionaries,
        crate::api::routes::catalog::teachers,
        crate::api::routes::achievements::list,
        crate::api::routes::achievements::create,
        crate::api::routes::achievements::detail,
        crate::api::routes::achievements::update,
        crate::api::routes::achievements::submit,
        crate::api::routes::achievements::revisions,
        crate::api::routes::files::upload,
        crate::api::routes::files::list,
        crate::api::routes::files::preview,
        crate::api::routes::files::download,
    ),
    components(
        schemas(
            crate::api::response::ErrorResponse,
            crate::api::response::StatusResponse,
            crate::api::error::Failure,
        )
    ),
    modifiers(&Security),
)]
pub struct ApiDoc;

struct Security;
impl utoipa::Modify for Security {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "session",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("teacher_oa_session"))),
            );
            components.add_security_scheme(
                "csrf",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-CSRF-Token"))),
            );
        }
    }
}
