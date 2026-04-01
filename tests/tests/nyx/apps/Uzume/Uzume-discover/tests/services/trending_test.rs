//! Unit tests for trending score computation.
//!
//! All tests are pure — no I/O, no database, no mocks needed.

use uzume_discover::services::trending::{
    compute_hashtag_trending_score, rank_trending_items, HasScore,
};

// ── Helper ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct ScoredItem {
    name: &'static str,
    score: f64,
}

impl HasScore for ScoredItem {
    fn score(&self) -> f64 {
        self.score
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

// #given two hashtags with the same usage count
// #when one was used more recently
// #then the more recent hashtag scores higher
#[test]
fn test_more_recent_hashtag_scores_higher() {
    let recent = compute_hashtag_trending_score(100, 1.0);
    let older = compute_hashtag_trending_score(100, 12.0);
    assert!(
        recent > older,
        "recent score {recent} should exceed older score {older}"
    );
}

// #given a hashtag with many recent uses
// #when compared to a stale popular hashtag with more total uses
// #then the viral recent hashtag beats the stale one
#[test]
fn test_viral_hashtag_beats_stale_popular_one() {
    // 400 uses in the last hour vs 2000 uses from 36 hours ago
    let viral = compute_hashtag_trending_score(400, 1.0);
    let stale_popular = compute_hashtag_trending_score(2000, 36.0);
    assert!(
        viral > stale_popular,
        "viral score {viral} should beat stale_popular score {stale_popular}"
    );
}

// #given a list of items with known scores
// #when ranked
// #then the output list is sorted highest score first
#[test]
fn test_ranking_preserves_order() {
    let items = vec![
        ScoredItem {
            name: "low",
            score: 1.0,
        },
        ScoredItem {
            name: "high",
            score: 100.0,
        },
        ScoredItem {
            name: "mid",
            score: 50.0,
        },
    ];

    let ranked = rank_trending_items(items);

    assert_eq!(ranked[0].name, "high");
    assert_eq!(ranked[1].name, "mid");
    assert_eq!(ranked[2].name, "low");
}

// #given an empty candidate list
// #when ranked
// #then an empty list is returned
#[test]
fn test_empty_candidates_returns_empty() {
    let empty: Vec<ScoredItem> = vec![];
    let result = rank_trending_items(empty);
    assert!(result.is_empty());
}

// Additional edge-case coverage

#[test]
fn score_is_always_non_negative() {
    for usage in [0_i64, 1, 10, 10_000] {
        for hours in [0.0_f64, 0.001, 1.0, 24.0, 720.0] {
            let score = compute_hashtag_trending_score(usage, hours);
            assert!(
                score >= 0.0,
                "score must be >= 0 for usage={usage}, hours={hours}, got {score}"
            );
        }
    }
}

#[test]
fn age_factor_prevents_division_by_zero() {
    // hours_ago = 0.0 should not produce infinity (the +2 guard handles this)
    let score = compute_hashtag_trending_score(100, 0.0);
    assert!(score.is_finite(), "score must be finite for hours_ago=0");
}

#[test]
fn ranking_single_item_returns_unchanged() {
    let items = vec![ScoredItem {
        name: "only",
        score: 42.0,
    }];
    let ranked = rank_trending_items(items);
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].name, "only");
}
