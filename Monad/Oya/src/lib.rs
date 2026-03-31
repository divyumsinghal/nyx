//! Oya media pipeline primitives.

use Nun::Id;
use serde::{Deserialize, Serialize};

/// Input event payload for raw uploaded story media.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryMediaUploaded {
    pub story_id: String,
    pub raw_object_key: String,
}

/// Output event payload for processed story media.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryMediaProcessed {
    pub story_id: String,
    pub ready_object_key: String,
}

/// Build a story upload payload from a typed story ID and raw object key.
pub fn story_uploaded<T>(story_id: Id<T>, raw_object_key: impl Into<String>) -> StoryMediaUploaded {
    StoryMediaUploaded {
        story_id: story_id.to_string(),
        raw_object_key: raw_object_key.into(),
    }
}

/// Build a story processed payload from a typed story ID and processed object key.
pub fn story_processed<T>(
    story_id: Id<T>,
    ready_object_key: impl Into<String>,
) -> StoryMediaProcessed {
    StoryMediaProcessed {
        story_id: story_id.to_string(),
        ready_object_key: ready_object_key.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Story;

    #[test]
    fn uploaded_payload_carries_story_id_and_key() {
        let story_id = Id::<Story>::new();
        let payload = story_uploaded(story_id, "Uzume/stories/raw.mp4");

        assert_eq!(payload.story_id, story_id.to_string());
        assert_eq!(payload.raw_object_key, "Uzume/stories/raw.mp4");
    }

    #[test]
    fn processed_payload_carries_story_id_and_ready_key() {
        let story_id = Id::<Story>::new();
        let payload = story_processed(story_id, "Uzume/stories/ready.m3u8");

        assert_eq!(payload.story_id, story_id.to_string());
        assert_eq!(payload.ready_object_key, "Uzume/stories/ready.m3u8");
    }
}
