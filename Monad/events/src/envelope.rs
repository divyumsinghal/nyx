//! Typed event envelope for the Nyx platform.
//!
//! Every event flowing through the Nyx event bus uses the [`NyxEvent<T>`] envelope.
//! This guarantees a consistent shape across all apps and services:
//!
//! ```text
//! {
//!   "id": "0194e5a2-7b3c-7def-8a12-...",
//!   "subject": "Uzume.media.uploaded",
//!   "app": "uzume",
//!   "timestamp": "2026-03-31T12:00:00Z",
//!   "payload": { ... }
//! }
//! ```
//!
//! The `id` field serves as the idempotency key — consumers can deduplicate
//! by tracking processed event IDs.

use nun::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A typed event envelope wrapping any serializable payload.
///
/// The phantom type parameter `T` ensures compile-time type safety when
/// publishing and subscribing to events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NyxEvent<T> {
    /// Unique event identifier (UUIDv7). Also serves as idempotency key.
    pub id: Uuid,
    /// Event subject following `{app}.{entity}.{action}` convention.
    pub subject: String,
    /// The app that originated this event (lowercase).
    pub app: String,
    /// UTC timestamp when the event was created.
    pub timestamp: Timestamp,
    /// The typed payload specific to this event type.
    pub payload: T,
}

impl<T> NyxEvent<T> {
    /// Create a new event with the given subject, app, and payload.
    ///
    /// Generates a UUIDv7 `id` and captures the current UTC timestamp.
    pub fn new(subject: impl Into<String>, app: impl Into<String>, payload: T) -> Self {
        Self {
            id: Uuid::now_v7(),
            subject: subject.into(),
            app: app.into(),
            timestamp: nun::time::now(),
            payload,
        }
    }

    /// Create a new event with an explicit id (for replay/testing).
    pub fn with_id(
        id: Uuid,
        subject: impl Into<String>,
        app: impl Into<String>,
        payload: T,
    ) -> Self {
        Self {
            id,
            subject: subject.into(),
            app: app.into(),
            timestamp: nun::time::now(),
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        value: String,
    }

    #[test]
    fn new_event_has_valid_uuid_and_timestamp() {
        let event = NyxEvent::new(
            "test.subject",
            "test-app",
            TestPayload {
                value: "hello".into(),
            },
        );

        assert_eq!(event.id.get_version(), Some(uuid::Version::SortRand));
        assert_eq!(event.subject, "test.subject");
        assert_eq!(event.app, "test-app");
        assert!(event.timestamp.timestamp() > 0);
        assert_eq!(event.payload.value, "hello");
    }

    #[test]
    fn with_id_preserves_given_id() {
        let fixed_id = Uuid::nil();
        let event = NyxEvent::with_id(
            fixed_id,
            "test.subject",
            "test-app",
            TestPayload {
                value: "hello".into(),
            },
        );

        assert_eq!(event.id, fixed_id);
    }

    #[test]
    fn event_serializes_to_json() {
        let event = NyxEvent::new(
            "test.subject",
            "test-app",
            TestPayload {
                value: "hello".into(),
            },
        );

        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("id").is_some());
        assert_eq!(parsed["subject"], "test.subject");
        assert_eq!(parsed["app"], "test-app");
        assert!(parsed.get("timestamp").is_some());
        assert_eq!(parsed["payload"]["value"], "hello");
    }

    #[test]
    fn event_deserializes_from_json() {
        let event = NyxEvent::new(
            "test.subject",
            "test-app",
            TestPayload {
                value: "hello".into(),
            },
        );

        let json = serde_json::to_string(&event).unwrap();
        let roundtrip: NyxEvent<TestPayload> = serde_json::from_str(&json).unwrap();

        assert_eq!(roundtrip.id, event.id);
        assert_eq!(roundtrip.subject, event.subject);
        assert_eq!(roundtrip.app, event.app);
        assert_eq!(roundtrip.payload.value, event.payload.value);
    }
}
