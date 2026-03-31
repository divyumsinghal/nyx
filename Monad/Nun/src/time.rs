//! Time utilities for the Nyx platform.
//!
//! All timestamps in the Nyx platform are UTC. The [`Timestamp`] type alias
//! enforces this at the type level — there is no `DateTime<Local>` anywhere
//! in the codebase.
//!
//! # TTL constants
//!
//! The [`ttl`] module provides standard cache and expiration durations used
//! across the platform. Services reference these constants rather than
//! hardcoding durations.

use chrono::{DateTime, Utc};
use std::time::Duration;

/// A UTC timestamp. Every timestamp in the Nyx platform uses this type.
///
/// This is a type alias, not a newtype, so it works seamlessly with `chrono`,
/// `serde`, and `sqlx` without additional trait impls.
pub type Timestamp = DateTime<Utc>;

/// Returns the current UTC timestamp.
pub fn now() -> Timestamp {
    Utc::now()
}

/// Standard TTL (Time To Live) durations used across the platform.
///
/// Services reference these constants for cache expiration, story lifetimes,
/// and other time-bounded operations. Using shared constants ensures consistency
/// and makes TTL changes a single-point update.
pub mod ttl {
    use super::Duration;

    // ── Content TTLs ────────────────────────────────────────────────────

    /// Story expiration: 24 hours.
    pub const STORY: Duration = Duration::from_secs(24 * 60 * 60);

    // ── Cache TTLs ──────────────────────────────────────────────────────

    /// Validated Kratos session cache: 15 minutes.
    /// Sessions are re-validated against Kratos after this period.
    pub const SESSION_CACHE: Duration = Duration::from_secs(15 * 60);

    /// User profile cache: 5 minutes.
    /// Profiles change infrequently; a short TTL balances freshness and load.
    pub const PROFILE_CACHE: Duration = Duration::from_secs(5 * 60);

    /// Home feed cache: 10 minutes.
    /// Feeds are expensive to build; caching amortizes the cost.
    pub const FEED_CACHE: Duration = Duration::from_secs(10 * 60);

    /// Individual post cache: 2 minutes.
    /// Posts are read-heavy; short TTL keeps like/comment counts fresh.
    pub const POST_CACHE: Duration = Duration::from_secs(2 * 60);

    /// Hot/trending data cache: 5 minutes.
    /// Trending hashtags, explore page candidates, etc.
    pub const TRENDING_CACHE: Duration = Duration::from_secs(5 * 60);

    // ── Rate limiting windows ───────────────────────────────────────────

    /// Default rate limit window: 60 seconds.
    pub const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_returns_utc() {
        let ts = now();
        // Verify it's a valid timestamp (not zero/epoch).
        assert!(ts.timestamp() > 0);
    }

    #[test]
    fn story_ttl_is_24h() {
        assert_eq!(ttl::STORY.as_secs(), 86_400);
    }

    #[test]
    fn session_cache_is_15min() {
        assert_eq!(ttl::SESSION_CACHE.as_secs(), 900);
    }
}
