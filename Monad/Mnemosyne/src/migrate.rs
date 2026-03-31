//! Database migration runner.
use Nun::{NyxError, Result};

/// Run all pending migrations from `migrations_path` against `pool`.
///
/// # Errors
///
/// Returns [`NyxError`] if the migrator cannot be built or a migration fails.
pub async fn run_migrations(pool: &sqlx::PgPool, migrations_path: &std::path::Path) -> Result<()> {
    sqlx::migrate::Migrator::new(migrations_path)
        .await
        .map_err(NyxError::internal)?
        .run(pool)
        .await
        .map_err(NyxError::internal)?;

    tracing::info!(path = %migrations_path.display(), "Migrations applied");
    Ok(())
}
