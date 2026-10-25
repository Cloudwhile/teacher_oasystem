//! Security records never use Forge cache's availability fallback.
use crate::api::error::ApiError;
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[async_trait]
pub trait AuthStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ApiError>;
    async fn put(&self, key: &str, value: Vec<u8>, ttl: u64) -> Result<(), ApiError>;
    async fn take(&self, key: &str) -> Result<Option<Vec<u8>>, ApiError>;
    async fn delete(&self, key: &str) -> Result<(), ApiError>;
}

#[derive(Default)]
pub struct MemoryAuthStore {
    entries: Mutex<HashMap<String, (Instant, Vec<u8>)>>,
}

#[async_trait]
impl AuthStore for MemoryAuthStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ApiError> {
        let mut entries = self.entries.lock().await;
        entries.retain(|_, (expires, _)| *expires > Instant::now());
        Ok(entries.get(key).map(|(_, value)| value.clone()))
    }
    async fn put(&self, key: &str, value: Vec<u8>, ttl: u64) -> Result<(), ApiError> {
        let mut entries = self.entries.lock().await;
        entries.retain(|_, (expires, _)| *expires > Instant::now());
        entries.insert(
            key.into(),
            (Instant::now() + Duration::from_secs(ttl), value),
        );
        Ok(())
    }
    async fn take(&self, key: &str) -> Result<Option<Vec<u8>>, ApiError> {
        Ok(self
            .entries
            .lock()
            .await
            .remove(key)
            .filter(|(expires, _)| *expires > Instant::now())
            .map(|(_, value)| value))
    }
    async fn delete(&self, key: &str) -> Result<(), ApiError> {
        self.entries.lock().await.remove(key);
        Ok(())
    }
}

struct RedisAuthStore {
    client: redis::Client,
}

impl RedisAuthStore {
    async fn command<T: redis::FromRedisValue>(&self, command: redis::Cmd) -> Result<T, ApiError> {
        tokio::time::timeout(Duration::from_secs(3), async {
            let mut connection = self
                .client
                .get_multiplexed_async_connection()
                .await
                .map_err(|_| ApiError::Unavailable)?;
            command
                .query_async(&mut connection)
                .await
                .map_err(|_| ApiError::Unavailable)
        })
        .await
        .map_err(|_| ApiError::Unavailable)?
    }
}

#[async_trait]
impl AuthStore for RedisAuthStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ApiError> {
        self.command(redis::cmd("GET").arg(key).to_owned()).await
    }
    async fn put(&self, key: &str, value: Vec<u8>, ttl: u64) -> Result<(), ApiError> {
        self.command(
            redis::cmd("SET")
                .arg(key)
                .arg(value)
                .arg("EX")
                .arg(ttl)
                .to_owned(),
        )
        .await
    }
    async fn take(&self, key: &str) -> Result<Option<Vec<u8>>, ApiError> {
        self.command(redis::cmd("GETDEL").arg(key).to_owned()).await
    }
    async fn delete(&self, key: &str) -> Result<(), ApiError> {
        self.command::<i64>(redis::cmd("DEL").arg(key).to_owned())
            .await?;
        Ok(())
    }
}

pub fn build(config: &crate::config::AuthConfig) -> crate::errors::Result<Arc<dyn AuthStore>> {
    if config.store == "memory" {
        return Ok(Arc::new(MemoryAuthStore::default()));
    }
    let endpoint = std::env::var("AUTH_REDIS_URL")
        .map_err(|_| crate::errors::AppError::Config("AUTH_REDIS_URL is required".into()))?;
    let client = redis::Client::open(endpoint)
        .map_err(|_| crate::errors::AppError::Config("invalid AUTH_REDIS_URL".into()))?;
    Ok(Arc::new(RedisAuthStore { client }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn state_is_consumed_once_and_expiry_is_enforced() {
        let store = Arc::new(MemoryAuthStore::default());
        store
            .put("state", b"value".to_vec(), 60)
            .await
            .expect("put");
        let (a, b) = tokio::join!(store.take("state"), store.take("state"));
        assert_eq!(
            usize::from(a.expect("first").is_some()) + usize::from(b.expect("second").is_some()),
            1
        );
        store.put("expired", vec![1], 0).await.expect("put expired");
        assert!(store.get("expired").await.expect("get expired").is_none());
    }
    #[tokio::test]
    async fn redis_failure_does_not_fall_back_to_memory() {
        let store = RedisAuthStore {
            client: redis::Client::open("redis://127.0.0.1:1").expect("client"),
        };
        assert!(matches!(
            store.get("state").await,
            Err(ApiError::Unavailable)
        ));
        assert!(store.put("session", vec![1], 60).await.is_err());
        assert!(store.take("state").await.is_err());
    }
}
