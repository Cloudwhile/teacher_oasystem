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
    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window_secs: u64,
    ) -> Result<Result<(), u64>, ApiError>;
}

pub struct MemoryAuthStore {
    entries: Mutex<HashMap<String, (Instant, Vec<u8>)>>,
    rate_limits: Mutex<HashMap<String, (Instant, u32)>>,
    /// Live entries allowed per key namespace (the key up to its last `:`), so a flood of
    /// OAuth state records cannot exhaust memory or crowd out sessions.
    capacity: usize,
}

impl Default for MemoryAuthStore {
    fn default() -> Self {
        Self::with_capacity(50_000)
    }
}

impl MemoryAuthStore {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Mutex::default(),
            rate_limits: Mutex::default(),
            capacity,
        }
    }
}

fn namespace(key: &str) -> &str {
    key.rsplit_once(':').map_or("", |(prefix, _)| prefix)
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
        let prefix = namespace(key);
        if !entries.contains_key(key)
            && entries.keys().filter(|k| namespace(k) == prefix).count() >= self.capacity
        {
            return Err(ApiError::Unavailable);
        }
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
    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window_secs: u64,
    ) -> Result<Result<(), u64>, ApiError> {
        let now = Instant::now();
        let window = Duration::from_secs(window_secs.max(1));
        let mut entries = self.rate_limits.lock().await;
        entries.retain(|_, (expires, _)| *expires > now);
        if !entries.contains_key(key) && entries.len() >= self.capacity {
            return Err(ApiError::Unavailable);
        }
        let entry = entries.entry(key.into()).or_insert((now + window, 0));
        entry.1 = entry.1.saturating_add(1);
        if entry.1 > limit {
            let remaining = entry.0.saturating_duration_since(now);
            let retry_after = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
            return Ok(Err(retry_after.max(1)));
        }
        Ok(Ok(()))
    }
}

const RATE_LIMIT_LUA: &str = r"
local current = redis.call('INCR', KEYS[1])
if current == 1 then
    redis.call('EXPIRE', KEYS[1], ARGV[2])
elseif redis.call('TTL', KEYS[1]) < 0 then
    redis.call('EXPIRE', KEYS[1], ARGV[2])
end
if current > tonumber(ARGV[1]) then
    local ttl = redis.call('TTL', KEYS[1])
    if ttl < 1 then
        ttl = tonumber(ARGV[2])
    end
    return ttl
end
return 0
";

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
    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window_secs: u64,
    ) -> Result<Result<(), u64>, ApiError> {
        let window = window_secs.max(1);
        let remaining: i64 = self
            .command(
                redis::cmd("EVAL")
                    .arg(RATE_LIMIT_LUA)
                    .arg(1_u8)
                    .arg(key)
                    .arg(limit)
                    .arg(window)
                    .to_owned(),
            )
            .await?;
        if remaining > 0 {
            Ok(Err(u64::try_from(remaining).unwrap_or(window).max(1)))
        } else {
            Ok(Ok(()))
        }
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
    async fn each_namespace_is_capped_without_affecting_others() {
        let store = MemoryAuthStore::with_capacity(2);
        store.put("app:oauth:a", vec![1], 60).await.expect("first");
        store.put("app:oauth:b", vec![1], 60).await.expect("second");
        assert!(matches!(
            store.put("app:oauth:c", vec![1], 60).await,
            Err(ApiError::Unavailable)
        ));
        store
            .put("app:oauth:a", vec![2], 60)
            .await
            .expect("overwrite");
        store
            .put("app:session:a", vec![1], 60)
            .await
            .expect("other namespace");
        store.take("app:oauth:b").await.expect("take");
        store
            .put("app:oauth:c", vec![1], 60)
            .await
            .expect("freed slot");
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
        assert!(matches!(
            store.check_rate_limit("login:1", 5, 60).await,
            Err(ApiError::Unavailable)
        ));
    }
    #[tokio::test]
    async fn rate_limit_counts_and_reports_remaining_window() {
        let store = MemoryAuthStore::default();
        assert_eq!(store.check_rate_limit("k", 2, 60).await.unwrap(), Ok(()));
        assert_eq!(store.check_rate_limit("k", 2, 60).await.unwrap(), Ok(()));
        let limited = store.check_rate_limit("k", 2, 60).await.unwrap();
        let retry_after = limited.expect_err("third request must hit limit");
        assert!((1..=60).contains(&retry_after));
    }
}
