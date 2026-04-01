//! Story viewer domain model.
//!
//! Viewer records track which identities have seen a story. The
//! `viewer_identity_id` is the private Kratos UUID and must never appear in
//! HTTP responses. Only `viewer_alias` is returned to the story author.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// Raw database row for a story view.
#[derive(Debug, Clone, FromRow)]
pub struct StoryViewRow {
    pub story_id: Uuid,
    /// Internal Kratos identity UUID — never expose over HTTP.
    pub viewer_identity_id: Uuid,
    pub viewer_alias: String,
    pub viewed_at: DateTime<Utc>,
}

/// Public-facing viewer entry returned to the story author.
///
/// `viewer_identity_id` is intentionally absent.
#[derive(Debug, Clone, Serialize)]
pub struct ViewerResponse {
    pub viewer_alias: String,
    pub viewed_at: DateTime<Utc>,
}

impl From<StoryViewRow> for ViewerResponse {
    fn from(row: StoryViewRow) -> Self {
        Self {
            viewer_alias: row.viewer_alias,
            viewed_at: row.viewed_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_response_excludes_identity_id() {
        let row = StoryViewRow {
            story_id: Uuid::nil(),
            viewer_identity_id: Uuid::nil(),
            viewer_alias: "bob".to_string(),
            viewed_at: Utc::now(),
        };
        let response = ViewerResponse::from(row);
        let json = serde_json::to_value(&response).unwrap();
        assert!(
            json.get("viewer_identity_id").is_none(),
            "viewer_identity_id must not appear in HTTP responses"
        );
        assert_eq!(json["viewer_alias"], "bob");
    }
}
