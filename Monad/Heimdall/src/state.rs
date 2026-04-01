//! Shared application state threaded through Axum handlers via [`axum::extract::State`].

use reqwest::Client;

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
}

impl AppState {
    /// Construct a new `AppState` from the provided configuration.
    ///
    /// This builds the shared `reqwest::Client` with `rustls` TLS enabled.
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be constructed.
    #[must_use]
    pub fn new(config: HeimdallConfig) -> Self {
        let http = Client::builder()
            .use_rustls_tls()
            .build()
            .expect("failed to build reqwest::Client — this is a programmer error");
        Self { config, http }
    }
}
