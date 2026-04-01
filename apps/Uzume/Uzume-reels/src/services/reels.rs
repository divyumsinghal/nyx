//! Pure domain logic for reel operations.
//!
//! Helpers here are side-effect free and easy to unit test.

use nun::{Cursor, NyxError, PageResponse};
use uuid::Uuid;

use crate::models::reel::{ReelResponse, ReelRow};

/// Ensure the caller is the reel author.
///
/// # Errors
///
/// Returns `403 Forbidden` if `requester_profile_id` does not match
/// `reel.author_profile_id`.
pub fn ensure_reel_owner(reel: &ReelRow, requester_profile_id: Uuid) -> Result<(), NyxError> {
    if reel.author_profile_id == requester_profile_id {
        Ok(())
    } else {
        Err(NyxError::forbidden(
            "not_reel_owner",
            "Only the reel owner can perform this action",
        ))
    }
}

/// Build a paginated reel page sorted by score (algorithmic feed).
///
/// Uses `(score, id)` as the cursor key for stable keyset pagination.
pub fn build_reel_feed_page(rows: Vec<ReelRow>, limit: u16) -> PageResponse<ReelResponse> {
    PageResponse::from_overflowed(
        rows.into_iter().map(ReelResponse::from).collect(),
        limit,
        |reel| Cursor::score_id(reel.score, reel.id),
    )
}

/// Build a paginated reel page sorted by creation time (author profile page).
pub fn build_reel_time_page(rows: Vec<ReelRow>, limit: u16) -> PageResponse<ReelResponse> {
    PageResponse::from_overflowed(
        rows.into_iter().map(ReelResponse::from).collect(),
        limit,
        |reel| Cursor::timestamp_id(reel.created_at, reel.id),
    )
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn make_reel(score: f64) -> ReelRow {
        ReelRow {
            id: Uuid::now_v7(),
            author_profile_id: Uuid::now_v7(),
            caption: String::new(),
            hashtags: vec![],
            raw_key: "raw/key.mp4".to_string(),
            media_key: None,
            thumbnail_key: None,
            duration_ms: 15_000,
            processing_state: "ready".to_string(),
            audio_id: None,
            audio_start_ms: 0,
            view_count: 0,
            like_count: 0,
            comment_count: 0,
            share_count: 0,
            score,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn ensure_reel_owner_allows_owner() {
        let owner = Uuid::now_v7();
        let mut reel = make_reel(0.0);
        reel.author_profile_id = owner;
        assert!(ensure_reel_owner(&reel, owner).is_ok());
    }

    #[test]
    fn ensure_reel_owner_rejects_non_owner() {
        let owner = Uuid::now_v7();
        let other = Uuid::now_v7();
        let mut reel = make_reel(0.0);
        reel.author_profile_id = owner;
        let err = ensure_reel_owner(&reel, other).unwrap_err();
        assert_eq!(err.status_code(), 403);
        assert_eq!(err.code(), "not_reel_owner");
    }

    #[test]
    fn build_reel_feed_page_paginates_correctly() {
        let rows: Vec<ReelRow> = (0..21).map(|i| make_reel(f64::from(i))).collect();
        let page = build_reel_feed_page(rows, 20);
        assert_eq!(page.items.len(), 20);
        assert!(page.has_more);
        assert!(page.next_cursor.is_some());
    }

    #[test]
    fn build_reel_feed_page_no_more_pages() {
        let rows: Vec<ReelRow> = (0..10).map(|i| make_reel(f64::from(i))).collect();
        let page = build_reel_feed_page(rows, 20);
        assert_eq!(page.items.len(), 10);
        assert!(!page.has_more);
        assert!(page.next_cursor.is_none());
    }
}
