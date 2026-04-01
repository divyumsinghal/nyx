use async_nats::jetstream;
use thiserror::Error;

/// Errors that can occur during NATS client operations.
#[derive(Debug, Error)]
pub enum NatsError {
    #[error("failed to connect to NATS: {0}")]
    Connection(#[from] async_nats::ConnectError),

    #[error("failed to create stream: {0}")]
    StreamCreate(String),

    #[error("failed to publish event: {0}")]
    Publish(String),

    #[error("failed to subscribe: {0}")]
    Subscribe(String),
}

/// NATS JetStream client wrapper.
///
/// Provides connection management, stream creation, and raw publish/subscribe
/// operations. Higher-level typed publishing/subscribing is handled by
/// [`Publisher`](crate::publisher::Publisher) and
/// [`Subscriber`](crate::subscriber::Subscriber).
#[derive(Clone)]
pub struct NatsClient {
    client: async_nats::Client,
    jetstream: jetstream::Context,
}

impl NatsClient {
    /// Connect to a NATS server and create a JetStream context.
    pub async fn connect(url: &str) -> Result<Self, NatsError> {
        let client = async_nats::connect(url).await?;
        let jetstream = jetstream::new(client.clone());
        Ok(Self { client, jetstream })
    }

    /// Connect with options (auth, retries, etc.).
    pub async fn connect_with_options(
        url: &str,
        options: async_nats::ConnectOptions,
    ) -> Result<Self, NatsError> {
        let client = options.connect(url).await?;
        let jetstream = jetstream::new(client.clone());
        Ok(Self { client, jetstream })
    }

    /// Get the underlying NATS client (for advanced usage).
    pub fn client(&self) -> &async_nats::Client {
        &self.client
    }

    /// Get the JetStream context.
    pub fn jetstream(&self) -> &jetstream::Context {
        &self.jetstream
    }

    /// Create or get a JetStream stream with the given configuration.
    pub async fn ensure_stream(
        &self,
        name: &str,
        subjects: Vec<String>,
    ) -> Result<jetstream::stream::Stream, NatsError> {
        self.jetstream
            .get_or_create_stream(jetstream::stream::Config {
                name: name.to_string(),
                subjects,
                ..Default::default()
            })
            .await
            .map_err(|e| NatsError::StreamCreate(e.to_string()))
    }

    /// Publish raw bytes to a subject.
    pub async fn publish_raw(&self, subject: String, payload: Vec<u8>) -> Result<(), NatsError> {
        self.jetstream
            .publish(subject, payload.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;
        Ok(())
    }

    /// Subscribe to a subject, returning a message stream.
    pub async fn subscribe(&self, subject: String) -> Result<async_nats::Subscriber, NatsError> {
        self.client
            .subscribe(subject)
            .await
            .map_err(|e| NatsError::Subscribe(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nats_error_display() {
        let err = NatsError::StreamCreate("stream error".into());
        let msg = err.to_string();
        assert!(msg.contains("stream error"));
    }
}
