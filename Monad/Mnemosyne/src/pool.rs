use std::time::Duration;

use nun::{config::DatabaseConfig, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn build_pool_from_config(config: &DatabaseConfig) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .connect(config.url.expose())
        .await
        .map_err(nun::NyxError::from)
}
