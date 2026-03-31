//! Database migration command.
//!
//! Applies all pending migrations in dependency order:
//! 1. `migrations/Monad/` — creates the `nyx` schema and its tables.
//! 2. `migrations/Uzume/` — creates the `Uzume` schema which depends on `nyx`.
#![warn(clippy::pedantic)]

/// Walk up the directory tree from the current working directory until a
/// `Cargo.toml` containing `[workspace]` is found.
///
/// # Errors
///
/// Returns [`anyhow::Error`] when no workspace root can be located.
///
/// # Examples
///
/// ```no_run
/// let root = nyx_xtask::commands::migrate::find_workspace_root().unwrap();
/// assert!(root.join("Cargo.toml").exists());
/// ```
pub fn find_workspace_root() -> anyhow::Result<std::path::PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        anyhow::ensure!(dir.pop(), "Could not find workspace root (no Cargo.toml with [workspace] found)");
    }
}

/// Run all pending database migrations against the given PostgreSQL URL.
///
/// Migrations are applied in two passes to respect the FK dependency:
/// - `migrations/Monad/` first (creates `nyx` schema)
/// - `migrations/Uzume/` second (creates `Uzume` schema, references `nyx.app_aliases`)
///
/// Both passes are idempotent — re-running against an already-migrated database
/// is safe.
///
/// # Errors
///
/// Returns [`anyhow::Error`] on connection failure, missing migration directory,
/// or SQL execution error.
pub async fn run(db_url: &str) -> anyhow::Result<()> {
    let pool = sqlx::PgPool::connect(db_url).await?;

    let workspace_root = find_workspace_root()?;

    // Monad migrations first — establishes the `nyx` schema.
    let monad_path = workspace_root.join("migrations").join("Monad");
    tracing::info!(path = %monad_path.display(), "Running Monad migrations");
    sqlx::migrate::Migrator::new(monad_path)
        .await?
        .run(&pool)
        .await?;

    // Uzume migrations second — depends on `nyx.app_aliases`.
    let uzume_path = workspace_root.join("migrations").join("Uzume");
    tracing::info!(path = %uzume_path.display(), "Running Uzume migrations");
    sqlx::migrate::Migrator::new(uzume_path)
        .await?
        .run(&pool)
        .await?;

    tracing::info!("All migrations applied successfully");
    Ok(())
}
