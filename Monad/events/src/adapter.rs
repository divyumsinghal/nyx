use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use nun::Result;

use crate::{EventEnvelope, EventPublisher};

#[derive(Debug, Default)]
pub struct NoopEventPublisher;

#[async_trait]
impl EventPublisher for NoopEventPublisher {
    async fn publish(&self, _event: EventEnvelope) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryEventPublisher {
    events: Arc<Mutex<Vec<EventEnvelope>>>,
}

impl InMemoryEventPublisher {
    pub fn snapshot(&self) -> Vec<EventEnvelope> {
        self.events
            .lock()
            .expect("in-memory events mutex should not be poisoned")
            .clone()
    }

    pub fn drain(&self) -> Vec<EventEnvelope> {
        std::mem::take(
            &mut *self
                .events
                .lock()
                .expect("in-memory events mutex should not be poisoned"),
        )
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventPublisher {
    async fn publish(&self, event: EventEnvelope) -> Result<()> {
        self.events
            .lock()
            .expect("in-memory events mutex should not be poisoned")
            .push(event);
        Ok(())
    }
}
