use super::business_audit;
use crate::{
    api::error::ApiError,
    domain::{achievements::FileMetadata, identity::CurrentUser},
    repositories::{achievement_entities::file, achievements as repo},
    runtime::AppState,
};
use actix_web::web::Bytes;
use chrono::Utc;
use object_store::{ObjectStoreExt, path::Path};
use sea_orm::{ActiveModelTrait, IntoActiveModel, Set, TransactionTrait};
use sha2::{Digest, Sha256};

pub fn inspect(filename: &str, claimed_mime: &str, bytes: &[u8]) -> Result<&'static str, ApiError> {
    if filename.is_empty()
        || filename.chars().count() > 255
        || filename
            .chars()
            .any(|c| c.is_control() || c == '/' || c == '\\')
    {
        return Err(ApiError::Invalid("文件名无效"));
    }
    let extension = filename
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mime = if bytes.starts_with(b"%PDF-") {
        if extension != "pdf" {
            return Err(ApiError::Invalid("文件扩展名与内容不一致"));
        }
        let pdf = lopdf::Document::load_mem(bytes).map_err(|_| ApiError::Invalid("PDF内容无效"))?;
        if pdf.is_encrypted() || pdf.get_pages().is_empty() || pdf.get_pages().len() > 200 {
            return Err(ApiError::Invalid("PDF无法读取或页数过多"));
        }
        for object in pdf.objects.values() {
            if contains_active_content(object, 0) {
                return Err(ApiError::Invalid("PDF包含不支持的活动内容"));
            }
        }
        "application/pdf"
    } else {
        let (format, mime) = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") && extension == "png" {
            (image::ImageFormat::Png, "image/png")
        } else if bytes.starts_with(b"\xff\xd8\xff")
            && ["jpg", "jpeg"].contains(&extension.as_str())
        {
            (image::ImageFormat::Jpeg, "image/jpeg")
        } else {
            return Err(ApiError::Invalid("仅支持有效的 PDF、JPG 和 PNG 文件"));
        };
        let mut reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(12000);
        limits.max_image_height = Some(12000);
        limits.max_alloc = Some(64 * 1024 * 1024);
        reader.limits(limits);
        reader
            .decode()
            .map_err(|_| ApiError::Invalid("图片内容无效或尺寸过大"))?;
        mime
    };
    if !claimed_mime.is_empty()
        && claimed_mime != "application/octet-stream"
        && claimed_mime != mime
    {
        return Err(ApiError::Invalid("文件类型与内容不一致"));
    }
    Ok(mime)
}
fn contains_active_content(object: &lopdf::Object, depth: usize) -> bool {
    if depth > 64 {
        return true;
    }
    match object {
        lopdf::Object::Dictionary(dict) => dict.iter().any(|(key, value)| {
            [
                b"JS".as_slice(),
                b"JavaScript",
                b"Launch",
                b"AA",
                b"OpenAction",
                b"EmbeddedFiles",
                b"RichMedia",
                b"XFA",
            ]
            .contains(&key.as_slice())
                || contains_active_content(value, depth + 1)
        }),
        lopdf::Object::Array(values) => {
            values.iter().any(|v| contains_active_content(v, depth + 1))
        }
        lopdf::Object::Stream(stream) => {
            contains_active_content(&lopdf::Object::Dictionary(stream.dict.clone()), depth + 1)
        }
        _ => false,
    }
}
pub async fn upload(
    state: &AppState,
    user: &CurrentUser,
    filename: String,
    claimed_mime: String,
    bytes: Vec<u8>,
) -> Result<FileMetadata, ApiError> {
    if !repo::has_permission(user, "files:write") {
        return Err(ApiError::Forbidden);
    }
    if bytes.len() > state.config.storage.max_upload_bytes {
        return Err(ApiError::PayloadTooLarge);
    }
    if bytes.is_empty() {
        return Err(ApiError::Invalid("文件不能为空"));
    }
    let (filename, mime, bytes) = tokio::task::spawn_blocking(move || {
        let mime = inspect(&filename, &claimed_mime, &bytes)?;
        Ok::<_, ApiError>((filename, mime, bytes))
    })
    .await
    .map_err(|_| ApiError::Internal)??;
    let sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let key = format!("evidence/{}", super::auth_service::random_token());
    let row = file::ActiveModel {
        object_key: Set(key.clone()),
        filename: Set(filename),
        size_bytes: Set(i64::try_from(bytes.len()).map_err(|_| ApiError::PayloadTooLarge)?),
        mime_type: Set(mime.into()),
        sha256: Set(sha256),
        uploaded_by: Set(user.id),
        upload_status: Set("uploading".into()),
        created_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(state.db_handles.writer())
    .await?;
    let location = Path::from(key);
    let put = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        state.files.put(&location, Bytes::from(bytes).into()),
    )
    .await;
    if !matches!(put, Ok(Ok(_))) {
        let mut failed = row.into_active_model();
        failed.upload_status = Set("failed".into());
        failed.update(state.db_handles.writer()).await?;
        return Err(ApiError::Unavailable);
    }
    let txn = state.db_handles.writer().begin().await?;
    let mut ready = row.into_active_model();
    ready.upload_status = Set("ready".into());
    let ready = ready.update(&txn).await?;
    business_audit::write(&txn, user.id, "file.upload", "file", Some(ready.id)).await?;
    txn.commit().await?;
    Ok(repo::metadata(ready))
}
pub async fn read(
    state: &AppState,
    user: &CurrentUser,
    id: i64,
) -> Result<(FileMetadata, Bytes), ApiError> {
    let row = repo::file_access(state.db_handles.writer(), user, id).await?;
    let object = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        state.files.get(&Path::from(row.object_key.clone())),
    )
    .await
    .map_err(|_| ApiError::Unavailable)?
    .map_err(|_| ApiError::Unavailable)?;
    if object.meta.size != u64::try_from(row.size_bytes).map_err(|_| ApiError::Internal)? {
        return Err(ApiError::Unavailable);
    }
    let bytes = tokio::time::timeout(std::time::Duration::from_secs(15), object.bytes())
        .await
        .map_err(|_| ApiError::Unavailable)?
        .map_err(|_| ApiError::Unavailable)?;
    if i64::try_from(bytes.len()).map_err(|_| ApiError::Internal)? != row.size_bytes
        || Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            != row.sha256
    {
        return Err(ApiError::Unavailable);
    }
    business_audit::write(
        state.db_handles.writer(),
        user.id,
        "file.read",
        "file",
        Some(id),
    )
    .await?;
    Ok((repo::metadata(row), bytes))
}
