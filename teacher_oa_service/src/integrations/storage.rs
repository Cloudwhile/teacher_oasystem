use crate::{config::AppConfig, errors::AppError};
use object_store::{ObjectStore, aws::AmazonS3Builder, local::LocalFileSystem, memory::InMemory};
use std::sync::Arc;
pub fn build(config: &AppConfig) -> crate::errors::Result<Arc<dyn ObjectStore>> {
    let storage = &config.storage;
    let invalid = || AppError::Config("invalid private file storage configuration".into());
    if !(1..=20 * 1024 * 1024).contains(&storage.max_upload_bytes) {
        return Err(invalid());
    }
    if config.auth.environment == "production"
        && (storage.backend != "s3" || !storage.endpoint.starts_with("https://"))
    {
        return Err(invalid());
    }
    match storage.backend.as_str() {
        "local" => {
            std::fs::create_dir_all(&storage.root)?;
            Ok(Arc::new(
                LocalFileSystem::new_with_prefix(&storage.root)
                    .map_err(|_| invalid())?
                    .with_fsync(true),
            ))
        }
        "memory" if config.auth.environment != "production" => Ok(Arc::new(InMemory::new())),
        "s3" => {
            let key = std::env::var("S3_ACCESS_KEY_ID").map_err(|_| invalid())?;
            let secret = std::env::var("S3_SECRET_ACCESS_KEY").map_err(|_| invalid())?;
            let store = AmazonS3Builder::new()
                .with_endpoint(&storage.endpoint)
                .with_bucket_name(&storage.bucket)
                .with_region(&storage.region)
                .with_access_key_id(key)
                .with_secret_access_key(secret)
                .with_allow_http(config.auth.environment != "production")
                .with_virtual_hosted_style_request(false)
                .with_retry(object_store::RetryConfig {
                    max_retries: 1,
                    ..Default::default()
                })
                .build()
                .map_err(|_| invalid())?;
            Ok(Arc::new(store))
        }
        _ => Err(invalid()),
    }
}
