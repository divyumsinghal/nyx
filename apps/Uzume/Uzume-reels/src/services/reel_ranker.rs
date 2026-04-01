//! Pure algorithmic scoring for reels.
//!
//! The score drives the algorithmic feed ordering. Higher score → shown more
//! often. This module is intentionally side-effect free — no I/O, no async.
//! All inputs come from the caller, making it easy to unit-test and tune.
//!
//! ## Score formula
//!
//! ```text
//! completion_rate = avg_watch_duration_ms / reel_duration_ms   (clamped 0..=1)
//!
//! raw_score = (like_weight   * likes)
//!           + (view_weight   * views)
//!           + (completion_weight * completion_rate)
//!
//! time_decay = exp(-decay_rate * hours_since_posted)
//!
//! score = raw_score * time_decay
//! ```
//!
//! Default weights:
//! - `like_weight`        = 2.0   (high-signal intent)
//! - `view_weight`        = 0.1   (lower signal; easy to inflate)
//! - `completion_weight`  = 5.0   (full watches are the strongest signal)
//! - `decay_rate`         = 0.05  (score halves after ≈ 14 h)

/// Metrics derived from the database / cache, passed to [`compute_score`].
#[derive(Debug, Clone)]
pub struct ReelMetrics {
    /// Number of likes.
    pub likes: i64,
    /// Number of qualified views (watch_percent ≥ 25).
    pub views: i64,
    /// Average watch duration in milliseconds across all recorded views.
    pub avg_watch_duration_ms: f64,
    /// Total reel duration in milliseconds (from the `duration_ms` column).
    pub reel_duration_ms: f64,
    /// How many hours ago the reel was created (may be fractional).
    pub hours_since_posted: f64,
}

/// Configurable weights for the ranking formula.
///
/// Override individual fields to tune the algorithm without changing code.
#[derive(Debug, Clone)]
pub struct RankerConfig {
    /// Weight applied to the like count.
    pub like_weight: f64,
    /// Weight applied to the view count.
    pub view_weight: f64,
    /// Weight applied to the completion rate.
    pub completion_weight: f64,
    /// Decay rate per hour (higher → faster decay).
    pub decay_rate: f64,
}

impl Default for RankerConfig {
    fn default() -> Self {
        Self {
            like_weight: 2.0,
            view_weight: 0.1,
            completion_weight: 5.0,
            decay_rate: 0.05,
        }
    }
}

/// Compute the algorithmic ranking score for a reel.
///
/// Returns `0.0` when there are no views (a reel with no engagement has no
/// score regardless of age).
///
/// # Arguments
///
/// * `metrics` — engagement data and reel metadata.
/// * `config`  — tunable scoring weights; use [`RankerConfig::default()`] for
///   the production defaults.
///
/// # Examples
///
/// ```
/// use uzume_reels::services::reel_ranker::{compute_score, ReelMetrics, RankerConfig};
///
/// let metrics = ReelMetrics {
///     likes: 100,
///     views: 500,
///     avg_watch_duration_ms: 24_000.0, // 24 s out of 30 s = 80% completion
///     reel_duration_ms: 30_000.0,
///     hours_since_posted: 2.0,
/// };
///
/// let score = compute_score(&metrics, &RankerConfig::default());
/// assert!(score > 0.0);
/// ```
#[must_use]
pub fn compute_score(metrics: &ReelMetrics, config: &RankerConfig) -> f64 {
    // Nothing to score — avoid division by zero in completion rate.
    if metrics.views == 0 && metrics.likes == 0 {
        return 0.0;
    }

    let completion_rate = if metrics.reel_duration_ms > 0.0 {
        (metrics.avg_watch_duration_ms / metrics.reel_duration_ms).clamp(0.0, 1.0)
    } else {
        0.0
    };

    #[allow(clippy::cast_precision_loss)]
    let raw_score = config.like_weight * metrics.likes as f64
        + config.view_weight * metrics.views as f64
        + config.completion_weight * completion_rate;

    let time_decay = (-config.decay_rate * metrics.hours_since_posted).exp();

    raw_score * time_decay
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_metrics() -> ReelMetrics {
        ReelMetrics {
            likes: 100,
            views: 500,
            avg_watch_duration_ms: 24_000.0,
            reel_duration_ms: 30_000.0,
            hours_since_posted: 2.0,
        }
    }

    // #given a fresh viral reel (high likes + views, posted 2h ago)
    // #and an old reel (same engagement, posted 48h ago)
    // #then fresh reel scores higher due to time decay
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
            "fresh={fresh_score} should be > old={old_score}"
        );
    }

    // #given two reels with identical likes/views but different completion rates
    // #then higher completion rate yields a higher score
    #[test]
    fn test_high_completion_rate_boosts_score() {
        let config = RankerConfig::default();

        let high_completion = ReelMetrics {
            avg_watch_duration_ms: 28_000.0, // ~93% completion
            ..default_metrics()
        };
        let low_completion = ReelMetrics {
            avg_watch_duration_ms: 5_000.0, // ~17% completion
            ..default_metrics()
        };

        let high_score = compute_score(&high_completion, &config);
        let low_score = compute_score(&low_completion, &config);

        assert!(
            high_score > low_score,
            "high_completion={high_score} should be > low_completion={low_score}"
        );
    }

    // #given a reel posted many hours ago
    // #then score decays significantly compared to when newly posted
    #[test]
    fn test_score_decays_over_time() {
        let config = RankerConfig::default();

        let new_reel = ReelMetrics {
            hours_since_posted: 0.0,
            ..default_metrics()
        };
        let week_old = ReelMetrics {
            hours_since_posted: 168.0, // 7 days
            ..default_metrics()
        };

        let new_score = compute_score(&new_reel, &config);
        let old_score = compute_score(&week_old, &config);

        assert!(
            new_score > old_score * 2.0,
            "new={new_score} should be much larger than week_old={old_score}"
        );
    }

    // #given a reel with zero views and zero likes
    // #then score is exactly 0.0
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

        let score = compute_score(&metrics, &config);
        assert_eq!(score, 0.0, "zero engagement must yield zero score");
    }

    // #given custom ranker weights (like_weight boosted to 10.0)
    // #then like-heavy reel outscores view-heavy reel with low likes
    #[test]
    fn test_ranker_config_weights_are_applied() {
        let config = RankerConfig {
            like_weight: 10.0,
            view_weight: 0.01,
            completion_weight: 1.0,
            decay_rate: 0.0, // disable decay for this test
        };

        let like_heavy = ReelMetrics {
            likes: 1000,
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

        let like_score = compute_score(&like_heavy, &config);
        let view_score = compute_score(&view_heavy, &config);

        assert!(
            like_score > view_score,
            "like_score={like_score} should dominate view_score={view_score} with like_weight=10"
        );
    }

    #[test]
    fn test_completion_rate_clamped_to_one() {
        // avg_watch > reel_duration should not produce rate > 1.0
        let config = RankerConfig {
            decay_rate: 0.0,
            ..RankerConfig::default()
        };
        let metrics = ReelMetrics {
            likes: 0,
            views: 1,
            avg_watch_duration_ms: 60_000.0, // longer than reel
            reel_duration_ms: 30_000.0,
            hours_since_posted: 0.0,
        };

        let score = compute_score(&metrics, &config);
        // completion_weight * 1.0 + view_weight * 1 = 5.0 + 0.1 = 5.1
        let expected = config.completion_weight * 1.0 + config.view_weight * 1.0;
        assert!(
            (score - expected).abs() < 1e-9,
            "score={score} expected≈{expected}"
        );
    }

    #[test]
    fn test_zero_duration_does_not_panic() {
        let config = RankerConfig::default();
        let metrics = ReelMetrics {
            likes: 10,
            views: 50,
            avg_watch_duration_ms: 0.0,
            reel_duration_ms: 0.0, // zero duration — must not divide by zero
            hours_since_posted: 1.0,
        };

        let score = compute_score(&metrics, &config);
        assert!(score.is_finite(), "score must be finite, got {score}");
    }
}
