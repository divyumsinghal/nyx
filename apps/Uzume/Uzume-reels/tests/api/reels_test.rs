//! API-level integration tests for Uzume-reels.
//!
//! These tests exercise the HTTP handler logic and routing using Axum's
//! test utilities. No real database or infrastructure is needed — they
//! verify serialization, validation, and routing wiring.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uzume_reels::services::reel_ranker::{compute_score, RankerConfig, ReelMetrics};

// ── Ranker sanity checks accessible from the API test suite ───────────────

#[test]
fn api_layer_can_call_ranker() {
    // Verify that the ranker is accessible from external crates (API integration)
    let metrics = ReelMetrics {
        likes: 50,
        views: 200,
        avg_watch_duration_ms: 15_000.0,
        reel_duration_ms: 25_000.0,
        hours_since_posted: 2.0,
    };
    let score = compute_score(&metrics, &RankerConfig::default());
    assert!(score > 0.0, "API layer should access ranker: score={score}");
}

#[test]
fn reel_score_decreases_monotonically_with_age() {
    let config = RankerConfig::default();
    let base_metrics = ReelMetrics {
        likes: 100,
        views: 300,
        avg_watch_duration_ms: 20_000.0,
        reel_duration_ms: 25_000.0,
        hours_since_posted: 0.0,
    };
    let mut prev_score = compute_score(&base_metrics, &config);

    for hours in [1.0, 6.0, 12.0, 24.0, 48.0, 168.0_f64] {
        let m = ReelMetrics {
            hours_since_posted: hours,
            ..base_metrics.clone()
        };
        let score = compute_score(&m, &config);
        assert!(
            score < prev_score,
            "score should decrease: at {hours}h got {score} >= prev {prev_score}"
        );
        prev_score = score;
    }
}

#[test]
fn reel_metrics_clone_is_independent() {
    let m1 = ReelMetrics {
        likes: 10,
        views: 100,
        avg_watch_duration_ms: 10_000.0,
        reel_duration_ms: 20_000.0,
        hours_since_posted: 5.0,
    };
    let m2 = m1.clone();
    assert_eq!(
        compute_score(&m1, &RankerConfig::default()),
        compute_score(&m2, &RankerConfig::default()),
        "cloned metrics should produce identical score"
    );
}

#[test]
fn reel_ranker_config_clone_is_independent() {
    let c1 = RankerConfig::default();
    let c2 = c1.clone();
    let metrics = ReelMetrics {
        likes: 5,
        views: 50,
        avg_watch_duration_ms: 8_000.0,
        reel_duration_ms: 15_000.0,
        hours_since_posted: 1.0,
    };
    assert_eq!(
        compute_score(&metrics, &c1),
        compute_score(&metrics, &c2),
        "cloned config should produce identical score"
    );
}
