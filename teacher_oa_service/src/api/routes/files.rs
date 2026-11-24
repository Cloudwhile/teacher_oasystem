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
            .order_by_desc(file::Column::Id)
            .paginate(state.db_handles.writer(), page_size);
        let total = pager.num_items().await?;
        let page = page.min(total.div_ceil(page_size).max(1));
        let rows = pager.fetch_page(page - 1).await?;
        let ids: Vec<i64> = rows.iter().map(|row| row.id).collect();
        let associations = repo::file_associations(state.db_handles.writer(), &user, &ids).await?;
        let items = rows
            .into_iter()
            .map(|row| {
                let mut item = repo::metadata(row);
                item.association = Some(
                    associations
                        .get(&item.id)
                        .cloned()
                        .unwrap_or(crate::domain::achievements::FileAssociation::Unlinked),
                );
                item
            })
            .collect();
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
fn is_rfc8187_attr_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
        )
}

fn encode_rfc8187_attr_chars(value: &str) -> String {
    use std::fmt::Write;
    let mut encoded = String::with_capacity(value.len());
    for &byte in value.as_bytes() {
        if is_rfc8187_attr_char(byte) {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

fn ascii_fallback_filename(filename: &str, mime_type: &str) -> String {
    let (raw_stem, raw_ext) = match filename.rsplit_once('.') {
        Some((stem, ext)) => (stem, Some(ext)),
        None => (filename, None),
    };
    let ext = raw_ext
        .map(str::trim)
        .filter(|ext| {
            (1..=10).contains(&ext.len()) && ext.bytes().all(|b| b.is_ascii_alphanumeric())
        })
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| match mime_type {
            "application/pdf" => "pdf".into(),
            "image/png" => "png".into(),
            "image/jpeg" => "jpg".into(),
            _ => "bin".into(),
        });
    let mut stem = String::new();
    for ch in raw_stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            stem.push(ch);
        } else if ch.is_whitespace() && !stem.ends_with('_') && !stem.is_empty() {
            stem.push('_');
        }
        if stem.len() >= 64 {
            break;
        }
    }
    let trimmed = stem.trim_matches(|c| c == '_' || c == '-');
    let stem = if trimmed.chars().any(|c| c.is_ascii_alphanumeric()) {
        trimmed
    } else {
        "material"
    };
    format!("{stem}.{ext}")
}

async fn read(
    request: &HttpRequest,
    state: &AppState,
    id: i64,
    inline: bool,
) -> Result<HttpResponse, ApiError> {
    let (_, user) = auth::authenticate(request, state).await?;
    let (meta, bytes) = file_service::read(state, &user, id).await?;
    let encoded = encode_rfc8187_attr_chars(&meta.filename);
    let fallback = ascii_fallback_filename(&meta.filename, &meta.mime_type);
    let mode = if inline { "inline" } else { "attachment" };
    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", meta.mime_type))
        .insert_header((
            "Content-Disposition",
            format!("{mode}; filename=\"{fallback}\"; filename*=UTF-8''{encoded}"),
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
#[aster_forge_api_docs_macros::path(delete,path="/api/v1/files/{id}",operation_id="file_delete",params(("id"=i64,Path)),security(("session"=[],"csrf"=[])),responses((status=204),(status=401,body=Failure),(status=403,body=Failure),(status=409,body=Failure),(status=500,body=Failure)))]
pub async fn delete(
    request: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse, RequestError> {
    let result = async {
        let (session, user) = auth::authenticate(&request, &state).await?;
        auth::ensure_csrf(&request, &state.config.auth, &session)?;
        file_service::delete(&state, &user, *id).await?;
        Ok(HttpResponse::NoContent()
            .insert_header(("Cache-Control", "no-store"))
            .finish())
    }
    .await;
    result.map_err(|e: ApiError| e.at(&request))
}
pub fn configure(cfg: &mut web::ServiceConfig) {
    use crate::api::common::{get_resource, method_not_allowed, post_resource};

    cfg.service(get_resource("/files", list))
        .service(post_resource("/files/upload", upload))
        .service(
            web::resource("/files/{id}")
                .route(web::delete().to(delete))
                .default_service(method_not_allowed("DELETE")),
        )
        .service(get_resource("/files/{id}/preview", preview))
        .service(get_resource("/files/{id}/download", download));
}
