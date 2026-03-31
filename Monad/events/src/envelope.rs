use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nun::{NyxApp, NyxError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub trait DomainEvent: Serialize {
    const APP: NyxApp;
    const SUBJECT: &'static str;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub app: NyxApp,
    pub subject: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: Value,
}

impl EventEnvelope {
    pub fn new(app: NyxApp, subject: impl Into<String>, payload: Value) -> Self {
        Self {
            id: Uuid::now_v7(),
            app,
            subject: subject.into(),
            occurred_at: Utc::now(),
            payload,
        }
    }

    pub fn from_domain<E: DomainEvent>(event: &E) -> Result<Self> {
        let payload = serde_json::to_value(event).map_err(NyxError::from)?;
        Ok(Self::new(E::APP, E::SUBJECT, payload))
    }
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: EventEnvelope) -> Result<()>;
}
