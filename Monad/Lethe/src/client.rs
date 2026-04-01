use std::time::Duration;

use fred::prelude::*;
use nun::{NyxError, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::helpers::ttl_to_secs;

#[derive(Clone)]
pub struct CacheClient {
    inner: fred::clients::Client,
}

impl CacheClient {
    pub fn from_inner(inner: fred::clients::Client) -> Self {
        Self { inner }
    }

    pub async fn connect(redis_url: &str) -> Result<Self> {
        let config = Config::from_url(redis_url).map_err(NyxError::internal)?;
        let client = Builder::from_config(config)
            .build()
            .map_err(NyxError::internal)?;
        client.init().await.map_err(NyxError::internal)?;
        Ok(Self { inner: client })
    }

    pub fn inner(&self) -> &fred::clients::Client {
        &self.inner
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let raw: Option<String> = self.inner.get(key).await.map_err(NyxError::internal)?;
        raw.map(|s| serde_json::from_str(&s).map_err(NyxError::from))
            .transpose()
    }

    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let payload = serde_json::to_string(value).map_err(NyxError::from)?;
        let ttl = ttl_to_secs(ttl)?;
        let _: () = self
            .inner
            .set(key, payload, Some(Expiration::EX(ttl)), None, false)
            .await
            .map_err(NyxError::internal)?;
        Ok(())
    }

    pub async fn del(&self, key: &str) -> Result<()> {
        let _: i64 = self.inner.del(key).await.map_err(NyxError::internal)?;
        Ok(())
    }

    pub async fn incr(&self, key: &str) -> Result<i64> {
        self.inner.incr(key).await.map_err(NyxError::internal)
    }

    pub async fn expire(&self, key: &str, ttl: Duration) -> Result<bool> {
        let ttl = ttl_to_secs(ttl)?;
        self.inner
            .expire(key, ttl, None)
            .await
            .map_err(NyxError::internal)
    }
}
