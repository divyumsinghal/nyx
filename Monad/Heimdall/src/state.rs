//! Shared application state threaded through Axum handlers via [`axum::extract::State`].

use std::sync::Arc;
use std::time::Duration;

use heka::NyxIdRegistry;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::HeimdallConfig;

/// Shared state available to every handler and middleware.
///
/// `AppState` is cheaply `Clone` — all inner fields are either `Clone` values
/// or `Arc`-backed handles. Axum clones this for each request automatically
/// when used with `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    /// Resolved configuration loaded from environment variables.
    pub config: HeimdallConfig,
    /// Shared `reqwest` HTTP client for proxying upstream requests.
    ///
    /// Reusing a single client enables connection pooling across all requests.
    pub http: Client,
    /// Heka NyxIdRegistry with DB pool
    pub nyx_id_registry: Arc<NyxIdRegistry>,
    /// Shared PostgreSQL pool used by gateway checks and Heka helpers.
    pub db: PgPool,
}

impl AppState {
    /// Construct a new `AppState` from the provided configuration.
    ///
    /// This builds the shared `reqwest::Client` with:
    /// - `rustls` TLS enabled
    /// - Connection timeout (10s)
    /// - Request timeout (30s)
    /// - Pool idle timeout (60s)
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be constructed.
    pub async fn new(config: HeimdallConfig) -> anyhow::Result<Self> {
        let http = Client::builder()
            .use_rustls_tls()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(4)
            .build()
            .expect("failed to build reqwest::Client — this is a programmer error");

        let db = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&config.database_url)
            .await?;
        let nyx_id_registry = Arc::new(NyxIdRegistry::new(db.clone()));

        Ok(Self {
            config,
            http,
            nyx_id_registry,
            db,
        })
    }
}
