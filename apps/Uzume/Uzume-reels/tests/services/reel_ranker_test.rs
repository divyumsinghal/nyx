//! External integration tests for the reel ranking service.
//!
//! These tests verify the scoring algorithm with various input combinations
//! using the public API.

use uzume_reels::services::reel_ranker::{compute_score, RankerConfig, ReelMetrics};

fn default_metrics() -> ReelMetrics {
    ReelMetrics {
        likes: 100,
        views: 500,
        avg_watch_duration_ms: 20_000.0,
        reel_duration_ms: 30_000.0,
        hours_since_posted: 1.0,
    }
}

#[test]
fn score_is_positive_for_engaged_reel() {
    let metrics = default_metrics();
    let score = compute_score(&metrics, &RankerConfig::default());
    assert!(score > 0.0, "score should be positive: {score}");
}

#[test]
fn fresh_reel_scores_higher_than_old_reel() {
    let config = RankerConfig::default();
    let fresh = ReelMetrics {
        hours_since_posted: 1.0,
        ..default_metrics()
    };
    let old = ReelMetrics {
        hours_since_posted: 100.0,
        ..default_metrics()
    };
    let fresh_score = compute_score(&fresh, &config);
    let old_score = compute_score(&old, &config);
    assert!(
        fresh_score > old_score,
        "fresh_score={fresh_score} should > old_score={old_score}"
    );
}

#[test]
fn zero_engagement_returns_zero() {
    let config = RankerConfig::default();
    let metrics = ReelMetrics {
        likes: 0,
        views: 0,
        avg_watch_duration_ms: 0.0,
        reel_duration_ms: 30_000.0,
        hours_since_posted: 1.0,
    };
    let score = compute_score(&metrics, &config);
    assert_eq!(score, 0.0, "no engagement should yield zero score");
}

#[test]
fn high_completion_rate_boosts_score() {
    let config = RankerConfig::default();
    let low_completion = ReelMetrics {
        avg_watch_duration_ms: 1_000.0,
        reel_duration_ms: 30_000.0,
        likes: 10,
        views: 100,
        hours_since_posted: 0.0,
    };
    let high_completion = ReelMetrics {
        avg_watch_duration_ms: 29_000.0,
        reel_duration_ms: 30_000.0,
        likes: 10,
        views: 100,
        hours_since_posted: 0.0,
    };
    let low_score = compute_score(&low_completion, &config);
    let high_score = compute_score(&high_completion, &config);
    assert!(
        high_score > low_score,
        "high completion rate ({high_score}) should score above low ({low_score})"
    );
}

#[test]
fn score_is_always_finite() {
    let config = RankerConfig::default();
    // Edge: zero duration
    let m = ReelMetrics {
        likes: 5,
        views: 5,
        avg_watch_duration_ms: 0.0,
        reel_duration_ms: 0.0,
        hours_since_posted: 0.0,
    };
    let score = compute_score(&m, &config);
    assert!(score.is_finite(), "score must be finite, got {score}");
}

#[test]
fn custom_weights_change_score() {
    let default_score = compute_score(&default_metrics(), &RankerConfig::default());
    let custom = RankerConfig {
        like_weight: 10.0,
        view_weight: 0.0,
        completion_weight: 0.0,
        decay_rate: 0.0,
    };
    let custom_score = compute_score(&default_metrics(), &custom);
    assert_ne!(
        default_score, custom_score,
        "custom weights should produce different score"
    );
}
