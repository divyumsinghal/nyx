//! Unit tests for `services::follow`.

use chrono::Utc;
use uuid::Uuid;
use uzume_profiles::{
    models::follow::FollowProfileRow,
    services::follow::{build_follow_page, validate_not_self_follow, FollowProfileResponse},
};

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

// ── validate_not_self_follow ──────────────────────────────────────────────────

#[test]
fn self_follow_returns_400() {
    let id = Uuid::now_v7();
    let err = validate_not_self_follow(id, id).unwrap_err();
    assert_eq!(err.status_code(), 400);
    assert_eq!(err.code(), "self_follow_not_allowed");
}

#[test]
fn different_ids_are_allowed() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    assert!(validate_not_self_follow(a, b).is_ok());
}

// ── build_follow_page ─────────────────────────────────────────────────────────

#[test]
fn page_fewer_than_limit_has_no_more() {
    let rows: Vec<FollowProfileRow> = (0..5).map(|_| make_row(Uuid::now_v7())).collect();
    let page = build_follow_page(rows, 20);
    assert_eq!(page.items.len(), 5);
    assert!(!page.has_more);
    assert!(page.next_cursor.is_none());
}

#[test]
fn page_exactly_at_limit_has_no_more() {
    let rows: Vec<FollowProfileRow> = (0..20).map(|_| make_row(Uuid::now_v7())).collect();
    let page = build_follow_page(rows, 20);
    assert_eq!(page.items.len(), 20);
    assert!(!page.has_more);
}

#[test]
fn page_one_over_limit_has_more() {
    let rows: Vec<FollowProfileRow> = (0..21).map(|_| make_row(Uuid::now_v7())).collect();
    let page = build_follow_page(rows, 20);
    assert_eq!(page.items.len(), 20);
    assert!(page.has_more);
    assert!(page.next_cursor.is_some());
}

#[test]
fn empty_rows_returns_empty_page() {
    let page = build_follow_page(vec![], 20);
    assert!(page.items.is_empty());
    assert!(!page.has_more);
    assert!(page.next_cursor.is_none());
}

// ── FollowProfileResponse from row ───────────────────────────────────────────

#[test]
fn follow_profile_response_maps_fields() {
    let id = Uuid::now_v7();
    let row = FollowProfileRow {
        id,
        alias: "alice".to_string(),
        display_name: "Alice".to_string(),
        avatar_url: Some("https://example.com/alice.jpg".to_string()),
        is_verified: true,
        created_at: Utc::now(),
    };

    let resp = FollowProfileResponse::from(row);
    assert_eq!(resp.id, id);
    assert_eq!(resp.alias, "alice");
    assert!(resp.is_verified);
}

#[test]
fn cursor_in_page_is_decodable() {
    use nun::Cursor;

    let rows: Vec<FollowProfileRow> = (0..21).map(|_| make_row(Uuid::now_v7())).collect();
    let page = build_follow_page(rows, 20);

    let cursor_str = page.next_cursor.as_ref().expect("cursor should be present");
    let cursor = Cursor::decode(cursor_str).expect("cursor should be decodable");
    let (_ts, _id) = cursor
        .as_timestamp_id()
        .expect("cursor should be timestamp+id");
}
