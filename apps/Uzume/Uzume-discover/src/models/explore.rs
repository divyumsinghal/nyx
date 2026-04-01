//! Explore page domain models for the discover service.
//!
//! The explore page is composed of heterogeneous sections (trending posts,
//! suggested users, trending hashtags, featured reels), each containing a
//! ranked list of items.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The category of content within an explore section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExploreSectionType {
    /// High-engagement posts from the last 24–48 hours.
    TrendingPosts,

    /// Users the viewer might want to follow (second-degree graph).
    SuggestedUsers,

    /// Hashtags currently spiking in usage.
    TrendingHashtags,

    /// Algorithmically selected reels.
    FeaturedReels,
}

/// The type of entity inside an [`ExploreItem`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExploreItemType {
    /// A photo or carousel post.
    Post,

    /// A user profile card.
    User,

    /// A hashtag pill.
    Hashtag,

    /// A short-form video reel.
    Reel,
}

/// A single ranked item within an explore section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreItem {
    /// UUID of the entity (post ID, profile ID, reel ID, etc.).
    /// For hashtag items this field is `Uuid::nil()`.
    pub id: Uuid,

    /// The kind of entity this item represents.
    pub item_type: ExploreItemType,

    /// CDN URL for a thumbnail preview image.
    /// `None` for hashtag items.
    pub thumbnail_url: Option<String>,

    /// Relevance / engagement score used for ranking.
    /// Higher score → shown earlier in the list.
    pub score: f64,
}

/// A ranked list of items of one content category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreSection {
    /// The category this section represents.
    pub section_type: ExploreSectionType,

    /// Ranked items, highest score first.
    pub items: Vec<ExploreItem>,
}

/// Query parameters for the `GET /explore` endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ExploreQuery {
    /// Opaque cursor for pagination (score + ID).
    pub cursor: Option<String>,

    /// Page size, clamped 1–100, defaults to 20.
    #[serde(default = "default_explore_limit")]
    pub limit: u16,
}

fn default_explore_limit() -> u16 {
    20
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_type_serializes_snake_case() {
        assert_eq!(
            serde_json::to_value(ExploreSectionType::TrendingPosts).unwrap(),
            "trending_posts"
        );
        assert_eq!(
            serde_json::to_value(ExploreSectionType::SuggestedUsers).unwrap(),
            "suggested_users"
        );
        assert_eq!(
            serde_json::to_value(ExploreSectionType::TrendingHashtags).unwrap(),
            "trending_hashtags"
        );
        assert_eq!(
            serde_json::to_value(ExploreSectionType::FeaturedReels).unwrap(),
            "featured_reels"
        );
    }

    #[test]
    fn item_type_serializes_snake_case() {
        assert_eq!(
            serde_json::to_value(ExploreItemType::Post).unwrap(),
            "post"
        );
        assert_eq!(
            serde_json::to_value(ExploreItemType::User).unwrap(),
            "user"
        );
        assert_eq!(
            serde_json::to_value(ExploreItemType::Reel).unwrap(),
            "reel"
        );
        assert_eq!(
            serde_json::to_value(ExploreItemType::Hashtag).unwrap(),
            "hashtag"
        );
    }

    #[test]
    fn explore_section_serializes_correctly() {
        let section = ExploreSection {
            section_type: ExploreSectionType::TrendingPosts,
            items: vec![ExploreItem {
                id: Uuid::nil(),
                item_type: ExploreItemType::Post,
                thumbnail_url: Some("https://cdn.example/thumb.jpg".to_string()),
                score: 99.5,
            }],
        };
        let json = serde_json::to_value(&section).unwrap();
        assert_eq!(json["section_type"], "trending_posts");
        assert_eq!(json["items"][0]["score"], 99.5);
    }
}
