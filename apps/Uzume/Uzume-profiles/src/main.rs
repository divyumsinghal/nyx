//! `uzume-profiles` service entry point.
//!
//! Start sequence:
//! 1. Load `NyxConfig` (env vars → config files → defaults).
//! 2. Connect PostgreSQL pool, DragonflyDB cache, NATS, Meilisearch.
//! 3. Build `AppState` and Axum router.
//! 4. Wrap with `NyxServer` (adds healthz, tracing, request-id middleware).
//! 5. Spawn background NATS workers.
//! 6. Serve.

use tracing::info;
use uzume_profiles::{config, routes, state::AppState, workers};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Logging ───────────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "uzume_profiles=info,nyx_api=info".into()),
        )
        .json()
        .init();

    // ── Configuration ─────────────────────────────────────────────────────────
    let nyx_config = config::load()?;
    info!(
        environment = %nyx_config.environment,
        addr = %nyx_config.server.addr(),
        "uzume-profiles starting"
    );

    // ── Infrastructure connections ────────────────────────────────────────────
    let db = mnemosyne::build_pool_from_config(&nyx_config.database).await?;
    info!("PostgreSQL pool ready");

    let cache = lethe::CacheClient::connect(nyx_config.cache.url.expose()).await?;
    info!("DragonflyDB cache ready");

    let nats = nyx_events::NatsClient::connect(&nyx_config.nats.url).await?;
    info!("NATS JetStream ready");

    let search = brizo::connect(&nyx_config.search);
    info!("Meilisearch client ready");

    // ── Application state ─────────────────────────────────────────────────────
    let state = AppState {
        db,
        cache,
        nats,
        search,
    };

    // ── HTTP server ───────────────────────────────────────────────────────────
    let router = routes::router(state.clone());

    let server = nyx_api::NyxServer::builder()
        .with_config(nyx_config)
        .with_routes(router)
        .build()?;

    // ── Background workers ────────────────────────────────────────────────────
    tokio::spawn(workers::profile_stub::run(state.clone()));
    tokio::spawn(workers::search_sync::run(state.clone()));

    info!("background workers spawned");

    // ── Serve (blocks until shutdown signal) ──────────────────────────────────
    server.serve().await?;

    Ok(())
}
