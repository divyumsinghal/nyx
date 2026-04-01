//! Highlight domain model.
//!
//! Highlights are curated collections of stories that persist beyond the 24h
//! window. The `owner_identity_id` is private and must never appear in HTTP
//! responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Raw database row for a highlight.
#[derive(Debug, Clone, FromRow)]
pub struct HighlightRow {
    pub id: Uuid,
    /// Internal Kratos identity UUID — never expose over HTTP.
    pub owner_identity_id: Uuid,
    pub owner_alias: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Data required to insert a new highlight.
#[derive(Debug, Clone)]
pub struct HighlightInsert {
    pub id: Uuid,
    pub owner_identity_id: Uuid,
    pub owner_alias: String,
    pub title: String,
}

/// Public-facing highlight representation returned in API responses.
///
/// `owner_identity_id` is intentionally absent.
#[derive(Debug, Clone, Serialize)]
pub struct HighlightResponse {
    pub id: Uuid,
    pub owner_alias: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<HighlightRow> for HighlightResponse {
    fn from(row: HighlightRow) -> Self {
        Self {
            id: row.id,
            owner_alias: row.owner_alias,
            title: row.title,
            cover_url: row.cover_url,
            created_at: row.created_at,
        }
    }
}

/// Request body for creating a highlight.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateHighlightRequest {
    #[validate(length(min = 1, max = 150, message = "title must be 1-150 characters"))]
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_response_excludes_identity_id() {
        let row = HighlightRow {
            id: Uuid::nil(),
            owner_identity_id: Uuid::nil(),
            owner_alias: "alice".to_string(),
            title: "Summer 2025".to_string(),
            cover_url: None,
            created_at: Utc::now(),
        };
        let response = HighlightResponse::from(row);
        let json = serde_json::to_value(&response).unwrap();
        assert!(
            json.get("owner_identity_id").is_none(),
            "owner_identity_id must not appear in HTTP responses"
        );
        assert_eq!(json["owner_alias"], "alice");
        assert_eq!(json["title"], "Summer 2025");
    }
}
