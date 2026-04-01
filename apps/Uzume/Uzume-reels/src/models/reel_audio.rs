//! Reel audio track domain model.
//!
//! Audio tracks are first-class entities. A reel can reference platform-uploaded
//! audio (no `original_reel_id`) or audio extracted from another creator's reel.
//! `use_count` is denormalized and incremented when a reel referencing this audio
//! is created, powering the trending audio feature.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Raw database row for an audio track.
#[derive(Debug, Clone, FromRow)]
pub struct ReelAudioRow {
    pub id: Uuid,
    pub title: String,
    pub artist_name: Option<String>,
    pub original_reel_id: Option<Uuid>,
    pub audio_key: String,
    pub duration_ms: i32,
    pub use_count: i64,
    pub created_at: DateTime<Utc>,
}

/// Data required to insert a new audio track.
#[derive(Debug, Clone)]
pub struct ReelAudioInsert {
    pub id: Uuid,
    pub title: String,
    pub artist_name: Option<String>,
    pub original_reel_id: Option<Uuid>,
    pub audio_key: String,
    pub duration_ms: i32,
}

/// Public-facing audio track representation returned in API responses.
#[derive(Debug, Clone, Serialize)]
pub struct ReelAudioResponse {
    pub id: Uuid,
    pub title: String,
    pub artist_name: Option<String>,
    /// Whether this audio was extracted from another creator's reel.
    pub original_reel_id: Option<Uuid>,
    pub audio_key: String,
    pub duration_ms: i32,
    pub use_count: i64,
    pub created_at: DateTime<Utc>,
}

impl From<ReelAudioRow> for ReelAudioResponse {
    fn from(row: ReelAudioRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            artist_name: row.artist_name,
            original_reel_id: row.original_reel_id,
            audio_key: row.audio_key,
            duration_ms: row.duration_ms,
            use_count: row.use_count,
            created_at: row.created_at,
        }
    }
}

/// Request body for creating an audio track.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateReelAudioRequest {
    /// Audio title. Max 200 characters.
    #[validate(length(min = 1, max = 200, message = "title must be between 1 and 200 characters"))]
    pub title: String,

    /// Display name of the artist. Optional.
    #[validate(length(max = 100, message = "artist_name must be 100 characters or fewer"))]
    pub artist_name: Option<String>,

    /// ID of the reel this audio was extracted from, if any.
    pub original_reel_id: Option<Uuid>,

    /// Storage key for the processed audio file in MinIO (AAC 128kbps).
    #[validate(length(min = 1, message = "audio_key must be non-empty"))]
    pub audio_key: String,

    /// Duration of the audio in milliseconds. Must be positive.
    #[validate(range(min = 1, message = "duration_ms must be positive"))]
    pub duration_ms: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_audio_row() -> ReelAudioRow {
        ReelAudioRow {
            id: Uuid::now_v7(),
            title: "Lo-fi Chill".to_string(),
            artist_name: Some("Nyx Music".to_string()),
            original_reel_id: None,
            audio_key: "Uzume/audio/lo-fi-chill.aac".to_string(),
            duration_ms: 30_000,
            use_count: 150,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn audio_response_from_row_preserves_fields() {
        let row = make_audio_row();
        let resp = ReelAudioResponse::from(row.clone());
        assert_eq!(resp.id, row.id);
        assert_eq!(resp.title, row.title);
        assert_eq!(resp.use_count, 150);
        assert!(resp.original_reel_id.is_none());
    }

    #[test]
    fn audio_response_serializes_correctly() {
        let row = make_audio_row();
        let resp = ReelAudioResponse::from(row);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["title"], "Lo-fi Chill");
        assert_eq!(json["use_count"], 150);
        assert_eq!(json["duration_ms"], 30_000);
    }

    #[test]
    fn audio_with_original_reel_id() {
        let original_id = Uuid::now_v7();
        let row = ReelAudioRow {
            original_reel_id: Some(original_id),
            ..make_audio_row()
        };
        let resp = ReelAudioResponse::from(row);
        assert_eq!(resp.original_reel_id, Some(original_id));
    }
}
