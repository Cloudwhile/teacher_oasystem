//! Frontend asset routes.
//!
//! The generated service embeds `frontend-panel/dist` into the Rust binary. `index.html` and the
//! web manifest are treated as runtime templates so service metadata and CSP values can be injected
//! without rebuilding the frontend.

use actix_web::{HttpRequest, HttpResponse, Scope, web};
use aster_forge_utils::html::escape_html;
use rust_embed::Embed;
use std::path::PathBuf;

#[derive(Embed)]
#[folder = "$ASTER_FRONTEND_DIST_DIR"]
struct FrontendAssets;

/// Frontend override directory used by deployments that replace embedded assets.
const CUSTOM_FRONTEND_DIR: &str = "./frontend-override";
const FILE_NOT_FOUND_MESSAGE: &str = "File not found";
const INDEX_CACHE_CONTROL: &str = "no-cache";
const IMMUTABLE_ASSET_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";
const STATIC_ASSET_CACHE_CONTROL: &str = "public, max-age=86400";
const PWA_CACHE_CONTROL: &str = "no-cache";
const FRONTEND_TITLE: &str = "TeacherOA";
const FRONTEND_DESCRIPTION: &str = "TeacherOA";
const FRONTEND_FAVICON_URL: &str = "/favicon.svg";
const FALLBACK_FAVICON_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><title>TeacherOA</title><rect width="64" height="64" rx="14" fill="#1b56c4"/><path d="M32 13 8 25l24 12 18-9v13h4V26l2-1L32 13Zm-14 20.5v11C18 49.2 24.3 52 32 52s14-2.8 14-7.5v-11l-14 7-14-7Z" fill="#ffffff"/></svg>"##;

pub const FRONTEND_CSP_HEADER: &str = concat!(
    "default-src 'self'; ",
    "base-uri 'self'; ",
    "object-src 'none'; ",
    "frame-ancestors 'self'; ",
    "script-src 'self' 'unsafe-inline'; ",
    "style-src 'self' 'unsafe-inline'; ",
    "img-src 'self' data: blob:; ",
    "font-src 'self' data:; ",
    "connect-src 'self' http: https: ws: wss:; ",
    "worker-src 'self' blob:; ",
    "manifest-src 'self'"
);

pub const FRONTEND_CSP_META: &str = concat!(
    "default-src 'self'; ",
    "base-uri 'self'; ",
    "object-src 'none'; ",
    "script-src 'self' 'unsafe-inline'; ",
    "style-src 'self' 'unsafe-inline'; ",
    "img-src 'self' data: blob:; ",
    "font-src 'self' data:; ",
    "connect-src 'self' http: https: ws: wss:; ",
    "worker-src 'self' blob:; ",
    "manifest-src 'self'"
);

pub struct FrontendService;

impl FrontendService {
    async fn load_file(file_path: &str) -> Option<Vec<u8>> {
        if file_path.contains("..") {
            return None;
        }

        let custom_path = PathBuf::from(CUSTOM_FRONTEND_DIR).join(file_path);
        if let Ok(data) = tokio::fs::read(&custom_path).await {
            tracing::trace!("serving frontend override asset: {file_path}");
            return Some(data);
        }

        FrontendAssets::get(file_path).map(|file| file.data.into_owned())
    }

    fn process_index_html(html: &str, _state: &crate::runtime::AppState) -> String {
        html.replace("%ASTER_SERVICE_VERSION%", env!("CARGO_PKG_VERSION"))
            .replace("%ASTER_SERVICE_TITLE%", &escape_html(FRONTEND_TITLE))
            .replace(
                "%ASTER_SERVICE_DESCRIPTION%",
                &escape_html(FRONTEND_DESCRIPTION),
            )
            .replace(
                "%ASTER_SERVICE_FAVICON_URL%",
                &escape_html(FRONTEND_FAVICON_URL),
            )
            .replace("%ASTER_SERVICE_CSP%", &escape_html(FRONTEND_CSP_META))
    }

    fn process_manifest(manifest: &str) -> String {
        manifest
            .replace(
                "%ASTER_SERVICE_TITLE%",
                &Self::escape_manifest_string(FRONTEND_TITLE),
            )
            .replace(
                "%ASTER_SERVICE_DESCRIPTION%",
                &Self::escape_manifest_string(FRONTEND_DESCRIPTION),
            )
    }

    fn escape_manifest_string(value: &str) -> String {
        let encoded = serde_json::Value::String(value.to_string()).to_string();
        if let Some(inner) = encoded
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        {
            inner.to_string()
        } else {
            encoded
        }
    }

    fn get_content_type(path: &str) -> &'static str {
        match path.rsplit('.').next() {
            Some("css") => "text/css",
            Some("js" | "mjs") => "application/javascript",
            Some("json") => "application/json",
            Some("webmanifest") => "application/manifest+json",
            Some("png") => "image/png",
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            _ => "application/octet-stream",
        }
    }

    async fn serve_index(state: &crate::runtime::AppState) -> HttpResponse {
        let html = match Self::load_file("index.html").await {
            Some(data) => String::from_utf8_lossy(&data).into_owned(),
            None => {
                include_str!(concat!(env!("ASTER_FRONTEND_DIST_DIR"), "/index.html")).to_string()
            }
        };
        let processed = Self::process_index_html(&html, state);

        HttpResponse::Ok()
            .insert_header(("Content-Security-Policy", FRONTEND_CSP_HEADER))
            .insert_header(("Cache-Control", INDEX_CACHE_CONTROL))
            .content_type("text/html; charset=utf-8")
            .body(processed)
    }

    pub async fn handle_index(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
        Self::serve_index(state.get_ref()).await
    }

    pub async fn handle_assets(req: HttpRequest) -> HttpResponse {
        let path = req.match_info().query("path");
        let asset_path = format!("assets/{path}");
        let content_type = Self::get_content_type(path);

        match Self::load_file(&asset_path).await {
            Some(data) => HttpResponse::Ok()
                .insert_header(("Cache-Control", IMMUTABLE_ASSET_CACHE_CONTROL))
                .content_type(content_type)
                .body(data),
            None => HttpResponse::NotFound().body(FILE_NOT_FOUND_MESSAGE),
        }
    }

    pub async fn handle_static(req: HttpRequest) -> HttpResponse {
        let path = req.match_info().query("path");
        let asset_path = format!("static/{path}");
        let content_type = Self::get_content_type(path);

        match Self::load_file(&asset_path).await {
            Some(data) => HttpResponse::Ok()
                .insert_header(("Cache-Control", STATIC_ASSET_CACHE_CONTROL))
                .content_type(content_type)
                .body(data),
            None => HttpResponse::NotFound().body(FILE_NOT_FOUND_MESSAGE),
        }
    }

    pub async fn handle_favicon(_req: HttpRequest) -> HttpResponse {
        match Self::load_file("favicon.svg").await {
            Some(data) => HttpResponse::Ok()
                .insert_header(("Cache-Control", STATIC_ASSET_CACHE_CONTROL))
                .content_type("image/svg+xml")
                .body(data),
            None => HttpResponse::Ok()
                .insert_header(("Cache-Control", STATIC_ASSET_CACHE_CONTROL))
                .content_type("image/svg+xml")
                .body(FALLBACK_FAVICON_SVG),
        }
    }

    pub async fn handle_pwa_file(req: HttpRequest) -> HttpResponse {
        let filename = req.uri().path().trim_start_matches('/');
        let content_type = Self::get_content_type(filename);

        match Self::load_file(filename).await {
            Some(data) => {
                let body = if filename == "manifest.webmanifest" {
                    let manifest = String::from_utf8_lossy(&data);
                    Self::process_manifest(&manifest).into_bytes()
                } else {
                    data
                };

                HttpResponse::Ok()
                    .insert_header(("Cache-Control", PWA_CACHE_CONTROL))
                    .content_type(content_type)
                    .body(body)
            }
            None => HttpResponse::NotFound().body(FILE_NOT_FOUND_MESSAGE),
        }
    }

    pub async fn handle_spa_fallback(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
        Self::serve_index(state.get_ref()).await
    }
}

/// Frontend routes mounted at `/`; this scope must be registered last.
pub fn routes() -> Scope {
    use crate::api::common::get_resource;

    web::scope("")
        .service(get_resource("/", FrontendService::handle_index))
        .service(get_resource("/index.html", FrontendService::handle_index))
        .service(get_resource(
            "/assets/{path:.*}",
            FrontendService::handle_assets,
        ))
        .service(get_resource(
            "/static/{path:.*}",
            FrontendService::handle_static,
        ))
        .service(get_resource(
            "/favicon.svg",
            FrontendService::handle_favicon,
        ))
        .service(get_resource(
            "/registerSW.js",
            FrontendService::handle_pwa_file,
        ))
        .service(get_resource(
            "/manifest.webmanifest",
            FrontendService::handle_pwa_file,
        ))
        .service(get_resource("/sw.js", FrontendService::handle_pwa_file))
        .service(get_resource(
            "/{filename:workbox-[^/]*}",
            FrontendService::handle_pwa_file,
        ))
        .service(get_resource(
            "/{path:.*}",
            FrontendService::handle_spa_fallback,
        ))
}
