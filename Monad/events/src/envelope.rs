//! `NyxEvent<T>` — the typed event envelope.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use Nun::NyxApp;

/// The envelope wrapping every event published on NATS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NyxEvent<T> {
    /// Unique event ID (UUIDv7).
    pub id: Uuid,
    /// NATS subject this event was published on.
    pub subject: String,
    /// Which app published this event.
    pub app: NyxApp,
    /// When the event was created (UTC).
    pub timestamp: DateTime<Utc>,
    /// The domain payload.
    pub payload: T,
}

impl<T: Serialize + for<'de> Deserialize<'de>> NyxEvent<T> {
    /// Wrap a payload in the standard envelope.
    pub fn new(subject: impl Into<String>, app: NyxApp, payload: T) -> Self {
        Self {
            id: Uuid::now_v7(),
            subject: subject.into(),
            app,
            timestamp: Utc::now(),
            payload,
        }
    }
}
