//! Ushas worker — NATS subscriber → notification dispatch.

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ushas_worker=info".into()),
        )
        .json()
        .init();

    info!("ushas-worker starting (stub — NATS subscription not yet wired)");
    // TODO: connect to NATS and subscribe to uzume.post.liked, uzume.comment.created, etc.
    // For now, keep alive so the container doesn't restart.
    tokio::signal::ctrl_c().await?;
    Ok(())
}
