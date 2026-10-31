//! API route registration.

use actix_web::web;

pub mod achievements;
pub mod admin;
pub mod auth;
pub mod catalog;
pub mod dashboard;
pub mod files;
pub mod frontend;
pub mod health;
pub mod login_page;
pub mod management;
pub mod reviews;
pub mod teachers;

/// Versioned product API prefix used by generated services.
pub const API_V1_PREFIX: &str = "/api/v1";

/// Registers product API routes under [`API_V1_PREFIX`].
///
/// Add product-owned route modules here. The default service keeps unknown API paths from falling
/// through to the SPA fallback registered at `/`.
pub fn configure_api(cfg: &mut web::ServiceConfig) {
    cfg.app_data(
        web::JsonConfig::default()
            .limit(65536)
            .error_handler(|_, request| {
                crate::api::error::ApiError::Invalid("请求内容无效")
                    .at(request)
                    .into()
            }),
    )
    .app_data(web::QueryConfig::default().error_handler(|_, request| {
        crate::api::error::ApiError::Invalid("请求参数无效")
            .at(request)
            .into()
    }))
    .app_data(web::PathConfig::default().error_handler(|_, request| {
        crate::api::error::ApiError::Invalid("请求路径无效")
            .at(request)
            .into()
    }));
    cfg.configure(auth::configure)
        .configure(admin::configure)
        .configure(login_page::configure)
        .configure(management::configure)
        .route("/dashboard", web::get().to(dashboard::dashboard))
        .configure(teachers::configure)
        .configure(catalog::configure)
        .configure(achievements::configure)
        .configure(reviews::configure)
        .configure(files::configure);
    cfg.default_service(web::to(crate::api::common::api_not_found));
}
