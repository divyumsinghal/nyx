//! Configuration for the Nyx platform.
//!
//! [`NyxConfig`] is the top-level configuration struct loaded by every service
//! at startup. It contains sub-structs for each infrastructure concern (database,
//! cache, NATS, storage, search, auth, messaging).
//!
//! # Loading priority
//!
//! Configuration is resolved in this order (highest priority wins):
//! 1. Environment variables (prefix `NYX_`, double underscore for nesting)
//! 2. `config.{environment}.toml` (e.g., `config.development.toml`)
//! 3. `config.toml`
//! 4. Serde defaults
//!
//! # Environment variables
//!
//! ```bash
//! NYX_ENVIRONMENT=production
//! NYX_SERVER__PORT=3001
//! NYX_DATABASE__URL=postgres://user:pass@host/db
//! NYX_CACHE__URL=redis://localhost:6379
//! ```
//!
//! Double underscore (`__`) separates struct nesting levels. This is the
//! standard convention for the `config` crate.

use serde::Deserialize;
use std::path::Path;

use crate::error::{NyxError, Result};
use crate::sensitive::Sensitive;

// ── Top-level config ────────────────────────────────────────────────────────

/// Complete platform configuration. Loaded once at service startup.
///
/// Services pass individual sub-configs to the platform crates they use:
/// ```rust,ignore
/// let config = NyxConfig::load()?;
/// let pool = nyx_db::connect(&config.database).await?;
/// let cache = nyx_cache::connect(&config.cache).await?;
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct NyxConfig {
    /// Which environment this instance is running in.
    #[serde(default)]
    pub environment: Environment,

    /// HTTP server settings.
    #[serde(default)]
    pub server: ServerConfig,

    /// PostgreSQL connection.
    pub database: DatabaseConfig,

    /// DragonflyDB / Redis cache connection.
    pub cache: CacheConfig,

    /// NATS JetStream connection.
    pub nats: NatsConfig,

    /// MinIO / S3 object storage.
    pub storage: StorageConfig,

    /// Meilisearch full-text search.
    pub search: SearchConfig,

    /// Ory Kratos identity service.
    pub auth: AuthConfig,

    /// Continuwuity (Matrix) messaging.
    pub messaging: MessagingConfig,
}

impl NyxConfig {
    /// Load configuration with full priority chain:
    /// env vars > `config.{env}.toml` > `config.toml` > defaults.
    ///
    /// The environment is determined by `NYX_ENVIRONMENT` (defaults to
    /// `development`).
    pub fn load() -> Result<Self> {
        let env = std::env::var("NYX_ENVIRONMENT").unwrap_or_else(|_| "development".into());

        let builder = config::Config::builder()
            // Base config file (optional)
            .add_source(config::File::with_name("config").required(false))
            // Environment-specific override (optional)
            .add_source(
                config::File::with_name(&format!("config.{env}")).required(false),
            )
            // Environment variables: NYX_SERVER__PORT=3001
            .add_source(
                config::Environment::with_prefix("NYX")
                    .separator("__")
                    .try_parsing(true),
            );

        let config = builder.build()?;
        config.try_deserialize().map_err(NyxError::from)
    }

    /// Load configuration from a specific TOML file, with env var overrides.
    pub fn from_file(path: &Path) -> Result<Self> {
        let builder = config::Config::builder()
            .add_source(config::File::from(path))
            .add_source(
                config::Environment::with_prefix("NYX")
                    .separator("__")
                    .try_parsing(true),
            );

        let config = builder.build()?;
        config.try_deserialize().map_err(NyxError::from)
    }

    /// Load configuration from environment variables only.
    /// For Docker / Kubernetes deployments with no config files.
    pub fn from_env() -> Result<Self> {
        let builder = config::Config::builder().add_source(
            config::Environment::with_prefix("NYX")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder.build()?;
        config.try_deserialize().map_err(NyxError::from)
    }

    /// Returns `true` if running in development mode.
    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    /// Returns `true` if running in production mode.
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}

// ── Environment ─────────────────────────────────────────────────────────────

/// The deployment environment. Affects logging verbosity, debug endpoints,
/// and error detail in responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => f.write_str("development"),
            Self::Staging => f.write_str("staging"),
            Self::Production => f.write_str("production"),
        }
    }
}

// ── Server config ───────────────────────────────────────────────────────────

/// HTTP server settings for any Nyx service binary.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Bind address.
    #[serde(default = "defaults::server_host")]
    pub host: String,

    /// Bind port.
    #[serde(default = "defaults::server_port")]
    pub port: u16,

    /// Request timeout in seconds. Requests exceeding this are aborted.
    #[serde(default = "defaults::request_timeout_secs")]
    pub request_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: defaults::server_host(),
            port: defaults::server_port(),
            request_timeout_secs: defaults::request_timeout_secs(),
        }
    }
}

impl ServerConfig {
    /// Returns the socket address string (`host:port`).
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// ── Database config ─────────────────────────────────────────────────────────

/// PostgreSQL connection settings.
///
/// Uses a connection URL to keep config simple. The URL contains the
/// host, port, username, password, and database name.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Full connection string: `postgres://user:pass@host:5432/nyx`
    pub url: Sensitive<String>,

    /// Maximum number of connections in the pool.
    #[serde(default = "defaults::db_max_connections")]
    pub max_connections: u32,

    /// Minimum number of idle connections maintained.
    #[serde(default = "defaults::db_min_connections")]
    pub min_connections: u32,

    /// Timeout (seconds) for acquiring a connection from the pool.
    #[serde(default = "defaults::db_acquire_timeout_secs")]
    pub acquire_timeout_secs: u64,

    /// Connections idle longer than this (seconds) are closed.
    #[serde(default = "defaults::db_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
}

// ── Cache config ────────────────────────────────────────────────────────────

/// DragonflyDB / Redis connection settings.
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    /// Connection URL: `redis://host:6379`
    pub url: Sensitive<String>,

    /// Number of connections in the pool.
    #[serde(default = "defaults::cache_pool_size")]
    pub pool_size: u32,
}

// ── NATS config ─────────────────────────────────────────────────────────────

/// NATS JetStream connection settings.
#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
    /// Connection URL: `nats://host:4222`
    pub url: String,
}

// ── Storage config ──────────────────────────────────────────────────────────

/// MinIO / S3-compatible object storage settings.
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    /// S3 endpoint URL (e.g., `http://localhost:9000` for MinIO).
    pub endpoint: String,

    /// S3 region (e.g., `us-east-1`, or any value for MinIO).
    #[serde(default = "defaults::storage_region")]
    pub region: String,

    /// Bucket name.
    #[serde(default = "defaults::storage_bucket")]
    pub bucket: String,

    /// S3 access key.
    pub access_key: Sensitive<String>,

    /// S3 secret key.
    pub secret_key: Sensitive<String>,
}

// ── Search config ───────────────────────────────────────────────────────────

/// Meilisearch connection settings.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchConfig {
    /// Meilisearch URL (e.g., `http://localhost:7700`).
    pub url: String,

    /// Master API key.
    pub api_key: Sensitive<String>,
}

// ── Auth config ─────────────────────────────────────────────────────────────

/// Ory Kratos connection settings.
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// Kratos public API URL (e.g., `http://localhost:4433`).
    /// Used for session validation, login/registration flows.
    pub public_url: String,

    /// Kratos admin API URL (e.g., `http://localhost:4434`).
    /// Used for identity management, creating accounts.
    pub admin_url: String,
}

// ── Messaging config ────────────────────────────────────────────────────────

/// Continuwuity (Matrix homeserver) connection settings.
#[derive(Debug, Clone, Deserialize)]
pub struct MessagingConfig {
    /// Homeserver URL (e.g., `http://localhost:6167`).
    pub homeserver_url: String,

    /// Matrix server name (e.g., `nyx.local`).
    pub server_name: String,
}

// ── Defaults ────────────────────────────────────────────────────────────────

mod defaults {
    pub fn server_host() -> String {
        "0.0.0.0".to_string()
    }
    pub fn server_port() -> u16 {
        3000
    }
    pub fn request_timeout_secs() -> u64 {
        30
    }
    pub fn db_max_connections() -> u32 {
        10
    }
    pub fn db_min_connections() -> u32 {
        2
    }
    pub fn db_acquire_timeout_secs() -> u64 {
        5
    }
    pub fn db_idle_timeout_secs() -> u64 {
        300
    }
    pub fn cache_pool_size() -> u32 {
        8
    }
    pub fn storage_region() -> String {
        "us-east-1".to_string()
    }
    pub fn storage_bucket() -> String {
        "nyx".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.request_timeout_secs, 30);
    }

    #[test]
    fn server_addr_format() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            request_timeout_secs: 30,
        };
        assert_eq!(config.addr(), "127.0.0.1:8080");
    }

    #[test]
    fn environment_default_is_development() {
        assert_eq!(Environment::default(), Environment::Development);
    }

    #[test]
    fn environment_display() {
        assert_eq!(Environment::Production.to_string(), "production");
        assert_eq!(Environment::Staging.to_string(), "staging");
        assert_eq!(Environment::Development.to_string(), "development");
    }

    #[test]
    fn database_config_debug_redacts_url() {
        let config = DatabaseConfig {
            url: Sensitive::new("postgres://user:secret@host/db".to_string()),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_secs: 5,
            idle_timeout_secs: 300,
        };
        let debug = format!("{config:?}");
        assert!(!debug.contains("secret"));
        assert!(debug.contains("[REDACTED]"));
    }
}
