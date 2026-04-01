//! Story domain model.
//!
//! # Privacy contract
//!
//! `author_identity_id` is the internal Kratos UUID and MUST NEVER appear in
//! any HTTP response. Only `author_alias` (the app-scoped alias) is visible
//! to clients.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Story lifecycle states.
///
/// Stories begin as [`StoryStatus::Pending`] while media is being processed by
/// Oya. Once the `Uzume.media.processed` event fires, they transition to
/// [`StoryStatus::Active`]. After 24 hours the expiry worker transitions them
/// to [`StoryStatus::Expired`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "story_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum StoryStatus {
    /// Media upload accepted; processing in progress.
    Pending,
    /// Media ready and within 24-hour visibility window.
    Active,
    /// 24-hour window has elapsed; story is no longer visible in feeds.
    Expired,
}

/// Media type carried by a story.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "media_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// A static image (JPEG, PNG, WebP).
    Image,
    /// A video clip.
    Video,
}

/// Raw database row for a story.
///
/// This type is `sqlx::FromRow` — use it only inside the `queries` module.
/// The `author_identity_id` field must be stripped before returning data to
/// HTTP clients.
#[derive(Debug, Clone, FromRow)]
pub struct StoryRow {
    pub id: Uuid,
    /// Internal Kratos identity UUID — never expose over HTTP.
    pub author_identity_id: Uuid,
    pub author_alias: String,
    pub media_url: Option<String>,
    pub media_type: MediaType,
    pub duration_secs: Option<i32>,
    pub status: StoryStatus,
    pub view_count: i64,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Data required to insert a new story row.
#[derive(Debug, Clone)]
pub struct StoryInsert {
    pub id: Uuid,
    pub author_identity_id: Uuid,
    pub author_alias: String,
    pub media_type: MediaType,
    pub duration_secs: Option<i32>,
}

/// Public-facing story representation returned in API responses.
///
/// Note: `author_identity_id` is intentionally absent.
#[derive(Debug, Clone, Serialize)]
pub struct StoryResponse {
    pub id: Uuid,
    pub author_alias: String,
    pub media_url: Option<String>,
    pub media_type: MediaType,
    pub duration_secs: Option<i32>,
    pub status: StoryStatus,
    pub view_count: i64,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<StoryRow> for StoryResponse {
    fn from(row: StoryRow) -> Self {
        Self {
            id: row.id,
            author_alias: row.author_alias,
            media_url: row.media_url,
            media_type: row.media_type,
            duration_secs: row.duration_secs,
            status: row.status,
            view_count: row.view_count,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}

/// Request body for creating a story.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateStoryRequest {
    /// MIME type of the media to upload (e.g. `"image/jpeg"`, `"video/mp4"`).
    pub content_type: String,
    /// Exact byte size of the media file to upload.
    pub content_length: u64,
    /// For videos: intended duration in seconds.
    pub duration_secs: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_response_excludes_identity_id() {
        let row = StoryRow {
            id: Uuid::nil(),
            author_identity_id: Uuid::nil(),
            author_alias: "alice".to_string(),
            media_url: None,
            media_type: MediaType::Image,
            duration_secs: None,
            status: StoryStatus::Active,
            view_count: 0,
            expires_at: Utc::now(),
            created_at: Utc::now(),
        };
        let response = StoryResponse::from(row);
        let json = serde_json::to_value(&response).unwrap();
        assert!(
            json.get("author_identity_id").is_none(),
            "author_identity_id must not appear in HTTP responses"
        );
        assert_eq!(json["author_alias"], "alice");
    }

    #[test]
    fn story_status_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(StoryStatus::Active).unwrap(),
            serde_json::json!("active")
        );
        assert_eq!(
            serde_json::to_value(StoryStatus::Pending).unwrap(),
            serde_json::json!("pending")
        );
        assert_eq!(
            serde_json::to_value(StoryStatus::Expired).unwrap(),
            serde_json::json!("expired")
        );
    }

    #[test]
    fn media_type_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(MediaType::Image).unwrap(),
            serde_json::json!("image")
        );
        assert_eq!(
            serde_json::to_value(MediaType::Video).unwrap(),
            serde_json::json!("video")
        );
    }
}
