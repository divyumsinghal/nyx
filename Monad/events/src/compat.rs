//! Backward-compatible event publishing traits.
//!
//! Provides the `EventPublisher` trait and `EventEnvelope` struct for services
//! that use dependency injection for event publishing. This keeps services
//! testable without requiring a real NATS connection in unit tests.
//!
//! For production use, wire up [`NatsPublisher`] which delegates to
//! [`NatsClient`](crate::client::NatsClient). For tests, use
//! [`NoopEventPublisher`].

use nun::{NyxApp, NyxError};
use serde_json::Value;
use uuid::Uuid;

/// A serialized event ready to publish over any transport.
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub app: NyxApp,
    pub subject: String,
    pub payload: Value,
}

impl EventEnvelope {
    pub fn new(app: NyxApp, subject: impl Into<String>, payload: Value) -> Self {
        Self {
            id: Uuid::now_v7(),
            app,
            subject: subject.into(),
            payload,
        }
    }
}

/// Trait for publishing domain events. Implement for real (NATS) and fake (noop/test) backends.
#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: EventEnvelope) -> Result<(), NyxError>;
}

/// No-op event publisher for tests and services that don't need event emission.
pub struct NoopEventPublisher;

#[async_trait::async_trait]
impl EventPublisher for NoopEventPublisher {
    async fn publish(&self, _event: EventEnvelope) -> Result<(), NyxError> {
        Ok(())
    }
}
