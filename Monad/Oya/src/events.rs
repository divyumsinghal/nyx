//! NATS event types for the Oya media processing lifecycle.
//!
//! These types match the contracts used across the Nyx platform.
//! Subject constants follow the `{app}.{entity}.{action}` convention.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ── Subject constants ─────────────────────────────────────────────────────────

/// NATS subject for raw media uploads: `Uzume.media.uploaded`.
pub const UZUME_MEDIA_UPLOADED: &str = "Uzume.media.uploaded";
/// NATS subject for processed media variants: `Uzume.media.processed`.
pub const UZUME_MEDIA_PROCESSED: &str = "Uzume.media.processed";

// ── Payload types ─────────────────────────────────────────────────────────────

/// Payload published to `Uzume.media.uploaded` when a raw file lands in storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadedPayload {
    /// Unique job ID for idempotency tracking.
    pub job_id: uuid::Uuid,
    /// Entity type: `"story"`, `"post"`, `"reel"`, `"avatar"`.
    pub entity_type: String,
    /// Entity ID the media belongs to.
    pub entity_id: String,
    /// MinIO/S3 storage path of the original uploaded file.
    pub source_path: String,
    /// MIME type of the uploaded file.
    pub mime_type: String,
    /// File size in bytes.
    pub size_bytes: u64,
}

/// Payload published to `Uzume.media.processed` when variants are ready.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProcessedPayload {
    /// Same `job_id` as the corresponding uploaded event (for correlation).
    pub job_id: uuid::Uuid,
    /// Entity type.
    pub entity_type: String,
    /// Entity ID.
    pub entity_id: String,
    /// Map of variant name → storage path, e.g. `"1080" → "Uzume/story/abc/1080.jpg"`.
    pub variants: HashMap<String, String>,
    /// Total processing duration in milliseconds.
    pub processing_ms: u64,
}

/// Envelope wrapping all Nyx events published to NATS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NyxEvent<T> {
    /// Unique event ID (UUIDv7).
    pub id: uuid::Uuid,
    /// NATS subject the event was published to.
    pub subject: String,
    /// Name of the app or service that published the event.
    pub app: String,
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// Typed payload.
    pub payload: T,
}

impl<T> NyxEvent<T> {
    /// Construct a new event envelope with a generated UUIDv7 ID and current timestamp.
    pub fn new(subject: impl Into<String>, app: impl Into<String>, payload: T) -> Self {
        Self {
            id: uuid::Uuid::now_v7(),
            subject: subject.into(),
            app: app.into(),
            timestamp: chrono_timestamp(),
            payload,
        }
    }
}

fn chrono_timestamp() -> String {
    // Simple seconds-since-epoch timestamp to avoid importing chrono directly
    // in the events module. In production the events platform crate provides
    // full RFC-3339 timestamps.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or_else(|_| "0".to_string(), |d| d.as_secs().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_uploaded_payload_roundtrips() {
        let payload = MediaUploadedPayload {
            job_id: uuid::Uuid::now_v7(),
            entity_type: "story".into(),
            entity_id: "abc".into(),
            source_path: "Uzume/story/abc/original.jpg".into(),
            mime_type: "image/jpeg".into(),
            size_bytes: 1_048_576,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: MediaUploadedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entity_type, "story");
        assert_eq!(parsed.mime_type, "image/jpeg");
    }

    #[test]
    fn media_processed_payload_roundtrips() {
        let mut variants = HashMap::new();
        variants.insert("1080".into(), "Uzume/story/abc/1080.jpg".into());
        let payload = MediaProcessedPayload {
            job_id: uuid::Uuid::now_v7(),
            entity_type: "story".into(),
            entity_id: "abc".into(),
            variants,
            processing_ms: 123,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: MediaProcessedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.processing_ms, 123);
        assert_eq!(parsed.variants.len(), 1);
    }

    #[test]
    fn subject_constants_match_convention() {
        assert_eq!(UZUME_MEDIA_UPLOADED, "Uzume.media.uploaded");
        assert_eq!(UZUME_MEDIA_PROCESSED, "Uzume.media.processed");
    }
}
