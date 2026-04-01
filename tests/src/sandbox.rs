//! E2E sandbox infrastructure orchestration using testcontainers.
//!
//! Provides real service instances (postgres, NATS, DragonflyDB, MinIO, etc.)
//! for integration and E2E tests.

use std::collections::HashMap;
use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, Image};
use testcontainers_modules::{minio::MinIO, postgres::Postgres, redis::Redis};

/// Sandbox manager that orchestrates all test infrastructure.
pub struct SandboxManager {
    pub postgres: Option<ContainerAsync<Postgres>>,
    pub redis: Option<ContainerAsync<Redis>>,
    pub minio: Option<ContainerAsync<MinIO>>,
    // Additional containers will be added as needed
}

impl SandboxManager {
    /// Create a new empty sandbox manager.
    pub fn new() -> Self {
        Self {
            postgres: None,
            redis: None,
            minio: None,
        }
    }

    /// Start a PostgreSQL container with nyx schema.
    pub async fn with_postgres(mut self) -> anyhow::Result<Self> {
        let postgres = Postgres::default()
            .with_db_name("nyx_test")
            .with_user("nyx")
            .with_password("nyx")
            .start()
            .await?;

        self.postgres = Some(postgres);
        Ok(self)
    }

    /// Start a Redis (DragonflyDB-compatible) container.
    pub async fn with_redis(mut self) -> anyhow::Result<Self> {
        let redis = Redis::default().start().await?;
        self.redis = Some(redis);
        Ok(self)
    }

    /// Start a MinIO (S3-compatible) container.
    pub async fn with_minio(mut self) -> anyhow::Result<Self> {
        let minio = MinIO::default().start().await?;
        self.minio = Some(minio);
        Ok(self)
    }

    /// Get PostgreSQL connection URL.
    pub async fn postgres_url(&self) -> String {
        if let Some(ref container) = self.postgres {
            let host = container.get_host().await.unwrap();
            let port = container.get_host_port_ipv4(5432).await.unwrap();
            format!("postgres://nyx:nyx@{host}:{port}/nyx_test")
        } else {
            panic!("PostgreSQL container not started");
        }
    }

    /// Get Redis connection URL.
    pub async fn redis_url(&self) -> String {
        if let Some(ref container) = self.redis {
            let host = container.get_host().await.unwrap();
            let port = container.get_host_port_ipv4(6379).await.unwrap();
            format!("redis://{host}:{port}")
        } else {
            panic!("Redis container not started");
        }
    }

    /// Get MinIO connection details.
    pub async fn minio_config(&self) -> MinioConfig {
        if let Some(ref container) = self.minio {
            let host = container.get_host().await.unwrap();
            let port = container.get_host_port_ipv4(9000).await.unwrap();
            MinioConfig {
                endpoint: format!("http://{host}:{port}"),
                access_key: "minioadmin".to_string(),
                secret_key: "minioadmin".to_string(),
                bucket: "nyx-test".to_string(),
            }
        } else {
            panic!("MinIO container not started");
        }
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

/// MinIO configuration.
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

/// Custom NATS image for JetStream support.
pub struct NatsImage {
    env_vars: HashMap<String, String>,
}

impl Default for NatsImage {
    fn default() -> Self {
        Self {
            env_vars: HashMap::new(),
        }
    }
}

impl Image for NatsImage {
    fn name(&self) -> &str {
        "nats"
    }

    fn tag(&self) -> &str {
        "2.10-alpine"
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Server is ready")]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<std::borrow::Cow<'_, str>>, impl Into<std::borrow::Cow<'_, str>>)>
    {
        &self.env_vars
    }

    fn cmd(&self) -> impl IntoIterator<Item = impl Into<std::borrow::Cow<'_, str>>> {
        vec!["-js", "-m", "8222"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires Docker"]
    async fn sandbox_manager_starts_postgres() {
        let sandbox = SandboxManager::new()
            .with_postgres()
            .await
            .expect("Failed to start postgres");

        let url = sandbox.postgres_url().await;
        assert!(url.contains("postgres://"));
        assert!(url.contains("nyx_test"));
    }

    #[tokio::test]
    #[ignore = "requires Docker"]
    async fn sandbox_manager_starts_redis() {
        let sandbox = SandboxManager::new()
            .with_redis()
            .await
            .expect("Failed to start redis");

        let url = sandbox.redis_url().await;
        assert!(url.starts_with("redis://"));
    }

    #[tokio::test]
    #[ignore = "requires Docker"]
    async fn sandbox_manager_starts_minio() {
        let sandbox = SandboxManager::new()
            .with_minio()
            .await
            .expect("Failed to start minio");

        let config = sandbox.minio_config().await;
        assert!(config.endpoint.starts_with("http://"));
        assert_eq!(config.access_key, "minioadmin");
    }
}
