//! Pure domain logic for post interaction operations (likes and comments).
//!
//! Functions here are pure — they accept already-fetched database rows and
//! return domain results without performing any I/O.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::comment::CommentRow;
use nun::{Cursor, PageResponse};

// ── Comment request / response types ─────────────────────────────────────────

/// HTTP request body for `POST /feed/posts/:id/comments`.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCommentRequest {
    #[validate(length(min = 1, max = 500, message = "Comment must be 1–500 characters"))]
    pub content: String,
}

/// The public-facing comment representation returned to API clients.
///
/// SECURITY: `author_identity_id` is intentionally absent — only
/// `author_alias` is exposed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentResponse {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_alias: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl From<CommentRow> for CommentResponse {
    fn from(row: CommentRow) -> Self {
        Self {
            id: row.id,
            post_id: row.post_id,
            author_alias: row.author_alias,
            content: row.content,
            created_at: row.created_at,
        }
    }
}

// ── Like response type ────────────────────────────────────────────────────────

/// Response for like/unlike endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeResponse {
    pub post_id: Uuid,
    pub liked: bool,
    pub like_count: i64,
}

// ── Domain logic ──────────────────────────────────────────────────────────────

/// Build a paginated comment page from a raw DB result set.
///
/// Implements the "fetch one extra" pattern. Comments are sorted oldest-first
/// so the cursor advances by `(created_at, id)` ascending.
pub fn build_comment_page(rows: Vec<CommentRow>, limit: u16) -> PageResponse<CommentResponse> {
    PageResponse::from_overflowed(
        rows.into_iter().map(CommentResponse::from).collect(),
        limit,
        |comment| Cursor::timestamp_id(comment.created_at, comment.id),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_comment_row(post_id: Uuid) -> CommentRow {
        CommentRow {
            id: Uuid::now_v7(),
            post_id,
            author_identity_id: Uuid::now_v7(),
            author_alias: "bob".to_string(),
            content: "Great post!".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn comment_response_omits_identity_id() {
        let post_id = Uuid::now_v7();
        let row = make_comment_row(post_id);
        let response = CommentResponse::from(row);

        let json = serde_json::to_value(&response).unwrap();
        assert!(
            json.get("author_identity_id").is_none(),
            "author_identity_id must not appear in CommentResponse"
        );
        assert!(json.get("author_alias").is_some());
    }

    #[test]
    fn build_comment_page_no_overflow() {
        let post_id = Uuid::now_v7();
        let rows: Vec<CommentRow> = (0..3).map(|_| make_comment_row(post_id)).collect();
        let page = build_comment_page(rows, 20);
        assert_eq!(page.items.len(), 3);
        assert!(!page.has_more);
    }

    #[test]
    fn build_comment_page_with_overflow() {
        let post_id = Uuid::now_v7();
        let rows: Vec<CommentRow> = (0..21).map(|_| make_comment_row(post_id)).collect();
        let page = build_comment_page(rows, 20);
        assert_eq!(page.items.len(), 20);
        assert!(page.has_more);
        assert!(page.next_cursor.is_some());
    }

    #[test]
    fn create_comment_request_validates_empty_content() {
        use validator::Validate;
        let req = CreateCommentRequest { content: String::new() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_comment_request_validates_long_content() {
        use validator::Validate;
        let req = CreateCommentRequest { content: "x".repeat(501) };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_comment_request_accepts_valid_content() {
        use validator::Validate;
        let req = CreateCommentRequest { content: "Nice!".to_string() };
        assert!(req.validate().is_ok());
    }
}
