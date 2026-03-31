//! `nyx-xtask` — developer CLI for the Nyx platform.
//!
//! Run `nyx-xtask --help` to see all sub-commands.
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use clap::{Parser, Subcommand};

mod commands;
mod env;

/// Nyx developer CLI.
#[derive(Parser)]
#[command(name = "nyx-xtask", about = "Nyx developer CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all pending database migrations (Monad then Uzume).
    Migrate,
    /// Drop and recreate all schemas, then re-run migrations.
    DbReset,
    /// Seed the database with development fixture data.
    Seed,
    /// Create NATS JetStream streams (NYX, UZUME).
    NatsSetup,
    /// Scaffold a new Nyx app directory structure.
    NewApp {
        /// The app name (e.g. `Anteros`). Allowed chars: ASCII alphanumeric, `-`, `.`.
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate => {
            let db_url = env::require("DATABASE_URL")?;
            commands::migrate::run(&db_url).await?;
            println!("Migrations applied successfully");
        }
        Commands::DbReset => {
            let db_url = env::require("DATABASE_URL")?;
            commands::db_reset::run(&db_url).await?;
            println!("Database reset complete");
        }
        Commands::Seed => {
            let db_url = env::require("DATABASE_URL")?;
            let root = commands::migrate::find_workspace_root()?;
            commands::seed::run(&db_url, &root.join("tools/seed-data")).await?;
            println!("Seed data inserted");
        }
        Commands::NatsSetup => {
            let nats_url = env::require("NATS_URL")?;
            commands::nats_setup::run(&nats_url).await?;
            println!("NATS streams created");
        }
        Commands::NewApp { name } => {
            let root = commands::migrate::find_workspace_root()?;
            commands::new_app::run(&name, &root)?;
            println!("App '{name}' scaffolded");
        }
    }

    Ok(())
}
