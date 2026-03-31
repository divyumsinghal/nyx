//! Typed event publisher.
use async_nats::Client;
use serde::Serialize;
use Nun::{NyxApp, NyxError, Result};

use crate::envelope::NyxEvent;

/// Publishes typed events to NATS subjects.
pub struct Publisher {
    client: Client,
    app: NyxApp,
}

impl Publisher {
    /// Create a new [`Publisher`] for the given app.
    pub fn new(client: Client, app: NyxApp) -> Self {
        Self { client, app }
    }

    /// Serialise `payload` into a [`NyxEvent`] envelope and publish it.
    ///
    /// # Errors
    ///
    /// Returns [`NyxError`] on serialisation or NATS publish failure.
    pub async fn publish<T: Serialize>(&self, subject: &str, payload: T) -> Result<()> {
        let event = NyxEvent::new(subject, self.app, payload);
        let bytes = serde_json::to_vec(&event).map_err(NyxError::internal)?;
        self.client
            .publish(subject.to_owned(), bytes.into())
            .await
            .map_err(NyxError::internal)
    }
}
