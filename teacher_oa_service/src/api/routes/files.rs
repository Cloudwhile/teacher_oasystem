#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::api::error::{Failure, Success};
#[cfg(all(debug_assertions, feature = "openapi"))]
use crate::domain::achievements::FileMetadata;
use crate::{
    api::error::{ApiError, RequestError, success},
    domain::achievements::FilePage,
    repositories::{achievement_entities::file, achievements as repo},
    runtime::AppState,
    services::{auth_service as auth, business_audit, file_service},
};
use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, web};
use futures_util::StreamExt;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct UploadBody {
    #[cfg_attr(all(debug_assertions,feature="openapi"),schema(value_type=String,format=Binary))]
    pub file: String,
}
#[derive(Deserialize)]
pub struct FileQuery {
    pub mine: Option<bool>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[aster_forge_api_docs_macros::path(post,path="/api/v1/files/upload",operation_id="file_upload",request_body(content=UploadBody,content_type="multipart/form-data"),security(("session"=[],"csrf"=[])),responses((status=200,body=Success<FileMetadata>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=413,body=Failure),(status=503,body=Failure),(status=500,body=Failure)))]
pub async fn upload(
    request: HttpRequest,
    state: web::Data<AppState>,
    payload: Result<Multipart, actix_web::Error>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (session, user) = auth::authenticate(&request, &state).await?;
        auth::ensure_csrf(&request, &state.config.auth, &session)?;
        if !repo::has_permission(&user, "files:write") {
            return Err(ApiError::Forbidden);
        }
        if request
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
            .is_some_and(|size| size > state.config.storage.max_upload_bytes + 65536)
        {
            return Err(ApiError::PayloadTooLarge);
        }
        let mut payload = payload.map_err(|_| ApiError::Invalid("上传请求格式无效"))?;
        let mut material = None;
        while let Some(field) = payload.next().await {
            let mut field = field.map_err(|_| ApiError::Invalid("上传内容无效"))?;
            if field.name() != Some("file") || material.is_some() {
                return Err(ApiError::Invalid("每次请选择一个证明文件"));
            }
            let filename = field
                .content_disposition()
                .and_then(|v| v.get_filename())
                .ok_or(ApiError::Invalid("缺少文件名"))?
                .to_string();
            let mime = field
                .content_type()
                .map(|v| v.essence_str().to_string())
                .unwrap_or_default();
            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|_| ApiError::Invalid("文件上传中断"))?;
                if bytes.len() + chunk.len() > state.config.storage.max_upload_bytes {
                    return Err(ApiError::PayloadTooLarge);
                }
                bytes.extend_from_slice(&chunk);
            }
            material = Some((filename, mime, bytes));
        }
        let (name, mime, bytes) = material.ok_or(ApiError::Invalid("请选择证明文件"))?;
        success(
            &request,
            file_service::upload(&state, &user, name, mime, bytes).await?,
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}

#[aster_forge_api_docs_macros::path(get,path="/api/v1/files",operation_id="files_list",params(("page"=Option<u64>,Query),("page_size"=Option<u64>,Query),("mine"=Option<bool>,Query)),security(("session"=[])),responses((status=200,body=Success<FilePage>),(status=400,body=Failure),(status=401,body=Failure),(status=403,body=Failure),(status=500,body=Failure)))]
pub async fn list(
    request: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<FileQuery>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (_, user) = auth::authenticate(&request, &state).await?;
        if !repo::has_permission(&user, "files:read") {
            return Err(ApiError::Forbidden);
        }
        let (page, page_size) = super::achievements::pagination(query.page, query.page_size)?;
        let mut select = file::Entity::find()
            .filter(repo::file_scope(&user))
            .filter(file::Column::UploadStatus.eq("ready"));
        if query.mine.unwrap_or(false) {
            select = select.filter(file::Column::UploadedBy.eq(user.id));
        }
        let pager = select
            .order_by_desc(file::Column::CreatedAt)
            .paginate(state.db_handles.writer(), page_size);
        let total = pager.num_items().await?;
        let page = page.min(total.div_ceil(page_size).max(1));
        let rows = pager.fetch_page(page - 1).await?;
        let mut items = Vec::new();
        for row in rows {
            let mut item = repo::metadata(row);
            item.association =
                Some(repo::file_association(state.db_handles.writer(), &user, item.id).await?);
            items.push(item);
        }
        business_audit::write(
            state.db_handles.writer(),
            user.id,
            "file.list",
            "file",
            None,
        )
        .await?;
        success(
            &request,
            FilePage {
                items,
                page,
                page_size,
                total,
            },
        )
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
async fn read(
    request: &HttpRequest,
    state: &AppState,
    id: i64,
    inline: bool,
) -> Result<HttpResponse, ApiError> {
    let (_, user) = auth::authenticate(request, state).await?;
    let (meta, bytes) = file_service::read(state, &user, id).await?;
    let filename = url::form_urlencoded::byte_serialize(meta.filename.as_bytes())
        .collect::<String>()
        .replace('+', "%20");
    let mode = if inline { "inline" } else { "attachment" };
    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", meta.mime_type))
        .insert_header((
            "Content-Disposition",
            format!("{mode}; filename=\"material\"; filename*=UTF-8''{filename}"),
        ))
        .insert_header(("Cache-Control", "no-store"))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .insert_header((
            "Content-Security-Policy",
            "sandbox; default-src 'none'; frame-ancestors 'self'",
        ))
        .body(bytes))
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/files/{id}/preview",operation_id="file_preview",params(("id"=i64,Path)),security(("session"=[])),responses((status=200,content((Vec<u8>="application/pdf"),(Vec<u8>="image/png"),(Vec<u8>="image/jpeg"))),(status=401,body=Failure),(status=403,body=Failure),(status=503,body=Failure),(status=500,body=Failure)))]
pub async fn preview(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse, RequestError> {
    read(&request, &state, *id, true)
        .await
        .map_err(|e| e.at(&request))
}
#[aster_forge_api_docs_macros::path(get,path="/api/v1/files/{id}/download",operation_id="file_download",params(("id"=i64,Path)),security(("session"=[])),responses((status=200,content((Vec<u8>="application/pdf"),(Vec<u8>="image/png"),(Vec<u8>="image/jpeg"))),(status=401,body=Failure),(status=403,body=Failure),(status=503,body=Failure),(status=500,body=Failure)))]
pub async fn download(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse, RequestError> {
    read(&request, &state, *id, false)
        .await
        .map_err(|e| e.at(&request))
}
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/files", web::get().to(list))
        .route("/files/upload", web::post().to(upload))
        .route("/files/{id}/preview", web::get().to(preview))
        .route("/files/{id}/download", web::get().to(download));
}
