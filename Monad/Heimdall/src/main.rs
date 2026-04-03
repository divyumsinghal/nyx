//! Heimdall API gateway entry point.
//!
//! Loads configuration from environment variables, builds the Axum router,
//! and serves it with graceful shutdown on `CTRL+C`.
//!
//! # Environment variables
//!
//! See [`config::HeimdallConfig`] for the full list.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod auth_layer;
pub mod config;
pub mod health;
pub mod jwt;
pub mod proxy;
pub mod rate_limit;
pub mod routes;
pub mod state;
pub mod websocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise structured JSON logging.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let config = config::HeimdallConfig::from_env()?;
    let addr = format!("{}:{}", config.host, config.port);
    let state = state::AppState::new(config).await?;
    let router = routes::build_router(state);

    tracing::info!(addr = %addr, "Heimdall starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Heimdall shut down cleanly");
    Ok(())
}

/// Await a `CTRL+C` signal for graceful shutdown.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    tracing::info!("shutdown signal received");
}
