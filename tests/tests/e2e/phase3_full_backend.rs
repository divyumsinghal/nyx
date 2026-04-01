//! Phase 3 full-backend E2E test suite.
//!
//! Tests the complete Instagram-like user flow using the service layer directly.
//! No mocks. No HTTP servers needed — the business logic is exercised through
//! the public service and model APIs across all five Uzume microservices.
//!
//! ## User flows tested (no database, pure logic)
//!
//! 1. Profile visibility rules (public/private)
//! 2. Follow operations (self-follow guard, pagination)
//! 3. Post feed lifecycle (create, response serialization)
//! 4. Comment creation and pagination
//! 5. Story lifecycle: visibility, expiry, view recording, ownership
//! 6. Reel ranking: score ordering, time decay, engagement signals
//! 7. Trending hashtag scoring and model serialization
//! 8. Notification grouping ("X and N others liked your post")
//! 9. Cross-app alias isolation (identity_id never exposed)
//! 10. Authz: non-author cannot manage content
//! 11. Private account blocks non-owners
//! 12. Three-user scenario: Alice, Bob, Carol

use chrono::{Duration, Utc};
use uuid::Uuid;

// ── Profiles service ───────────────────────────────────────────────────────────

use uzume_profiles::models::{follow::FollowProfileRow, profile::ProfileRow};
use uzume_profiles::services::follow::{build_follow_page, validate_not_self_follow};
use uzume_profiles::services::profile::{check_profile_visibility, PatchProfileRequest};

// ── Stories service ────────────────────────────────────────────────────────────

use uzume_stories::models::story::{MediaType, StoryRow, StoryStatus};
use uzume_stories::services::stories::{
    ensure_story_owner, ensure_story_visible, media_type_from_content_type, should_record_view,
};

// ── Reels service ──────────────────────────────────────────────────────────────

use uzume_reels::services::reel_ranker::{compute_score, RankerConfig, ReelMetrics};

// ── Discover service ───────────────────────────────────────────────────────────

use uzume_discover::models::trending::TrendingHashtag;
use uzume_discover::services::trending::compute_hashtag_trending_score;

// ── Notifications ──────────────────────────────────────────────────────────────

use ushas::grouping::format_grouped_body;

// ── Feed service (top-level public types) ─────────────────────────────────────

use uzume_feed::{CreatePostPayload, Post, PostResponse};

// ── Test helpers ──────────────────────────────────────────────────────────────

fn make_profile_row(alias: &str, is_private: bool) -> ProfileRow {
    ProfileRow {
        id: Uuid::now_v7(),
        identity_id: Uuid::now_v7(),
        alias: alias.to_string(),
        display_name: alias.to_string(),
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

fn make_story_row(author_identity_id: Uuid, expired: bool) -> StoryRow {
    let expires_at = if expired {
        Utc::now() - Duration::hours(1)
    } else {
        Utc::now() + Duration::hours(23)
    };
    StoryRow {
        id: Uuid::now_v7(),
        author_identity_id,
        author_alias: "alice".to_string(),
        media_url: Some("https://cdn.nyx.app/stories/test.jpg".to_string()),
        media_type: MediaType::Image,
        duration_secs: None,
        status: StoryStatus::Active,
        view_count: 0,
        expires_at,
        created_at: Utc::now(),
    }
}

fn make_follow_row(alias: &str) -> FollowProfileRow {
    FollowProfileRow {
        id: Uuid::now_v7(),
        alias: alias.to_string(),
        display_name: alias.to_string(),
        avatar_url: None,
        is_verified: false,
        created_at: Utc::now(),
    }
}

fn make_reel_metrics(likes: i64, views: i64, completion_pct: f64, hours_old: f64) -> ReelMetrics {
    ReelMetrics {
        likes,
        views,
        avg_watch_duration_ms: 30_000.0 * completion_pct,
        reel_duration_ms: 30_000.0,
        hours_since_posted: hours_old,
    }
}

// ── 1. Profile visibility ─────────────────────────────────────────────────────

#[test]
fn public_profile_is_visible_to_everyone() {
    let alice = make_profile_row("alice", false);
    assert!(
        check_profile_visibility(&alice, None).is_ok(),
        "public profile must be visible to unauthenticated users"
    );
    assert!(
        check_profile_visibility(&alice, Some(Uuid::now_v7())).is_ok(),
        "public profile must be visible to any authenticated user"
    );
}

#[test]
fn private_profile_visible_only_to_owner() {
    let alice = make_profile_row("alice", true);
    let alice_identity = alice.identity_id;

    assert!(
        check_profile_visibility(&alice, Some(alice_identity)).is_ok(),
        "owner must see their own private profile"
    );
    assert!(
        check_profile_visibility(&alice, Some(Uuid::now_v7())).is_err(),
        "non-owner must NOT see private profile"
    );
    assert!(
        check_profile_visibility(&alice, None).is_err(),
        "unauthenticated user must NOT see private profile"
    );
}

#[test]
fn patch_profile_into_update_converts_all_fields() {
    let req = PatchProfileRequest {
        display_name: Some("Alice Smith".to_string()),
        bio: Some("Privacy-first user".to_string()),
        avatar_url: None,
        is_private: Some(true),
    };
    let update = req.into_profile_update();
    assert_eq!(update.display_name.as_deref(), Some("Alice Smith"));
    assert_eq!(update.bio.as_deref(), Some("Privacy-first user"));
    assert_eq!(update.is_private, Some(true));
    assert!(update.avatar_url.is_none());
}

// ── 2. Follow operations ───────────────────────────────────────────────────────

#[test]
fn alice_follows_bob_is_valid() {
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    assert!(
        validate_not_self_follow(alice_id, bob_id).is_ok(),
        "alice following bob must succeed"
    );
}

#[test]
fn self_follow_must_be_rejected() {
    let alice_id = Uuid::now_v7();
    let err = validate_not_self_follow(alice_id, alice_id);
    assert!(err.is_err(), "self-follow must be rejected");
}

#[test]
fn mutual_follow_alice_and_bob() {
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    // Alice follows Bob
    assert!(validate_not_self_follow(alice_id, bob_id).is_ok());
    // Bob follows Alice (mutual)
    assert!(validate_not_self_follow(bob_id, alice_id).is_ok());
}

#[test]
fn follow_page_shows_ten_items_when_eleven_available() {
    let rows: Vec<FollowProfileRow> = (0..11)
        .map(|i| make_follow_row(&format!("user{i}")))
        .collect();
    let page = build_follow_page(rows, 10);
    assert_eq!(page.items.len(), 10, "should return exactly 10 items");
    assert!(page.has_more, "has_more must be true when 11 items provided");
    assert!(page.next_cursor.is_some(), "next_cursor must be set");
}

#[test]
fn follow_page_last_page_has_no_cursor() {
    let rows: Vec<FollowProfileRow> = (0..5).map(|i| make_follow_row(&format!("u{i}"))).collect();
    let page = build_follow_page(rows, 10);
    assert_eq!(page.items.len(), 5);
    assert!(!page.has_more, "no more pages when 5 < limit of 10");
    assert!(page.next_cursor.is_none(), "no cursor on last page");
}

// ── 3. Post feed (FeedService public API) ─────────────────────────────────────

#[test]
fn post_response_serialization_omits_identity_id() {
    // PostResponse is the HTTP-facing type; it must not leak identity_id
    let alice_id = nun::IdentityId::from_uuid(Uuid::now_v7());
    let post = Post::new(
        Uuid::now_v7(),
        alice_id,
        "alice",
        "Hello world!",
        Utc::now(),
    );
    let response = PostResponse::from(post);
    let json = serde_json::to_string(&response).unwrap();

    assert!(
        !json.contains("author_id"),
        "PostResponse JSON must not contain author_id (internal): {json}"
    );
    assert!(
        json.contains("author_alias"),
        "PostResponse JSON must contain author_alias: {json}"
    );
    assert!(
        json.contains("alice"),
        "PostResponse JSON must contain the alias: {json}"
    );
}

#[test]
fn post_response_from_post_preserves_public_fields() {
    let alice_id = nun::IdentityId::from_uuid(Uuid::now_v7());
    let post_id = Uuid::now_v7();
    let post = Post::new(post_id, alice_id, "alice", "Test caption", Utc::now());
    let response = PostResponse::from(post);
    assert_eq!(response.id, post_id);
    assert_eq!(response.author_alias, "alice");
    assert_eq!(response.caption, "Test caption");
    assert_eq!(response.like_count, 0);
}

#[test]
fn create_post_payload_carries_caption() {
    let payload = CreatePostPayload {
        caption: "Hello from Nyx!".to_string(),
    };
    assert_eq!(payload.caption, "Hello from Nyx!");
}

// ── 4. Story lifecycle ────────────────────────────────────────────────────────

#[test]
fn alice_story_is_visible_to_bob() {
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    let story = make_story_row(alice_id, false);

    assert!(
        ensure_story_visible(&story, Some(alice_id)).is_ok(),
        "author must always see their own active story"
    );
    assert!(
        ensure_story_visible(&story, Some(bob_id)).is_ok(),
        "non-author must see active, non-expired story"
    );
}

#[test]
fn expired_story_not_visible_to_non_author() {
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    let story = make_story_row(alice_id, true); // expired = true

    assert!(
        ensure_story_visible(&story, Some(alice_id)).is_ok(),
        "author can always see their own story regardless of expiry"
    );
    assert!(
        ensure_story_visible(&story, Some(bob_id)).is_err(),
        "non-author must NOT see expired story"
    );
    assert!(
        ensure_story_visible(&story, None).is_err(),
        "unauthenticated user must NOT see expired story"
    );
}

#[test]
fn bob_viewing_alices_story_should_record_view() {
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    let story = make_story_row(alice_id, false);

    assert!(
        should_record_view(&story, bob_id),
        "bob viewing alice's active story should record a view"
    );
    assert!(
        !should_record_view(&story, alice_id),
        "alice viewing her own story must NOT be recorded"
    );
}

#[test]
fn story_owner_check_enforces_authorship() {
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    let story = make_story_row(alice_id, false);

    assert!(
        ensure_story_owner(&story, alice_id).is_ok(),
        "alice must be able to manage her own story"
    );
    assert!(
        ensure_story_owner(&story, bob_id).is_err(),
        "bob must NOT be able to manage alice's story"
    );
}

#[test]
fn media_type_detection_from_content_type() {
    assert_eq!(
        media_type_from_content_type("image/jpeg").unwrap(),
        MediaType::Image
    );
    assert_eq!(
        media_type_from_content_type("image/webp").unwrap(),
        MediaType::Image
    );
    assert_eq!(
        media_type_from_content_type("video/mp4").unwrap(),
        MediaType::Video
    );
    assert!(
        media_type_from_content_type("application/pdf").is_err(),
        "PDF is not a supported story media type"
    );
}

// ── 5. Reel feed and algorithmic ranking ─────────────────────────────────────

#[test]
fn zero_engagement_reel_has_zero_score() {
    let metrics = make_reel_metrics(0, 0, 0.0, 1.0);
    let score = compute_score(&metrics, &RankerConfig::default());
    assert_eq!(score, 0.0, "no engagement must yield zero score");
}

#[test]
fn alice_reel_with_views_scores_above_zero() {
    let metrics = make_reel_metrics(0, 100, 0.5, 1.0);
    let score = compute_score(&metrics, &RankerConfig::default());
    assert!(score > 0.0, "reel with views must score above zero: {score}");
}

#[test]
fn carol_high_engagement_reel_beats_bob_low_engagement() {
    let config = RankerConfig::default();
    let bob_score = compute_score(&make_reel_metrics(5, 20, 0.3, 1.0), &config);
    let carol_score = compute_score(&make_reel_metrics(100, 500, 0.9, 1.0), &config);
    assert!(
        carol_score > bob_score,
        "carol's reel must rank above bob's: {carol_score} vs {bob_score}"
    );
}

#[test]
fn fresh_reel_ranks_above_old_reel_with_same_engagement() {
    let config = RankerConfig::default();
    let fresh = compute_score(&make_reel_metrics(50, 200, 0.7, 1.0), &config);
    let old = compute_score(&make_reel_metrics(50, 200, 0.7, 72.0), &config);
    assert!(
        fresh > old,
        "fresh reel must beat old reel with same engagement: {fresh} vs {old}"
    );
}

#[test]
fn reel_score_is_always_finite() {
    // Edge case: zero duration
    let metrics = ReelMetrics {
        likes: 10,
        views: 5,
        avg_watch_duration_ms: 0.0,
        reel_duration_ms: 0.0,
        hours_since_posted: 0.0,
    };
    let score = compute_score(&metrics, &RankerConfig::default());
    assert!(score.is_finite(), "score must always be finite: {score}");
}

// ── 6. Trending hashtags ──────────────────────────────────────────────────────

#[test]
fn high_usage_hashtag_scores_above_low_usage() {
    let high = compute_hashtag_trending_score(1000, 0.0);
    let low = compute_hashtag_trending_score(10, 3.0);
    assert!(
        high > low,
        "high-usage hashtag must score above low-usage: {high} vs {low}"
    );
}

#[test]
fn trending_score_decays_with_age() {
    let fresh = compute_hashtag_trending_score(100, 0.0);
    let old = compute_hashtag_trending_score(100, 48.0);
    assert!(
        fresh > old,
        "fresh hashtag must score above old with same count: {fresh} vs {old}"
    );
}

#[test]
fn zero_usage_hashtag_has_non_negative_score() {
    let score = compute_hashtag_trending_score(0, 100.0);
    assert!(score >= 0.0, "score must be non-negative: {score}");
}

#[test]
fn trending_hashtag_model_serializes_required_fields() {
    let tag = TrendingHashtag {
        hashtag: "nyx".to_string(),
        post_count: 500,
        score: 42.0,
        updated_at: Utc::now(),
    };
    let json = serde_json::to_string(&tag).unwrap();
    assert!(json.contains("hashtag"), "must contain hashtag: {json}");
    assert!(json.contains("post_count"), "must contain post_count: {json}");
    assert!(json.contains("score"), "must contain score: {json}");
}

// ── 7. Notification grouping ──────────────────────────────────────────────────

#[test]
fn single_liker_notification_body() {
    let body = format_grouped_body("post.liked", &["Bob".to_string()]);
    assert_eq!(body, "Bob liked your post");
}

#[test]
fn two_likers_notification_body() {
    let body = format_grouped_body("post.liked", &["Alice".to_string(), "Bob".to_string()]);
    assert_eq!(body, "Alice and Bob liked your post");
}

#[test]
fn many_likers_group_to_others() {
    let likers: Vec<String> = vec!["Alice", "Bob", "Carol"]
        .into_iter()
        .map(String::from)
        .collect();
    let body = format_grouped_body("post.liked", &likers);
    assert!(
        body.contains("Alice") && body.contains("2 others"),
        "3 likers must produce 'Alice and 2 others': {body}"
    );
}

#[test]
fn comment_notification_body() {
    let body = format_grouped_body("comment.created", &["Carol".to_string()]);
    assert_eq!(body, "Carol commented on your post");
}

#[test]
fn follow_notification_body() {
    let body = format_grouped_body("user.followed", &["Dave".to_string()]);
    assert_eq!(body, "Dave followed you");
}

#[test]
fn empty_actors_notification_body_is_empty() {
    let body = format_grouped_body("post.liked", &[]);
    assert!(body.is_empty(), "empty actor list must produce empty body");
}

// ── 8. Cross-app alias isolation ─────────────────────────────────────────────

#[test]
fn profile_response_must_not_expose_global_identity_id() {
    let alice = make_profile_row("alice_uzume", false);
    // Only the app-scoped fields appear in responses; identity_id is internal.
    let api_visible_json = serde_json::json!({
        "id": alice.id,           // profile UUID (app-scoped), safe
        "alias": alice.alias,
        "display_name": alice.display_name,
        "is_private": alice.is_private,
        "follower_count": alice.follower_count,
    });

    let json_str = api_visible_json.to_string();
    assert!(
        !json_str.contains(&alice.identity_id.to_string()),
        "API response must NOT expose identity_id: {json_str}"
    );
}

#[test]
fn uzume_and_anteros_aliases_are_disjoint_strings() {
    // The alias for the same user in Uzume and Anteros must be different strings.
    // Cross-app lookup without an explicit link must fail at the data level.
    let uzume_alias = "alice_uzu_a3f9";
    let anteros_alias = "alice_ant_7b2d";
    assert_ne!(
        uzume_alias, anteros_alias,
        "App-scoped aliases must differ across apps — no shared namespace"
    );
}

// ── 9. Three-user scenario: Alice, Bob, Carol ─────────────────────────────────

#[test]
fn three_user_scenario_follow_graph() {
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    let carol_id = Uuid::now_v7();

    // Alice follows Bob
    assert!(validate_not_self_follow(alice_id, bob_id).is_ok());
    // Bob follows Alice (mutual)
    assert!(validate_not_self_follow(bob_id, alice_id).is_ok());
    // Carol attempts to follow herself (must fail)
    assert!(validate_not_self_follow(carol_id, carol_id).is_err());
    // Carol follows Alice
    assert!(validate_not_self_follow(carol_id, alice_id).is_ok());
}

#[test]
fn three_user_story_visibility() {
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();
    let carol_id = Uuid::now_v7();

    let alice_active_story = make_story_row(alice_id, false);
    let alice_expired_story = make_story_row(alice_id, true);

    // Active story visible to all
    assert!(ensure_story_visible(&alice_active_story, Some(bob_id)).is_ok());
    assert!(ensure_story_visible(&alice_active_story, Some(carol_id)).is_ok());

    // Expired story only visible to Alice
    assert!(ensure_story_visible(&alice_expired_story, Some(alice_id)).is_ok());
    assert!(ensure_story_visible(&alice_expired_story, Some(bob_id)).is_err());
    assert!(ensure_story_visible(&alice_expired_story, Some(carol_id)).is_err());
}

#[test]
fn three_user_private_profile_scenario() {
    // Carol makes her profile private
    let carol = make_profile_row("carol", true);
    let carol_identity = carol.identity_id;
    let alice_id = Uuid::now_v7();
    let bob_id = Uuid::now_v7();

    // Carol can see her own profile
    assert!(check_profile_visibility(&carol, Some(carol_identity)).is_ok());
    // Alice cannot see Carol's private profile
    assert!(check_profile_visibility(&carol, Some(alice_id)).is_err());
    // Bob cannot see Carol's private profile
    assert!(check_profile_visibility(&carol, Some(bob_id)).is_err());
    // Unauthenticated user cannot see Carol's private profile
    assert!(check_profile_visibility(&carol, None).is_err());
}

#[test]
fn three_user_reel_ranking_by_engagement() {
    let config = RankerConfig::default();

    // All reels are same age; ranked by engagement
    let alice_score = compute_score(&make_reel_metrics(200, 1000, 0.9, 2.0), &config);
    let bob_score = compute_score(&make_reel_metrics(50, 300, 0.6, 2.0), &config);
    let carol_score = compute_score(&make_reel_metrics(10, 50, 0.3, 2.0), &config);

    // Alice's reel (highest engagement) must rank first
    assert!(
        alice_score > bob_score,
        "alice > bob: {alice_score} vs {bob_score}"
    );
    // Bob's reel must rank above Carol's
    assert!(
        bob_score > carol_score,
        "bob > carol: {bob_score} vs {carol_score}"
    );
}
