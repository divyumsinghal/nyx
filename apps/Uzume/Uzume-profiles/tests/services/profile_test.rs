//! Unit tests for `services::profile`.
//!
//! All tests here are pure logic — no database, no network, no I/O.

use chrono::Utc;
use uuid::Uuid;
use uzume_profiles::{
    models::profile::{ProfileRow, ProfileUpdate},
    services::profile::{
        check_profile_visibility, PatchProfileRequest, ProfileResponse,
    },
};
use validator::Validate;

fn profile_row(is_private: bool, identity_id: Uuid) -> ProfileRow {
    ProfileRow {
        id: Uuid::now_v7(),
        identity_id,
        alias: "alice".to_string(),
        display_name: "Alice".to_string(),
        bio: None,
        avatar_url: None,
        is_private,
        is_verified: false,
        follower_count: 0,
        following_count: 0,
        post_count: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// ── check_profile_visibility ──────────────────────────────────────────────────

#[test]
fn public_profile_is_visible_to_anonymous_user() {
    let owner = Uuid::now_v7();
    let profile = profile_row(false, owner);
    assert!(check_profile_visibility(&profile, None).is_ok());
}

#[test]
fn public_profile_is_visible_to_any_authenticated_user() {
    let owner = Uuid::now_v7();
    let profile = profile_row(false, owner);
    assert!(check_profile_visibility(&profile, Some(Uuid::now_v7())).is_ok());
}

#[test]
fn private_profile_is_visible_to_owner() {
    let owner = Uuid::now_v7();
    let profile = profile_row(true, owner);
    assert!(check_profile_visibility(&profile, Some(owner)).is_ok());
}

#[test]
fn private_profile_is_hidden_from_other_authenticated_user() {
    let owner = Uuid::now_v7();
    let stranger = Uuid::now_v7();
    let profile = profile_row(true, owner);
    let err = check_profile_visibility(&profile, Some(stranger)).unwrap_err();
    assert_eq!(err.status_code(), 403);
    assert_eq!(err.code(), "profile_private");
}

#[test]
fn private_profile_is_hidden_from_anonymous_user() {
    let owner = Uuid::now_v7();
    let profile = profile_row(true, owner);
    let err = check_profile_visibility(&profile, None).unwrap_err();
    assert_eq!(err.status_code(), 403);
}

// ── PatchProfileRequest validation ───────────────────────────────────────────

#[test]
fn patch_request_empty_display_name_is_invalid() {
    let req = PatchProfileRequest {
        display_name: Some(String::new()),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn patch_request_display_name_too_long_is_invalid() {
    let req = PatchProfileRequest {
        display_name: Some("a".repeat(81)),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn patch_request_bio_too_long_is_invalid() {
    let req = PatchProfileRequest {
        bio: Some("x".repeat(301)),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn patch_request_invalid_avatar_url_is_rejected() {
    let req = PatchProfileRequest {
        avatar_url: Some("not-a-url".to_string()),
        ..Default::default()
    };
    assert!(req.validate().is_err());
}

#[test]
fn patch_request_valid_fields_pass_validation() {
    let req = PatchProfileRequest {
        display_name: Some("Alice Smith".to_string()),
        bio: Some("Hello world".to_string()),
        avatar_url: Some("https://cdn.example.com/alice.jpg".to_string()),
        is_private: Some(false),
    };
    assert!(req.validate().is_ok());
}

#[test]
fn patch_request_all_none_passes_validation() {
    let req = PatchProfileRequest::default();
    assert!(req.validate().is_ok());
}

#[test]
fn patch_request_converts_to_profile_update() {
    let req = PatchProfileRequest {
        display_name: Some("Bob".to_string()),
        bio: None,
        avatar_url: None,
        is_private: Some(true),
    };
    let update: ProfileUpdate = req.into_profile_update();
    assert_eq!(update.display_name.as_deref(), Some("Bob"));
    assert!(update.bio.is_none());
    assert_eq!(update.is_private, Some(true));
}

// ── ProfileResponse from row ──────────────────────────────────────────────────

#[test]
fn profile_response_maps_all_fields() {
    let id = Uuid::now_v7();
    let identity_id = Uuid::now_v7();
    let now = Utc::now();
    let row = ProfileRow {
        id,
        identity_id,
        alias: "bob".to_string(),
        display_name: "Bob".to_string(),
        bio: Some("A bio".to_string()),
        avatar_url: Some("https://example.com/bob.jpg".to_string()),
        is_private: true,
        is_verified: true,
        follower_count: 42,
        following_count: 10,
        post_count: 7,
        created_at: now,
        updated_at: now,
    };

    let resp = ProfileResponse::from(row);
    assert_eq!(resp.id, id);
    assert_eq!(resp.alias, "bob");
    assert_eq!(resp.display_name, "Bob");
    assert_eq!(resp.bio.as_deref(), Some("A bio"));
    assert!(resp.is_private);
    assert!(resp.is_verified);
    assert_eq!(resp.follower_count, 42);
    assert_eq!(resp.following_count, 10);
    assert_eq!(resp.post_count, 7);
}
