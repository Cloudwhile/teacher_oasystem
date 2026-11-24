//! Shared API route helpers.

use actix_web::{
    FromRequest, Handler, HttpRequest, HttpResponse, Resource, Responder, ResponseError, Route, web,
};

use crate::api::error::ApiError;

pub(crate) async fn api_not_found(request: HttpRequest) -> HttpResponse {
    ApiError::EndpointNotFound.at(&request).error_response()
}

pub(crate) fn method_not_allowed(allow: &'static str) -> Route {
    web::to(move |request: HttpRequest| async move {
        ApiError::MethodNotAllowed { allow }
            .at(&request)
            .error_response()
    })
}

pub(crate) fn get_resource<F, Args>(path: &str, handler: F) -> Resource
where
    F: Handler<Args> + Clone + 'static,
    Args: FromRequest + 'static,
    F::Output: Responder + 'static,
{
    web::resource(path)
        .route(web::get().to(handler.clone()))
        .route(web::head().to(handler))
        .default_service(method_not_allowed("GET, HEAD"))
}

pub(crate) fn post_resource<F, Args>(path: &str, handler: F) -> Resource
where
    F: Handler<Args> + 'static,
    Args: FromRequest + 'static,
    F::Output: Responder + 'static,
{
    web::resource(path)
        .route(web::post().to(handler))
        .default_service(method_not_allowed("POST"))
}
