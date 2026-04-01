//! Trending domain models for the discover service.
//!
//! These types represent the state of what is currently trending on Uzume,
//! computed periodically by the [`crate::workers::trending_updater`] worker.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A single trending hashtag with its engagement score.
///
/// Rows are stored in `uzume.trending_hashtags` and refreshed every 5 minutes
/// by the trending_updater worker.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrendingHashtag {
    /// The hashtag string (without the `#` prefix).
    pub hashtag: String,

    /// Number of posts that used this hashtag within the scoring window.
    pub post_count: i64,

    /// Computed trending score. Higher scores appear first in responses.
    pub score: f64,

    /// When this row was last upserted by the trending_updater worker.
    pub updated_at: DateTime<Utc>,
}

/// A reel that is currently trending in the algorithmic feed.
///
/// Populated from the `uzume.reels` table, ordered by their denormalized
/// `score` column maintained by `uzume-reels`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingReel {
    /// UUID of the trending reel.
    pub reel_id: Uuid,

    /// Algorithmic score from the `uzume.reels.score` column.
    pub score: f64,

    /// When the trending snapshot was computed.
    pub updated_at: DateTime<Utc>,
}

/// A piece of audio that is trending because many reels are using it.
///
/// Sourced from `uzume.reel_audio` ordered by `use_count DESC`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingAudio {
    /// UUID of the audio track.
    pub audio_id: Uuid,

    /// Display title of the audio track.
    pub title: String,

    /// Number of reels currently using this audio.
    pub reel_count: i64,

    /// Computed score (proportional to `reel_count` with time decay).
    pub score: f64,
}

/// A point-in-time snapshot of all trending content categories.
///
/// Returned by `GET /explore/trending`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingSnapshot {
    /// Top trending hashtags, ranked by score descending.
    pub hashtags: Vec<TrendingHashtag>,

    /// Top trending reels, ranked by score descending.
    pub reels: Vec<TrendingReel>,

    /// Top trending audio tracks, ranked by score descending.
    pub audio: Vec<TrendingAudio>,

    /// UTC timestamp when this snapshot was computed.
    pub computed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trending_hashtag_serializes_without_internal_ids() {
        let tag = TrendingHashtag {
            hashtag: "sunsets".to_string(),
            post_count: 1000,
            score: 42.5,
            updated_at: Utc::now(),
        };
        let json = serde_json::to_value(&tag).unwrap();
        assert_eq!(json["hashtag"], "sunsets");
        assert_eq!(json["post_count"], 1000);
    }

    #[test]
    fn trending_snapshot_contains_all_categories() {
        let snap = TrendingSnapshot {
            hashtags: vec![],
            reels: vec![],
            audio: vec![],
            computed_at: Utc::now(),
        };
        let json = serde_json::to_value(&snap).unwrap();
        assert!(json.get("hashtags").is_some());
        assert!(json.get("reels").is_some());
        assert!(json.get("audio").is_some());
        assert!(json.get("computed_at").is_some());
    }
}
