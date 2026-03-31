//! Database reset command.
//!
//! Drops all managed schemas (`Uzume` then `nyx`, in reverse dependency order)
//! using `CASCADE` to automatically remove dependent objects, then re-runs all
//! migrations via [`super::migrate::run`].
#![warn(clippy::pedantic)]

/// Drop all schemas and re-apply every migration from scratch.
///
/// **Warning**: this is a destructive operation that permanently deletes all
/// data. It is intended only for local development.
///
/// # Errors
///
/// Returns [`anyhow::Error`] on connection failure or SQL execution error.
pub async fn run(db_url: &str) -> anyhow::Result<()> {
    let pool = sqlx::PgPool::connect(db_url).await?;

    // Drop in reverse dependency order so FK constraints do not block the drop.
    tracing::info!("Dropping schema \"Uzume\" CASCADE");
    sqlx::query(r#"DROP SCHEMA IF EXISTS "Uzume" CASCADE"#)
        .execute(&pool)
        .await?;

    tracing::info!("Dropping schema nyx CASCADE");
    sqlx::query("DROP SCHEMA IF EXISTS nyx CASCADE")
        .execute(&pool)
        .await?;

    tracing::info!("Schemas dropped — re-running migrations");
    super::migrate::run(db_url).await
}
