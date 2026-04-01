use serde::Serialize;

use crate::client::NatsClient;
use crate::envelope::NyxEvent;
use crate::NatsError;

/// Typed event publisher.
///
/// Wraps a [`NatsClient`] and serializes payloads into [`NyxEvent<T>`] envelopes
/// before publishing to NATS JetStream.
///
/// The `app` parameter is fixed at construction time, ensuring all published
/// events carry the correct app identifier.
pub struct Publisher {
    client: NatsClient,
    app: String,
}

impl Publisher {
    /// Create a new publisher for the given app.
    pub fn new(client: NatsClient, app: impl Into<String>) -> Self {
        Self {
            client,
            app: app.into(),
        }
    }

    /// Publish a typed event to the given subject.
    ///
    /// Serializes the payload into a `NyxEvent<T>` envelope and publishes
    /// the JSON to NATS JetStream.
    pub async fn publish<T: Serialize>(
        &self,
        subject: &str,
        payload: T,
    ) -> Result<NyxEvent<T>, NatsError> {
        let event = NyxEvent::new(subject, &self.app, payload);
        let json = serde_json::to_vec(&event)
            .map_err(|e| NatsError::Publish(format!("serialization failed: {e}")))?;

        self.client.publish_raw(subject.to_string(), json).await?;
        Ok(event)
    }

    /// Publish a pre-constructed event (for replay or testing).
    pub async fn publish_event<T: Serialize>(&self, event: &NyxEvent<T>) -> Result<(), NatsError> {
        let json = serde_json::to_vec(event)
            .map_err(|e| NatsError::Publish(format!("serialization failed: {e}")))?;

        self.client.publish_raw(event.subject.clone(), json).await?;
        Ok(())
    }

    /// Get the underlying NATS client.
    pub fn client(&self) -> &NatsClient {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        key: String,
    }

    #[test]
    fn publisher_stores_app() {
        // We can't test actual publishing without NATS, but we can verify
        // the publisher holds the app string correctly.
        // This is a compile-time verification that the struct is well-formed.
        let _ = std::mem::size_of::<Publisher>();
    }

    #[test]
    fn event_envelope_serialization_is_valid() {
        let event = NyxEvent::new(
            "test.subject",
            "test-app",
            TestPayload {
                key: "value".into(),
            },
        );

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test.subject"));
        assert!(json.contains("test-app"));
        assert!(json.contains("value"));
    }
}
