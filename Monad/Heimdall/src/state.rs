//! Shared application state threaded through Axum handlers via [`axum::extract::State`].

use std::sync::Arc;
use std::time::Duration;

use heka::NyxIdRegistry;
use reqwest::Client;
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
            .pool_max_idle_per_host(10)
            .build()
            .expect("failed to build reqwest::Client — this is a programmer error");

        let db = PgPool::connect(&config.database_url).await?;
        let nyx_id_registry = Arc::new(NyxIdRegistry::new(db));

        Ok(Self { config, http, nyx_id_registry })
    }
}
