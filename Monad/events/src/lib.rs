//! Nyx typed event primitives.

use Nun::NyxApp;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod subjects {
    pub const UZUME_STORY_CREATED: &str = "Uzume.story.created";
    pub const UZUME_MEDIA_UPLOADED: &str = "Uzume.media.uploaded";
    pub const UZUME_MEDIA_PROCESSED: &str = "Uzume.media.processed";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NyxEvent<T> {
    pub id: Uuid,
    pub subject: String,
    pub app: NyxApp,
    pub timestamp: DateTime<Utc>,
    pub payload: T,
}

impl<T> NyxEvent<T> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct Payload {
        story_id: String,
    }

    #[test]
    fn event_constructor_sets_subject_and_payload() {
        let payload = Payload {
            story_id: "s1".to_string(),
        };

        let event = NyxEvent::new(
            subjects::UZUME_STORY_CREATED,
            NyxApp::Uzume,
            payload.clone(),
        );

        assert_eq!(event.subject, subjects::UZUME_STORY_CREATED);
        assert_eq!(event.app, NyxApp::Uzume);
        assert_eq!(event.payload, payload);
    }
}
