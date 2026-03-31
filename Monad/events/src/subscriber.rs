use async_nats::Message;
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use std::pin::Pin;

use crate::client::NatsClient;
use crate::envelope::NyxEvent;
use crate::NatsError;

/// A stream of typed events from a NATS subscription.
pub type EventStream<T> = Pin<Box<dyn Stream<Item = Result<NyxEvent<T>, NatsError>> + Send>>;

/// Typed event subscriber.
///
/// Wraps a [`NatsClient`] and deserializes incoming messages into
/// [`NyxEvent<T>`] envelopes.
pub struct Subscriber {
    client: NatsClient,
}

impl Subscriber {
    /// Create a new subscriber.
    pub fn new(client: NatsClient) -> Self {
        Self { client }
    }

    /// Subscribe to a subject and return a stream of typed events.
    ///
    /// Each incoming message is deserialized as `NyxEvent<T>`. Messages that
    /// fail deserialization are logged and skipped.
    pub async fn subscribe<T: DeserializeOwned + Send + 'static>(
        &self,
        subject: &str,
    ) -> Result<EventStream<T>, NatsError> {
        let subscriber = self.client.subscribe(subject.to_string()).await?;

        let subject_str = subject.to_string();
        let stream = async_stream::stream! {
            let mut sub = subscriber;
            while let Some(msg) = sub.next().await {
                match parse_event::<T>(&msg) {
                    Ok(event) => yield Ok(event),
                    Err(e) => {
                        tracing::warn!(
                            subject = subject_str,
                            error = %e,
                            "failed to deserialize event, skipping"
                        );
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Get the underlying NATS client.
    pub fn client(&self) -> &NatsClient {
        &self.client
    }
}

fn parse_event<T: DeserializeOwned>(msg: &Message) -> Result<NyxEvent<T>, NatsError> {
    let event: NyxEvent<T> = serde_json::from_slice(&msg.payload)
        .map_err(|e| NatsError::Subscribe(format!("deserialization failed: {e}")))?;
    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        value: i32,
    }

    #[test]
    fn parse_event_valid_json() {
        let event = NyxEvent::new("test.subject", "test-app", TestPayload { value: 42 });
        let json = serde_json::to_vec(&event).unwrap();

        let msg = Message {
            subject: "test.subject".into(),
            reply: None,
            payload: json.into(),
            headers: None,
            status: None,
            description: None,
            length: 0,
        };

        let parsed: NyxEvent<TestPayload> = parse_event(&msg).unwrap();
        assert_eq!(parsed.payload.value, 42);
        assert_eq!(parsed.subject, "test.subject");
    }

    #[test]
    fn parse_event_invalid_json() {
        let msg = Message {
            subject: "test.subject".into(),
            reply: None,
            payload: b"not json".to_vec().into(),
            headers: None,
            status: None,
            description: None,
            length: 0,
        };

        let result = parse_event::<TestPayload>(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn parse_event_wrong_payload_type() {
        let event = NyxEvent::new("test.subject", "test-app", TestPayload { value: 42 });
        let json = serde_json::to_vec(&event).unwrap();

        // Try to parse as a different payload type
        #[derive(Debug, Deserialize)]
        struct WrongPayload {
            #[allow(dead_code)]
            name: String,
        }

        let msg = Message {
            subject: "test.subject".into(),
            reply: None,
            payload: json.into(),
            headers: None,
            status: None,
            description: None,
            length: 0,
        };

        let result = parse_event::<WrongPayload>(&msg);
        assert!(result.is_err());
    }
}
