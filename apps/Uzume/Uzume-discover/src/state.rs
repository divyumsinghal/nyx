//! Shared application state injected into every Axum handler via
//! `axum::extract::State`.
//!
//! `AppState` is cheaply cloneable — all inner types are either `Arc`-wrapped
//! or already clone-on-share (`PgPool`, `CacheClient`, `NatsClient`,
//! `StorageClient`, `SearchClient`).

use akash::StorageClient;
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

    /// MinIO / S3-compatible object storage client.
    pub storage: StorageClient,

    /// Meilisearch search client.
    pub search: SearchClient,
}
