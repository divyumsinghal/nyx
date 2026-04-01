//! Pure domain logic for follow operations.

use crate::models::follow::FollowProfileRow;
use nun::{Cursor, NyxError, PageResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Response types ───────────────────────────────────────────────────────────

/// Compact profile summary returned inside follower/following lists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FollowProfileResponse {
    pub id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
}

impl From<FollowProfileRow> for FollowProfileResponse {
    fn from(row: FollowProfileRow) -> Self {
        Self {
            id: row.id,
            alias: row.alias,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            is_verified: row.is_verified,
        }
    }
}

// ── Domain logic ─────────────────────────────────────────────────────────────

/// Guard: prevent a user from following themselves.
pub fn validate_not_self_follow(
    follower_id: Uuid,
    followee_id: Uuid,
) -> Result<(), NyxError> {
    if follower_id == followee_id {
        return Err(NyxError::bad_request(
            "self_follow_not_allowed",
            "You cannot follow yourself",
        ));
    }
    Ok(())
}

/// Convert a raw database page of follower rows into a paginated API response.
///
/// Uses the "fetch one extra" pattern: rows contains `limit + 1` entries.
/// The last item's `(created_at, id)` pair is encoded as the next cursor.
pub fn build_follow_page(
    rows: Vec<FollowProfileRow>,
    limit: u16,
) -> PageResponse<FollowProfileResponse> {
    let limit_usize = usize::from(limit);
    let has_more = rows.len() > limit_usize;

    let mut rows = rows;
    if has_more {
        rows.truncate(limit_usize);
    }

    let next_cursor = if has_more {
        rows.last()
            .map(|r| Cursor::timestamp_id(r.created_at, r.id).encode())
    } else {
        None
    };

    let items = rows.into_iter().map(FollowProfileResponse::from).collect();

    PageResponse {
        items,
        next_cursor,
        has_more,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_row(id: Uuid) -> FollowProfileRow {
        FollowProfileRow {
            id,
            alias: format!("user_{id}"),
            display_name: format!("User {id}"),
            avatar_url: None,
            is_verified: false,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn self_follow_is_rejected() {
        let id = Uuid::now_v7();
        let err = validate_not_self_follow(id, id).unwrap_err();
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.code(), "self_follow_not_allowed");
    }

    #[test]
    fn different_users_can_follow() {
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        assert!(validate_not_self_follow(a, b).is_ok());
    }

    #[test]
    fn follow_profile_response_from_row() {
        let id = Uuid::now_v7();
        let row = make_row(id);
        let resp = FollowProfileResponse::from(row);
        assert_eq!(resp.id, id);
        assert!(!resp.is_verified);
    }

    #[test]
    fn build_follow_page_no_more() {
        let ids: Vec<Uuid> = (0..5).map(|_| Uuid::now_v7()).collect();
        let rows: Vec<FollowProfileRow> = ids.iter().copied().map(make_row).collect();
        let page = build_follow_page(rows, 20);
        assert_eq!(page.items.len(), 5);
        assert!(!page.has_more);
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn build_follow_page_has_more() {
        // 21 rows, limit 20 → has_more = true, 20 items returned
        let rows: Vec<FollowProfileRow> = (0..21).map(|_| make_row(Uuid::now_v7())).collect();
        let page = build_follow_page(rows, 20);
        assert_eq!(page.items.len(), 20);
        assert!(page.has_more);
        assert!(page.next_cursor.is_some());
    }
}
