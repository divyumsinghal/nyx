use std::time::Duration;

use chrono::{DateTime, Utc};
use nun::{types::NyxApp, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{keys::namespaced_key, CacheClient};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedSession {
    pub session_id: String,
    pub identity_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SessionCache {
    cache: CacheClient,
}

impl SessionCache {
    pub fn new(cache: CacheClient) -> Self {
        Self { cache }
    }

    fn key(&self, app: NyxApp, token: &str) -> String {
        namespaced_key(app, "session", token)
    }

    pub async fn store(
        &self,
        app: NyxApp,
        token: &str,
        session: &CachedSession,
        ttl: Duration,
    ) -> Result<()> {
        self.cache
            .set_json(&self.key(app, token), session, ttl)
            .await
    }

    pub async fn get(&self, app: NyxApp, token: &str) -> Result<Option<CachedSession>> {
        self.cache.get_json(&self.key(app, token)).await
    }

    pub async fn delete(&self, app: NyxApp, token: &str) -> Result<()> {
        self.cache.del(&self.key(app, token)).await
    }
}
