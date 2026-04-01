//! NATS `JetStream` setup command.
//!
//! Creates (or idempotently retrieves) the two top-level streams used by the
//! Nyx platform:
//!
//! - `NYX` — captures `nyx.>` subjects (identity, account lifecycle events).
//! - `UZUME` — captures `Uzume.>` subjects (all Uzume app events).
#![warn(clippy::pedantic)]

/// Plain data describing a `JetStream` stream to create.
///
/// Kept separate from `async_nats::jetstream::stream::Config` so that the
/// pure config-building functions can be tested without any network dependency.
#[derive(Debug, Clone)]
pub struct SimpleStreamConfig {
    /// Stream name (must be unique within the NATS server).
    pub name: String,
    /// Subject filter patterns this stream subscribes to.
    pub subjects: Vec<String>,
}

/// Return the configuration for the platform-wide `NYX` stream.
///
/// # Examples
///
/// ```
/// let cfg = nyx_xtask::commands::nats_setup::nyx_stream_config();
/// assert_eq!(cfg.name, "NYX");
/// ```
#[must_use]
pub fn nyx_stream_config() -> SimpleStreamConfig {
    SimpleStreamConfig {
        name: "NYX".to_string(),
        subjects: vec!["nyx.>".to_string()],
    }
}

/// Return the configuration for the Uzume app `UZUME` stream.
///
/// # Examples
///
/// ```
/// let cfg = nyx_xtask::commands::nats_setup::uzume_stream_config();
/// assert_eq!(cfg.name, "UZUME");
/// ```
#[must_use]
pub fn uzume_stream_config() -> SimpleStreamConfig {
    SimpleStreamConfig {
        name: "UZUME".to_string(),
        subjects: vec!["Uzume.>".to_string()],
    }
}

/// Connect to NATS at `nats_url` and create (or retrieve) all streams.
///
/// Uses `get_or_create_stream` so the operation is idempotent — re-running
/// against a server that already has the streams is safe.
///
/// # Errors
///
/// Returns [`anyhow::Error`] on connection failure or stream creation error.
pub async fn run(nats_url: &str) -> anyhow::Result<()> {
    let client = async_nats::connect(nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);

    for cfg in [nyx_stream_config(), uzume_stream_config()] {
        tracing::info!(stream = %cfg.name, "Creating JetStream stream");
        jetstream
            .get_or_create_stream(async_nats::jetstream::stream::Config {
                name: cfg.name.clone(),
                subjects: cfg.subjects,
                ..Default::default()
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create stream '{}': {e}", cfg.name))?;
        tracing::info!(stream = %cfg.name, "Stream ready");
    }

    tracing::info!("All NATS JetStream streams created");
    Ok(())
}
