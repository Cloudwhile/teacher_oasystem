use super::response::ApiErrorCode;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, ResponseError, http::StatusCode};
use aster_forge_api::response::{ApiErrorInfo, ApiResponse};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("请先登录")]
    Unauthorized,
    #[error("无权访问此内容")]
    Forbidden,
    #[error("{0}")]
    Invalid(&'static str),
    #[error("服务暂时不可用，请稍后重试")]
    Unavailable,
    #[error("请求处理失败，请稍后重试")]
    Internal,
    #[error("未找到记录")]
    NotFound,
    #[error("请检查填写内容")]
    Validation(std::collections::BTreeMap<String, String>),
    #[error("{0}")]
    Conflict(&'static str),
    #[error("文件超过允许大小")]
    PayloadTooLarge,
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(error: sea_orm::DbErr) -> Self {
        tracing::error!(error = %error, "identity database operation failed");
        Self::Internal
    }
}

pub fn request_id(request: &HttpRequest) -> String {
    request
        .extensions()
        .get::<aster_forge_middleware::actix::request_id::RequestId>()
        .map(|value| value.0.clone())
        .unwrap_or_default()
}

#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct RequestError {
    pub error: ApiError,
    pub request_id: String,
}

impl ApiError {
    pub fn at(self, request: &HttpRequest) -> RequestError {
        RequestError {
            error: self,
            request_id: request_id(request),
        }
    }
}

#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct Failure {
    #[serde(flatten)]
    pub body: ApiResponse<NoData, ApiErrorCode, ValidationDetails>,
    pub request_id: String,
}

type NoData = ();

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ValidationDetails {
    pub fields: std::collections::BTreeMap<String, String>,
}

#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct Success<T> {
    #[serde(flatten)]
    pub body: ApiResponse<T, ApiErrorCode>,
    pub request_id: String,
}

pub fn success<T: Serialize>(request: &HttpRequest, data: T) -> Result<HttpResponse, ApiError> {
    let body = ApiResponse::ok(ApiErrorCode::Success, data).map_err(|_| ApiError::Internal)?;
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(Success {
            body,
            request_id: request_id(request),
        }))
}

impl ResponseError for RequestError {
    fn status_code(&self) -> StatusCode {
        match self.error {
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::Invalid(_) => StatusCode::BAD_REQUEST,
            ApiError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }
    fn error_response(&self) -> HttpResponse {
        let code = match self.error {
            ApiError::Unauthorized => ApiErrorCode::Unauthorized,
            ApiError::Forbidden => ApiErrorCode::Forbidden,
            ApiError::Invalid(_) => ApiErrorCode::InvalidRequest,
            ApiError::Unavailable => ApiErrorCode::Unavailable,
            ApiError::Internal => ApiErrorCode::InternalError,
            ApiError::NotFound => ApiErrorCode::EndpointNotFound,
            ApiError::Validation(_) => ApiErrorCode::ValidationFailed,
            ApiError::Conflict(_) => ApiErrorCode::Conflict,
            ApiError::PayloadTooLarge => ApiErrorCode::PayloadTooLarge,
        };
        let diagnostic = if let ApiError::Validation(fields) = &self.error {
            Some(ValidationDetails {
                fields: fields.clone(),
            })
        } else {
            None
        };
        match ApiResponse::error_with_details(
            code,
            self.error.to_string(),
            Some(ApiErrorInfo {
                retryable: matches!(self.error, ApiError::Unavailable),
                diagnostic,
            }),
        ) {
            Ok(body) => HttpResponse::build(self.status_code())
                .insert_header(("Cache-Control", "no-store"))
                .json(Failure {
                    body,
                    request_id: self.request_id.clone(),
                }),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }
}
