//! PostgreSQL connection pool.
use std::time::Duration;

use Nun::{NyxError, Result, config::DatabaseConfig};

/// Type alias for a PostgreSQL connection pool.
pub type DbPool = sqlx::PgPool;

/// Create a [`DbPool`] from the given [`DatabaseConfig`].
///
/// # Errors
///
/// Returns [`NyxError`] if the pool cannot be created or an initial
/// connection cannot be established.
pub async fn connect(config: &DatabaseConfig) -> Result<DbPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .connect(config.url.as_ref())
        .await
        .map_err(NyxError::internal)?;

    tracing::info!(
        max_connections = config.max_connections,
        "PostgreSQL pool established"
    );

    Ok(pool)
}
