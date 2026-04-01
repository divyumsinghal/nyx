//! Unit tests for the discover reranker.
//!
//! All tests are pure — no I/O, no database, no network.

use chrono::Utc;
use uuid::Uuid;

use uzume_discover::{
    models::explore::{ExploreItem, ExploreItemType},
    services::discover_ranker::{rerank_candidates, UserSignals},
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn item(score: f64) -> ExploreItem {
    ExploreItem {
        id: Uuid::now_v7(),
        item_type: ExploreItemType::Post,
        thumbnail_url: None,
        score,
    }
}

fn signals(hashtags: Vec<&str>, followed: Vec<Uuid>) -> UserSignals {
    UserSignals {
        liked_hashtags: hashtags.into_iter().map(String::from).collect(),
        followed_user_ids: followed,
        last_active: Utc::now(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

// #given an item that shares a hashtag with the viewer's liked content
// #when reranked
// #then its score is boosted by the hashtag affinity multiplier
#[test]
fn test_hashtag_match_boosts_score() {
    let a = item(100.0); // has matching hashtag
    let b = item(100.0); // no hashtag match

    let sigs = signals(vec!["travel"], vec![]);
    let result = rerank_candidates(
        vec![a, b],
        &sigs,
        &[vec!["travel".to_string()], vec![]],
        &[None, None],
        &[0.0, 0.0],
    );

    // Item with hashtag boost should appear first with a higher score
    assert!(
        result[0].score > result[1].score,
        "expected hashtag-boosted item to rank higher: {:?}",
        result.iter().map(|i| i.score).collect::<Vec<_>>()
    );
    // Boosted score should be base * 1.3
    assert!(
        (result[0].score - 130.0).abs() < 1e-6,
        "expected score 130.0, got {}",
        result[0].score
    );
}

// #given an item authored by someone the viewer follows
// #when reranked
// #then content from followed users receives a social graph boost
#[test]
fn test_followed_user_content_boosted() {
    let followed_id = Uuid::now_v7();
    let a = item(100.0); // from followed user
    let b = item(100.0); // from stranger

    let sigs = signals(vec![], vec![followed_id]);
    let result = rerank_candidates(
        vec![a, b],
        &sigs,
        &[vec![], vec![]],
        &[Some(followed_id), None],
        &[0.0, 0.0],
    );

    assert!(
        result[0].score > result[1].score,
        "expected followed-user content to rank higher"
    );
    // Social boost is 1.2x
    assert!(
        (result[0].score - 120.0).abs() < 1e-6,
        "expected score 120.0, got {}",
        result[0].score
    );
}

// #given an item older than 24 hours
// #when reranked
// #then old content is penalized relative to fresh content
#[test]
fn test_old_content_penalized() {
    let a = item(100.0); // 2 hours old → no decay
    let b = item(100.0); // 34 hours old → (34 - 24) * 0.1 = 1.0 → capped at 0.70 penalty → 0.30

    let sigs = signals(vec![], vec![]);
    let result = rerank_candidates(
        vec![a, b],
        &sigs,
        &[vec![], vec![]],
        &[None, None],
        &[2.0, 34.0],
    );

    assert!(
        result[0].score > result[1].score,
        "fresh content should outrank stale content"
    );
}

// #given two items where personalization changes the expected rank
// #when reranked
// #then the final ordering differs from the input ordering
#[test]
fn test_reranking_changes_order() {
    let followed_id = Uuid::now_v7();
    // `a` has a higher raw score but `b` gets a social boost
    // a: 200 * 1.0 = 200
    // b: 170 * 1.2 = 204  → b should win
    let a = item(200.0);
    let b = item(170.0); // authored by followed user

    let sigs = signals(vec![], vec![followed_id]);
    let result = rerank_candidates(
        vec![a, b],
        &sigs,
        &[vec![], vec![]],
        &[None, Some(followed_id)],
        &[0.0, 0.0],
    );

    // b (170 * 1.2 = 204) should beat a (200)
    assert!(
        result[0].score > 200.0,
        "boosted b should exceed 200.0, got {}",
        result[0].score
    );
    // Verify the boosted score is approximately 204
    assert!(
        (result[0].score - 204.0).abs() < 1e-6,
        "expected b's score ~204.0, got {}",
        result[0].score
    );
}

// #given an empty candidate list
// #when reranked
// #then an empty list is returned
#[test]
fn empty_input_returns_empty() {
    let sigs = signals(vec![], vec![]);
    let result = rerank_candidates(vec![], &sigs, &[], &[], &[]);
    assert!(result.is_empty());
}

// #given an item with combined hashtag and social boosts
// #when reranked
// #then both multipliers are applied (multiplicative, not additive)
#[test]
fn combined_boosts_are_multiplicative() {
    let followed_id = Uuid::now_v7();
    let a = item(100.0); // hashtag match + followed user

    let sigs = signals(vec!["rust"], vec![followed_id]);
    let result = rerank_candidates(
        vec![a],
        &sigs,
        &[vec!["rust".to_string()]],
        &[Some(followed_id)],
        &[0.0],
    );

    // Expected: 100 * 1.3 * 1.2 = 156.0
    assert!(
        (result[0].score - 156.0).abs() < 1e-6,
        "expected combined score 156.0, got {}",
        result[0].score
    );
}
