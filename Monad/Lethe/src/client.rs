//! DragonflyDB / Redis cache client wrapper.
use fred::prelude::*;
use Nun::{NyxError, Result, config::CacheConfig};

/// Cache client wrapping a `fred` connection pool.
#[derive(Clone)]
pub struct CacheClient {
    pool: RedisPool,
}

/// Create a [`CacheClient`] from the given [`CacheConfig`].
///
/// # Errors
///
/// Returns [`NyxError`] if the URL is invalid or the pool cannot connect.
pub async fn connect(config: &CacheConfig) -> Result<CacheClient> {
    let redis_config =
        RedisConfig::from_url(config.url.as_ref()).map_err(NyxError::internal)?;

    let pool = Builder::from_config(redis_config)
        .build_pool(config.pool_size as usize)
        .map_err(NyxError::internal)?;

    pool.init().await.map_err(NyxError::internal)?;

    tracing::info!(pool_size = config.pool_size, "Cache pool established");

    Ok(CacheClient { pool })
}

impl CacheClient {
    /// Retrieve and deserialise a cached value.
    ///
    /// Returns `Ok(None)` when the key does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`NyxError`] on Redis or deserialisation failure.
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>> {
        let raw: Option<String> = self.pool.get(key).await.map_err(NyxError::internal)?;
        match raw {
            None => Ok(None),
            Some(json) => {
                let value = serde_json::from_str(&json).map_err(NyxError::internal)?;
                Ok(Some(value))
            }
        }
    }

    /// Serialise and store a value with a TTL.
    ///
    /// # Errors
    ///
    /// Returns [`NyxError`] on Redis or serialisation failure.
    pub async fn set<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> Result<()> {
        let json = serde_json::to_string(value).map_err(NyxError::internal)?;
        #[allow(clippy::cast_possible_wrap)]
        let expiry = Expiration::EX(ttl_seconds as i64);
        self.pool
            .set::<(), _, _>(key, json, Some(expiry), None, false)
            .await
            .map_err(NyxError::internal)
    }

    /// Delete a key.
    ///
    /// # Errors
    ///
    /// Returns [`NyxError`] on Redis failure.
    pub async fn del(&self, key: &str) -> Result<()> {
        self.pool
            .del::<(), _>(key)
            .await
            .map_err(NyxError::internal)
    }

    /// Atomically increment a counter and return the new value.
    ///
    /// # Errors
    ///
    /// Returns [`NyxError`] on Redis failure.
    pub async fn incr(&self, key: &str) -> Result<i64> {
        self.pool.incr(key).await.map_err(NyxError::internal)
    }
}
