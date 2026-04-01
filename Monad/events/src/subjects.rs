use serde::{Deserialize, Serialize};

/// Event subject constants following `{app}.{entity}.{action}` convention.
///
/// Platform-level events use `nyx.` prefix. App-level events use the app name
/// (e.g., `Uzume.`). New apps add their own subjects below the app section.

// ── Platform-level ──────────────────────────────────────────────────────────

pub const USER_CREATED: &str = "nyx.user.created";
pub const USER_DELETED: &str = "nyx.user.deleted";
pub const APPS_LINKED: &str = "nyx.apps.linked";

// ── Uzume ───────────────────────────────────────────────────────────────────

pub const UZUME_POST_CREATED: &str = "Uzume.post.created";
pub const UZUME_POST_DELETED: &str = "Uzume.post.deleted";
pub const UZUME_POST_LIKED: &str = "Uzume.post.liked";
pub const UZUME_COMMENT_CREATED: &str = "Uzume.comment.created";
pub const UZUME_PROFILE_UPDATED: &str = "Uzume.profile.updated";
pub const UZUME_STORY_CREATED: &str = "Uzume.story.created";
pub const UZUME_STORY_VIEWED: &str = "Uzume.story.viewed";
pub const UZUME_USER_FOLLOWED: &str = "Uzume.user.followed";
pub const UZUME_USER_BLOCKED: &str = "Uzume.user.blocked";
pub const UZUME_REEL_CREATED: &str = "Uzume.reel.created";
pub const UZUME_REEL_VIEWED: &str = "Uzume.reel.viewed";

// ── Uzume Media Lifecycle ───────────────────────────────────────────────────

pub const UZUME_MEDIA_UPLOADED: &str = "Uzume.media.uploaded";
pub const UZUME_MEDIA_PROCESSED: &str = "Uzume.media.processed";

// ── Anteros ─────────────────────────────────────────────────────────────────

pub const ANTEROS_SWIPE: &str = "Anteros.swipe";
pub const ANTEROS_MATCH_CREATED: &str = "Anteros.match.created";

// ── Themis ──────────────────────────────────────────────────────────────────

pub const THEMIS_LISTING_CREATED: &str = "Themis.listing.created";
pub const THEMIS_REVIEW_CREATED: &str = "Themis.review.created";

/// Typed media lifecycle event payloads.

/// Payload for `Uzume.media.uploaded` — emitted when a raw file lands in storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadedPayload {
    /// Unique job ID for idempotency tracking.
    pub job_id: uuid::Uuid,
    /// Entity type (e.g., "story", "post", "avatar").
    pub entity_type: String,
    /// Entity ID the media belongs to.
    pub entity_id: String,
    /// Storage path of the raw uploaded file.
    pub source_path: String,
    /// Original MIME type.
    pub mime_type: String,
    /// File size in bytes.
    pub size_bytes: u64,
}

/// Payload for `Uzume.media.processed` — emitted when variants are ready.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProcessedPayload {
    /// Same job_id as the uploaded event (correlation).
    pub job_id: uuid::Uuid,
    /// Entity type.
    pub entity_type: String,
    /// Entity ID.
    pub entity_id: String,
    /// Generated variant paths (variant_name → storage_path).
    pub variants: std::collections::HashMap<String, String>,
    /// Processing duration in milliseconds.
    pub processing_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_uploaded_payload_serializes() {
        let payload = MediaUploadedPayload {
            job_id: uuid::Uuid::now_v7(),
            entity_type: "story".into(),
            entity_id: "test-id".into(),
            source_path: "Uzume/stories/test-id/original.jpg".into(),
            mime_type: "image/jpeg".into(),
            size_bytes: 1_048_576,
        };

        let json = serde_json::to_string(&payload).unwrap();
        let parsed: MediaUploadedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entity_type, "story");
        assert_eq!(parsed.mime_type, "image/jpeg");
    }

    #[test]
    fn media_processed_payload_serializes() {
        let mut variants = std::collections::HashMap::new();
        variants.insert("1080".into(), "Uzume/stories/test-id/1080.jpg".into());
        variants.insert("640".into(), "Uzume/stories/test-id/640.jpg".into());

        let payload = MediaProcessedPayload {
            job_id: uuid::Uuid::now_v7(),
            entity_type: "story".into(),
            entity_id: "test-id".into(),
            variants,
            processing_ms: 342,
        };

        let json = serde_json::to_string(&payload).unwrap();
        let parsed: MediaProcessedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.processing_ms, 342);
        assert_eq!(parsed.variants.len(), 2);
    }

    #[test]
    fn subject_constants_match_convention() {
        assert_eq!(UZUME_MEDIA_UPLOADED, "Uzume.media.uploaded");
        assert_eq!(UZUME_MEDIA_PROCESSED, "Uzume.media.processed");
        assert_eq!(UZUME_STORY_CREATED, "Uzume.story.created");
        assert_eq!(UZUME_STORY_VIEWED, "Uzume.story.viewed");
    }
}
