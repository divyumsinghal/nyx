//! NATS JetStream connection.
use async_nats::Client;
use Nun::{NyxError, Result, config::NatsConfig};

/// Type alias for the NATS client.
pub type NatsClient = Client;

/// Connect to NATS using the given [`NatsConfig`].
///
/// # Errors
///
/// Returns [`NyxError`] if the connection cannot be established.
pub async fn connect(config: &NatsConfig) -> Result<NatsClient> {
    async_nats::connect(config.url.as_str())
        .await
        .map_err(NyxError::internal)
}
