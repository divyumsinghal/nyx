//! Pure domain logic for feed operations.
//!
//! Functions in this module perform no I/O — they operate on types already
//! fetched from the database and return domain results. This makes them
//! straightforward to unit-test without database fixtures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::post::PostRow;
use nun::{Cursor, PageResponse, NyxError};

// ── Request / response domain types ──────────────────────────────────────────

/// HTTP request body for `POST /feed/posts`.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 2200, message = "Caption must be 1–2200 characters"))]
    pub caption: String,
}

/// The public-facing post representation returned to API clients.
///
/// SECURITY: `identity_id` is intentionally absent — only `author_alias` is
/// exposed to prevent global identity correlation across apps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostResponse {
    pub id: Uuid,
    pub author_alias: String,
    pub caption: String,
    pub like_count: i64,
    pub comment_count: i64,
    pub created_at: DateTime<Utc>,
}

impl From<PostRow> for PostResponse {
    fn from(row: PostRow) -> Self {
        Self {
            id: row.id,
            author_alias: row.author_alias,
            caption: row.caption,
            like_count: row.like_count,
            comment_count: row.comment_count,
            created_at: row.created_at,
        }
    }
}

// ── Domain logic ──────────────────────────────────────────────────────────────

/// Build a paginated post feed from a raw DB result set.
///
/// Implements the "fetch one extra" pattern: the caller passes `limit + 1`
/// rows from the database. If `rows.len() > limit`, there are more pages.
pub fn build_post_page(rows: Vec<PostRow>, limit: u16) -> PageResponse<PostResponse> {
    PageResponse::from_overflowed(
        rows.into_iter().map(PostResponse::from).collect(),
        limit,
        |post| Cursor::timestamp_id(post.created_at, post.id),
    )
}

/// Verify that the requesting identity is the author of the post.
///
/// Returns `Err` with a `403 Forbidden` if the check fails.
pub fn assert_is_author(
    post: &PostRow,
    requesting_identity_id: Uuid,
) -> Result<(), NyxError> {
    if post.identity_id == requesting_identity_id {
        Ok(())
    } else {
        Err(NyxError::forbidden(
            "not_post_author",
            "Only the post author can perform this action",
        ))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_post_row(identity_id: Uuid) -> PostRow {
        PostRow {
            id: Uuid::now_v7(),
            identity_id,
            author_alias: "alice".to_string(),
            caption: "Hello world".to_string(),
            like_count: 0,
            comment_count: 0,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn post_response_omits_identity_id() {
        let identity_id = Uuid::now_v7();
        let row = make_post_row(identity_id);
        let response = PostResponse::from(row);

        // Serialize and verify identity_id is not present
        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("identity_id").is_none(), "identity_id must not appear in PostResponse");
        assert!(json.get("author_alias").is_some());
    }

    #[test]
    fn assert_is_author_allows_owner() {
        let owner = Uuid::now_v7();
        let row = make_post_row(owner);
        assert!(assert_is_author(&row, owner).is_ok());
    }

    #[test]
    fn assert_is_author_rejects_stranger() {
        let owner = Uuid::now_v7();
        let stranger = Uuid::now_v7();
        let row = make_post_row(owner);
        let err = assert_is_author(&row, stranger).unwrap_err();
        assert_eq!(err.status_code(), 403);
        assert_eq!(err.code(), "not_post_author");
    }

    #[test]
    fn build_post_page_no_overflow() {
        let owner = Uuid::now_v7();
        let rows: Vec<PostRow> = (0..5).map(|_| make_post_row(owner)).collect();
        let page = build_post_page(rows, 20);
        assert_eq!(page.items.len(), 5);
        assert!(!page.has_more);
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn build_post_page_with_overflow() {
        let owner = Uuid::now_v7();
        // Simulate "fetch one extra" — 21 rows returned for limit=20
        let rows: Vec<PostRow> = (0..21).map(|_| make_post_row(owner)).collect();
        let page = build_post_page(rows, 20);
        assert_eq!(page.items.len(), 20);
        assert!(page.has_more);
        assert!(page.next_cursor.is_some());
    }

    #[test]
    fn create_post_request_validates_empty_caption() {
        use validator::Validate;
        let req = CreatePostRequest { caption: String::new() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_post_request_validates_long_caption() {
        use validator::Validate;
        let req = CreatePostRequest { caption: "x".repeat(2201) };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_post_request_accepts_valid_caption() {
        use validator::Validate;
        let req = CreatePostRequest { caption: "A valid caption".to_string() };
        assert!(req.validate().is_ok());
    }
}
