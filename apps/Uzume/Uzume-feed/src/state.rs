//! Shared application state injected into every Axum handler via
//! `axum::extract::State`.
//!
//! `AppState` is cheaply cloneable — all inner types are either `Arc`-wrapped
//! or already clone-on-share (sqlx `PgPool`, `CacheClient`, `NatsClient`).

use brizo::SearchClient;
use lethe::CacheClient;
use nyx_events::NatsClient;
use sqlx::PgPool;

/// Application-wide state available to every handler.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool.
    pub db: PgPool,

    /// DragonflyDB / Redis cache connection.
    pub cache: CacheClient,

    /// NATS JetStream client for publishing domain events.
    pub nats: NatsClient,

    /// Meilisearch client for syncing posts to the search index.
    pub search: SearchClient,
}
