#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::api::error::{Failure, Success};
use crate::{
    api::error::{ApiError, RequestError, success},
    runtime::AppState,
    services::{
        business_audit, file_service,
        login_page_service::{self as pages, LoginPageSettings},
    },
};
use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, web};
use futures_util::StreamExt;
use object_store::{ObjectStoreExt, path::Path};
use serde::Serialize;
use sha2::{Digest, Sha256};

const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LoginPageAsset {
    pub url: String,
}

#[aster_forge_api_docs_macros::path(get,path="/api/v1/login-page",operation_id="login_page",responses((status=200,body=Success<LoginPageSettings>),(status=500,body=Failure)))]
pub async fn public(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async { success(&request, pages::public(&state).await?) }.await;
    result.map_err(|e: ApiError| e.at(&request))
}

#[aster_forge_api_docs_macros::path(get,path="/api/v1/admin/login-page",operation_id="admin_login_page",security(("session"=[])),responses((status=200,body=Success<LoginPageSettings>),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn read(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        super::admin::admin(&request, &state, false).await?;
        success(&request, pages::read(&state).await?)
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}

#[aster_forge_api_docs_macros::path(put,path="/api/v1/admin/login-page",operation_id="admin_save_login_page",request_body=LoginPageSettings,security(("session"=[],"csrf"=[])),responses((status=200,body=Success<LoginPageSettings>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn save(
    request: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<LoginPageSettings>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let actor = super::admin::admin(&request, &state, true).await?;
        success(
            &request,
            pages::save(&state, actor, body.into_inner()).await?,
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}

#[aster_forge_api_docs_macros::path(post,path="/api/v1/admin/login-page/assets",operation_id="admin_login_page_upload",request_body(content=super::files::UploadBody,content_type="multipart/form-data"),security(("session"=[],"csrf"=[])),responses((status=200,body=Success<LoginPageAsset>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=413,body=Failure),(status=500,body=Failure),(status=503,body=Failure)))]
pub async fn upload(
    request: HttpRequest,
    state: web::Data<AppState>,
    payload: Result<Multipart, actix_web::Error>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let actor = super::admin::admin(&request, &state, true).await?;
        if request
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .is_some_and(|v| v > MAX_IMAGE_BYTES + 65536)
        {
            return Err(ApiError::PayloadTooLarge);
        }
        let mut payload = payload.map_err(|_| ApiError::Invalid("上传请求格式无效"))?;
        let mut image = None;
        while let Some(field) = payload.next().await {
            let mut field = field.map_err(|_| ApiError::Invalid("上传内容无效"))?;
            if field.name() != Some("file") || image.is_some() {
                return Err(ApiError::Invalid("请选择一张图片"));
            }
            let name = field
                .content_disposition()
                .and_then(|value| value.get_filename())
                .ok_or(ApiError::Invalid("缺少文件名"))?
                .to_string();
            let claimed = field
                .content_type()
                .map(|mime| mime.essence_str().to_string())
                .unwrap_or_default();
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|_| ApiError::Invalid("图片上传中断"))?;
                if bytes.len() + chunk.len() > MAX_IMAGE_BYTES {
                    return Err(ApiError::PayloadTooLarge);
                }
                bytes.extend_from_slice(&chunk);
            }
            image = Some((name, claimed, bytes));
        }
        let (name, claimed, bytes) = image.ok_or(ApiError::Invalid("请选择图片"))?;
        let (mime, bytes) = tokio::task::spawn_blocking(move || {
            let ext = name
                .rsplit('.')
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            if !["png", "jpg", "jpeg"].contains(&ext.as_str()) {
                return Err(ApiError::Invalid("请选择PNG或JPG图片"));
            }
            let mime = file_service::inspect(&name, &claimed, &bytes)?;
            Ok::<_, ApiError>((mime, bytes))
        })
        .await
        .map_err(|_| ApiError::Internal)??;
        let hash: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let key = format!("{hash}.{}", if mime == "image/png" { "png" } else { "jpg" });
        tokio::time::timeout(
            std::time::Duration::from_secs(30),
            state.files.put(
                &Path::from(format!("login-page/{key}")),
                web::Bytes::from(bytes).into(),
            ),
        )
        .await
        .map_err(|_| ApiError::Unavailable)?
        .map_err(|_| ApiError::Unavailable)?;
        business_audit::write(
            state.db_handles.writer(),
            actor,
            "settings.login_page.image",
            "login_page_settings",
            Some(1),
        )
        .await?;
        success(
            &request,
            LoginPageAsset {
                url: format!("{}{key}", pages::ASSET_PREFIX),
            },
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}

pub async fn asset(
    request: HttpRequest,
    state: web::Data<AppState>,
    key: web::Path<String>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let mime = pages::asset_mime(&key)?;
        let get = tokio::time::timeout(std::time::Duration::from_secs(15), async {
            let object = state
                .files
                .get(&Path::from(format!("login-page/{key}")))
                .await
                .map_err(|e| match e {
                    object_store::Error::NotFound { .. } => ApiError::NotFound,
                    _ => ApiError::Unavailable,
                })?;
            object.bytes().await.map_err(|_| ApiError::Unavailable)
        })
        .await
        .map_err(|_| ApiError::Unavailable)??;
        Ok(HttpResponse::Ok()
            .content_type(mime)
            .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
            .insert_header(("X-Content-Type-Options", "nosniff"))
            .body(get))
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login-page", web::get().to(public))
        .route("/login-page/assets/{key}", web::get().to(asset))
        .route("/admin/login-page", web::get().to(read))
        .route("/admin/login-page", web::put().to(save))
        .route("/admin/login-page/assets", web::post().to(upload));
}
