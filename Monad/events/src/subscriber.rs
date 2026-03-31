//! Typed event subscriber.
use async_nats::{Client, Subscriber as NatsSubscriber};
use serde::de::DeserializeOwned;
use Nun::{NyxError, Result};

use crate::envelope::NyxEvent;

/// Subscribes to NATS subjects.
pub struct Subscriber {
    client: Client,
}

impl Subscriber {
    /// Create a new [`Subscriber`].
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Subscribe to `subject` and return the raw NATS subscriber.
    ///
    /// Use [`decode_message`] to deserialise individual messages.
    ///
    /// # Errors
    ///
    /// Returns [`NyxError`] if the subscription cannot be created.
    pub async fn subscribe(&self, subject: &str) -> Result<NatsSubscriber> {
        self.client
            .subscribe(subject.to_owned())
            .await
            .map_err(NyxError::internal)
    }
}

/// Decode a raw NATS message payload into a typed [`NyxEvent`].
///
/// # Errors
///
/// Returns [`NyxError`] when the payload is not valid JSON for `T`.
pub fn decode_message<T: DeserializeOwned>(
    msg: &async_nats::Message,
) -> Result<NyxEvent<T>> {
    serde_json::from_slice(&msg.payload)
        .map_err(|e| NyxError::bad_request("invalid_event", e.to_string()))
}
