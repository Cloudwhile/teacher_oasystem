//! Local capability API. Disabled in an ordinary server process; never grants business access.
use crate::{
    api::error::{ApiError, RequestError, success},
    runtime::AppState,
};
use actix_web::{HttpRequest, HttpResponse, web};
use management_protocol::{DependencyStatus, SaveConfig, ServiceStatus};
use std::{sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;

fn authorize(
    request: &HttpRequest,
    state: &AppState,
) -> Result<Arc<crate::services::management_service::Management>, ApiError> {
    let management = state.management.clone().ok_or(ApiError::NotFound)?;
    if !request
        .peer_addr()
        .is_some_and(|peer| peer.ip().is_loopback())
        || request.headers().contains_key("origin")
        || request.headers().contains_key("sec-fetch-site")
    {
        return Err(ApiError::Forbidden);
    }
    let token = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)?;
    if !management.matches(token) {
        return Err(ApiError::Unauthorized);
    }
    Ok(management)
}

async fn status(
    request: HttpRequest,
    state: web::Data<AppState>,
    shutdown: web::Data<CancellationToken>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let management = authorize(&request, &state)?;
        let timeout = Duration::from_secs(2);
        let (database, cache, auth_store) = tokio::join!(
            tokio::time::timeout(
                timeout,
                aster_forge_db::ping_database(state.db_handles.reader())
            ),
            tokio::time::timeout(timeout, state.cache.health_check()),
            tokio::time::timeout(timeout, state.auth_store.get("teacher_oa:health")),
        );
        success(
            &request,
            ServiceStatus {
                service: env!("CARGO_PKG_NAME").into(),
                version: env!("CARGO_PKG_VERSION").into(),
                pid: std::process::id(),
                uptime_seconds: management.uptime(),
                stopping: shutdown.is_cancelled(),
                dependencies: vec![
                    DependencyStatus {
                        name: "数据库".into(),
                        healthy: matches!(database, Ok(Ok(_))),
                    },
                    DependencyStatus {
                        name: "缓存".into(),
                        healthy: matches!(cache, Ok(Ok(_))),
                    },
                    DependencyStatus {
                        name: "会话".into(),
                        healthy: matches!(auth_store, Ok(Ok(_))),
                    },
                ],
            },
        )
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
}

async fn config(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let management = authorize(&request, &state)?;
        let active = state.config.clone();
        let snapshot = web::block(move || management.snapshot(&active))
            .await
            .map_err(|_| ApiError::Internal)??;
        success(&request, snapshot)
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
}

async fn save_config(
    request: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<SaveConfig>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let management = authorize(&request, &state)?;
        let active = state.config.clone();
        let body = body.into_inner();
        let snapshot = web::block(move || management.save(&active, body))
            .await
            .map_err(|_| ApiError::Internal)??;
        success(&request, snapshot)
    }
    .await;
    result.map_err(|error: ApiError| error.at(&request))
}

async fn shutdown(
    request: HttpRequest,
    state: web::Data<AppState>,
    shutdown: web::Data<CancellationToken>,
) -> Result<HttpResponse, RequestError> {
    let result = (|| {
        authorize(&request, &state)?;
        tracing::info!("desktop management requested graceful shutdown");
        actix_web::rt::spawn(async move {
            // Flush the HTTP acknowledgement before stopping the HTTP component.
            tokio::time::sleep(Duration::from_millis(150)).await;
            shutdown.cancel();
        });
        success(&request, serde_json::json!({"stopping": true}))
    })();
    result.map_err(|error: ApiError| error.at(&request))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/management")
            .route("/status", web::get().to(status))
            .route("/config", web::get().to(config))
            .route("/config", web::put().to(save_config))
            .route("/shutdown", web::post().to(shutdown)),
    );
}
