//! Unit tests for the reel ranking algorithm.
//!
//! These tests are pure — no I/O, no database, no network.
//! They verify the mathematical properties of the scoring formula.

use uzume_reels::services::reel_ranker::{compute_score, RankerConfig, ReelMetrics};

fn default_metrics() -> ReelMetrics {
    ReelMetrics {
        likes: 100,
        views: 500,
        avg_watch_duration_ms: 24_000.0,
        reel_duration_ms: 30_000.0,
        hours_since_posted: 2.0,
    }
}

/// A fresh, viral reel (2h old) must outscore an identically-engaged reel
/// that is 48 hours old. Time decay is the differentiator.
#[test]
fn test_fresh_viral_reel_scores_higher_than_old_reel() {
    let config = RankerConfig::default();

    let fresh = ReelMetrics {
        hours_since_posted: 2.0,
        ..default_metrics()
    };
    let old = ReelMetrics {
        hours_since_posted: 48.0,
        ..default_metrics()
    };

    let fresh_score = compute_score(&fresh, &config);
    let old_score = compute_score(&old, &config);

    assert!(
        fresh_score > old_score,
        "fresh reel (2h) score {fresh_score:.4} must exceed old reel (48h) score {old_score:.4}"
    );
}

/// High completion rate (93%) must beat low completion rate (17%) given
/// identical likes/views and age.
#[test]
fn test_high_completion_rate_boosts_score() {
    let config = RankerConfig::default();

    let high_completion = ReelMetrics {
        avg_watch_duration_ms: 28_000.0, // 93.3% of 30 s
        ..default_metrics()
    };
    let low_completion = ReelMetrics {
        avg_watch_duration_ms: 5_000.0, // 16.7% of 30 s
        ..default_metrics()
    };

    let high = compute_score(&high_completion, &config);
    let low = compute_score(&low_completion, &config);

    assert!(
        high > low,
        "high completion ({high:.4}) must exceed low completion ({low:.4})"
    );
}

/// Score must strictly decrease as `hours_since_posted` increases.
/// We verify three monotonically-increasing age points.
#[test]
fn test_score_decays_over_time() {
    let config = RankerConfig::default();

    let s0 = compute_score(&ReelMetrics { hours_since_posted: 0.0, ..default_metrics() }, &config);
    let s24 = compute_score(&ReelMetrics { hours_since_posted: 24.0, ..default_metrics() }, &config);
    let s168 = compute_score(&ReelMetrics { hours_since_posted: 168.0, ..default_metrics() }, &config);

    assert!(s0 > s24, "s0={s0:.4} > s24={s24:.4}");
    assert!(s24 > s168, "s24={s24:.4} > s168={s168:.4}");
}

/// A reel with zero views AND zero likes must return exactly 0.0.
#[test]
fn test_zero_views_gives_zero_score() {
    let config = RankerConfig::default();
    let metrics = ReelMetrics {
        likes: 0,
        views: 0,
        avg_watch_duration_ms: 0.0,
        reel_duration_ms: 30_000.0,
        hours_since_posted: 1.0,
    };

    assert_eq!(compute_score(&metrics, &config), 0.0);
}

/// Custom weights must be respected: boosting `like_weight` to 10.0 should
/// make a like-heavy reel outscore a view-heavy reel.
#[test]
fn test_ranker_config_weights_are_applied() {
    let config = RankerConfig {
        like_weight: 10.0,
        view_weight: 0.01,
        completion_weight: 1.0,
        decay_rate: 0.0, // disable decay so weights are the sole variable
    };

    let like_heavy = ReelMetrics {
        likes: 1_000,
        views: 10,
        avg_watch_duration_ms: 15_000.0,
        reel_duration_ms: 30_000.0,
        hours_since_posted: 0.0,
    };
    let view_heavy = ReelMetrics {
        likes: 1,
        views: 100_000,
        avg_watch_duration_ms: 15_000.0,
        reel_duration_ms: 30_000.0,
        hours_since_posted: 0.0,
    };

    let ls = compute_score(&like_heavy, &config);
    let vs = compute_score(&view_heavy, &config);

    assert!(ls > vs, "like_score={ls:.4} should dominate view_score={vs:.4} with like_weight=10");
}

/// Completion rate must be clamped to 1.0 even when avg_watch > reel_duration.
#[test]
fn test_completion_rate_cannot_exceed_one() {
    let config = RankerConfig {
        decay_rate: 0.0,
        ..RankerConfig::default()
    };
    let over_watch = ReelMetrics {
        likes: 0,
        views: 1,
        avg_watch_duration_ms: 60_000.0, // 2× the reel duration
        reel_duration_ms: 30_000.0,
        hours_since_posted: 0.0,
    };
    let full_watch = ReelMetrics {
        likes: 0,
        views: 1,
        avg_watch_duration_ms: 30_000.0, // exactly 100%
        reel_duration_ms: 30_000.0,
        hours_since_posted: 0.0,
    };

    let over_score = compute_score(&over_watch, &config);
    let full_score = compute_score(&full_watch, &config);

    assert!(
        (over_score - full_score).abs() < 1e-9,
        "over_watch ({over_score}) and full_watch ({full_score}) must produce equal scores"
    );
}

/// A reel with only likes (no views) must still receive a positive score.
#[test]
fn test_likes_alone_generate_positive_score() {
    let config = RankerConfig {
        decay_rate: 0.0,
        ..RankerConfig::default()
    };
    let metrics = ReelMetrics {
        likes: 50,
        views: 0,
        avg_watch_duration_ms: 0.0,
        reel_duration_ms: 30_000.0,
        hours_since_posted: 0.0,
    };

    let score = compute_score(&metrics, &config);
    assert!(score > 0.0, "likes alone should produce score > 0, got {score}");
}

/// Score must be finite for all reasonable inputs (no NaN, no infinity).
#[test]
fn test_score_is_always_finite() {
    let config = RankerConfig::default();

    let edge_cases = vec![
        ReelMetrics { likes: 0, views: 0, avg_watch_duration_ms: 0.0, reel_duration_ms: 0.0, hours_since_posted: 0.0 },
        ReelMetrics { likes: i64::MAX, views: i64::MAX, avg_watch_duration_ms: 1e15, reel_duration_ms: 1.0, hours_since_posted: 0.0 },
        ReelMetrics { likes: 1, views: 1, avg_watch_duration_ms: 1.0, reel_duration_ms: 1.0, hours_since_posted: 1e6 },
    ];

    for (i, m) in edge_cases.iter().enumerate() {
        let score = compute_score(m, &config);
        assert!(score.is_finite(), "test case {i}: score must be finite, got {score}");
        assert!(!score.is_nan(), "test case {i}: score must not be NaN");
    }
}
