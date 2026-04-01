//! Pure trending calculation logic for the discover service.
//!
//! All functions here are side-effect-free and independently unit-testable.
//! They operate on pre-fetched data and return computed scores; any database
//! or cache I/O is done by the callers in [`crate::workers::trending_updater`].

/// Gravity exponent used in the time-decay formula.
///
/// A higher gravity value makes recent content decay faster.
/// Instagram/HN typically use values between 1.5 and 2.0.
const GRAVITY: f64 = 1.5;

/// Compute the trending score for a hashtag.
///
/// Uses a time-decayed formula:
///
/// ```text
/// score = usage_count / (hours_ago + 2)^GRAVITY
/// ```
///
/// The `+ 2` constant prevents division explosion for very new items and
/// ensures a 0-hour-old item does not receive an infinite score relative
/// to older content.
///
/// # Arguments
///
/// - `usage_count` — Number of posts using this hashtag within the window.
/// - `hours_ago` — How many hours ago the first use of the hashtag was observed.
///
/// # Returns
///
/// A non-negative float. Higher values rank higher in trending lists.
#[must_use]
pub fn compute_hashtag_trending_score(usage_count: i64, hours_ago: f64) -> f64 {
    let count = usage_count as f64;
    let age_factor = (hours_ago + 2.0).powf(GRAVITY);
    count / age_factor
}

/// A type that carries a ranking score and entity ID, enabling generic ranking.
pub trait HasScore {
    /// The ranking score. Higher values are ranked first.
    fn score(&self) -> f64;
}

/// Rank a list of items by score descending.
///
/// Items with equal scores retain their original relative order (stable sort).
/// The original `Vec` is consumed and a sorted `Vec` is returned.
#[must_use]
pub fn rank_trending_items<T: HasScore>(mut items: Vec<T>) -> Vec<T> {
    items.sort_by(|a, b| {
        b.score()
            .partial_cmp(&a.score())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    // #given a hashtag used recently
    // #when compared to an older hashtag with the same count
    // #then the more recent hashtag scores higher
    #[test]
    fn test_more_recent_hashtag_scores_higher() {
        let recent = compute_hashtag_trending_score(100, 1.0);
        let older = compute_hashtag_trending_score(100, 10.0);
        assert!(recent > older, "recent={recent}, older={older}");
    }

    // #given a hashtag with many recent uses
    // #when compared to a stale popular one
    // #then the viral hashtag beats the stale popular one
    #[test]
    fn test_viral_hashtag_beats_stale_popular_one() {
        // 500 uses in the last hour vs 1000 uses from 24 hours ago
        let viral = compute_hashtag_trending_score(500, 1.0);
        let stale_popular = compute_hashtag_trending_score(1000, 24.0);
        assert!(viral > stale_popular, "viral={viral}, stale={stale_popular}");
    }

    // #given a list of scored items
    // #when ranked
    // #then the order is highest score first
    #[test]
    fn test_ranking_preserves_order() {
        #[derive(Debug, PartialEq)]
        struct Item {
            name: &'static str,
            s: f64,
        }
        impl HasScore for Item {
            fn score(&self) -> f64 {
                self.s
            }
        }

        let items = vec![
            Item { name: "c", s: 1.0 },
            Item { name: "a", s: 99.0 },
            Item { name: "b", s: 42.0 },
        ];

        let ranked = rank_trending_items(items);
        assert_eq!(ranked[0].name, "a");
        assert_eq!(ranked[1].name, "b");
        assert_eq!(ranked[2].name, "c");
    }

    // #given an empty list
    // #when ranked
    // #then an empty list is returned
    #[test]
    fn test_empty_candidates_returns_empty() {
        struct Item;
        impl HasScore for Item {
            fn score(&self) -> f64 {
                0.0
            }
        }

        let result: Vec<Item> = rank_trending_items(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn score_is_non_negative_for_any_valid_input() {
        for usage in [0, 1, 100, 10_000] {
            for hours in [0.0_f64, 0.5, 1.0, 24.0, 168.0] {
                let score = compute_hashtag_trending_score(usage, hours);
                assert!(score >= 0.0, "score must be non-negative: usage={usage}, hours={hours}");
            }
        }
    }
}
