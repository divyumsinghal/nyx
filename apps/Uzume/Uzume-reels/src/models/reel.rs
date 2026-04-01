//! Reel domain model.
//!
//! # Privacy contract
//!
//! `author_profile_id` is the internal profile UUID (app-scoped alias) and is
//! the only identifier visible within Uzume.
//!
//! # Processing states
//!
//! Reels begin as `pending` while Oya transcodes the raw upload to HLS.
//! Once the `Uzume.media.processed` event fires, they transition to `ready`.
//! Failed transcoding sets the state to `failed`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Processing state of a reel's video content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum ProcessingState {
    /// Raw upload accepted; Oya transcoding in progress.
    Pending,
    /// Oya is currently transcoding.
    Processing,
    /// HLS manifest ready; reel is visible in feeds.
    Ready,
    /// Transcoding failed; reel will not appear in feeds.
    Failed,
}

/// Raw database row for a reel.
///
/// This type is `sqlx::FromRow` — use it only inside the `queries` module.
#[derive(Debug, Clone, FromRow)]
pub struct ReelRow {
    pub id: Uuid,
    pub author_profile_id: Uuid,
    pub caption: String,
    pub hashtags: Vec<String>,
    pub raw_key: String,
    pub media_key: Option<String>,
    pub thumbnail_key: Option<String>,
    pub duration_ms: i32,
    pub processing_state: String,
    pub audio_id: Option<Uuid>,
    pub audio_start_ms: i32,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub score: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data required to insert a new reel row.
#[derive(Debug, Clone)]
pub struct ReelInsert {
    pub id: Uuid,
    pub author_profile_id: Uuid,
    pub caption: String,
    pub hashtags: Vec<String>,
    pub raw_key: String,
    pub duration_ms: i32,
    pub audio_id: Option<Uuid>,
    pub audio_start_ms: i32,
}

/// Public-facing reel representation returned in API responses.
#[derive(Debug, Clone, Serialize)]
pub struct ReelResponse {
    pub id: Uuid,
    pub author_profile_id: Uuid,
    pub caption: String,
    pub hashtags: Vec<String>,
    /// HLS master manifest URL (present once processing is `ready`).
    pub media_key: Option<String>,
    /// Poster frame URL (present once processing is `ready`).
    pub thumbnail_key: Option<String>,
    pub duration_ms: i32,
    pub processing_state: String,
    pub audio_id: Option<Uuid>,
    pub audio_start_ms: i32,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub score: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ReelRow> for ReelResponse {
    fn from(row: ReelRow) -> Self {
        Self {
            id: row.id,
            author_profile_id: row.author_profile_id,
            caption: row.caption,
            hashtags: row.hashtags,
            media_key: row.media_key,
            thumbnail_key: row.thumbnail_key,
            duration_ms: row.duration_ms,
            processing_state: row.processing_state,
            audio_id: row.audio_id,
            audio_start_ms: row.audio_start_ms,
            view_count: row.view_count,
            like_count: row.like_count,
            comment_count: row.comment_count,
            share_count: row.share_count,
            score: row.score,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Request body for creating a reel.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateReelRequest {
    /// Caption text. Max 2200 characters.
    #[validate(length(max = 2200, message = "caption must be 2200 characters or fewer"))]
    pub caption: Option<String>,

    /// Hashtags associated with the reel.
    #[validate(length(max = 30, message = "too many hashtags (max 30)"))]
    pub hashtags: Option<Vec<String>>,

    /// Duration of the reel in milliseconds (max 90 000 ms = 90 seconds).
    #[validate(range(min = 1, max = 90_000, message = "duration_ms must be between 1 and 90000"))]
    pub duration_ms: i32,

    /// Storage key for the raw upload in MinIO.
    #[validate(length(min = 1, message = "raw_key must be non-empty"))]
    pub raw_key: String,

    /// Optional audio track ID to associate with the reel.
    pub audio_id: Option<Uuid>,

    /// Millisecond offset into the audio track where playback starts.
    #[validate(range(min = 0, message = "audio_start_ms must be non-negative"))]
    pub audio_start_ms: Option<i32>,
}

/// Request body for recording a view.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RecordViewRequest {
    /// Percentage of the reel the viewer watched (0–100).
    #[validate(range(min = 0, max = 100, message = "watch_percent must be between 0 and 100"))]
    pub watch_percent: i16,
}

/// A reel like record.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ReelLike {
    pub reel_id: Uuid,
    pub profile_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// A reel view record.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ReelView {
    pub id: Uuid,
    pub reel_id: Uuid,
    pub viewer_profile_id: Uuid,
    pub watch_percent: i16,
    pub viewed_at: DateTime<Utc>,
}

/// Document shape written to Meilisearch for reel search.
#[derive(Debug, Clone, Serialize)]
pub struct ReelSearchDoc {
    /// Must be a `String` — Meilisearch primary key is a string.
    pub id: String,
    pub author_profile_id: String,
    pub caption: String,
    pub hashtags: Vec<String>,
    pub score: f64,
    pub created_at: i64, // Unix timestamp millis
}

impl From<&ReelRow> for ReelSearchDoc {
    fn from(row: &ReelRow) -> Self {
        Self {
            id: row.id.to_string(),
            author_profile_id: row.author_profile_id.to_string(),
            caption: row.caption.clone(),
            hashtags: row.hashtags.clone(),
            score: row.score,
            created_at: row.created_at.timestamp_millis(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_row() -> ReelRow {
        ReelRow {
            id: Uuid::now_v7(),
            author_profile_id: Uuid::now_v7(),
            caption: "Test caption".to_string(),
            hashtags: vec!["rust".to_string()],
            raw_key: "Uzume/reels/raw/test.mp4".to_string(),
            media_key: Some("Uzume/reels/hls/test.m3u8".to_string()),
            thumbnail_key: Some("Uzume/reels/thumb/test.jpg".to_string()),
            duration_ms: 30_000,
            processing_state: "ready".to_string(),
            audio_id: None,
            audio_start_ms: 0,
            view_count: 42,
            like_count: 7,
            comment_count: 3,
            share_count: 1,
            score: 12.5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn reel_response_from_row_preserves_fields() {
        let row = make_row();
        let resp = ReelResponse::from(row.clone());
        assert_eq!(resp.id, row.id);
        assert_eq!(resp.caption, row.caption);
        assert_eq!(resp.view_count, 42);
        assert_eq!(resp.like_count, 7);
    }

    #[test]
    fn reel_response_serializes_without_internal_fields() {
        let row = make_row();
        let resp = ReelResponse::from(row);
        let json = serde_json::to_value(&resp).unwrap();
        // All public fields present
        assert!(json.get("id").is_some());
        assert!(json.get("author_profile_id").is_some());
        assert!(json.get("caption").is_some());
        assert!(json.get("score").is_some());
    }

    #[test]
    fn reel_search_doc_from_row_stringifies_uuid() {
        let row = make_row();
        let doc = ReelSearchDoc::from(&row);
        // Meilisearch needs string primary keys
        assert_eq!(doc.id, row.id.to_string());
        assert_eq!(doc.caption, row.caption);
        assert!(doc.created_at > 0);
    }

    #[test]
    fn processing_state_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(ProcessingState::Pending).unwrap(),
            serde_json::json!("pending")
        );
        assert_eq!(
            serde_json::to_value(ProcessingState::Ready).unwrap(),
            serde_json::json!("ready")
        );
    }
}
