//! Search result domain models for the discover service.
//!
//! These types are returned from `GET /search` and related endpoints.
//! User identity IDs are intentionally absent; only app-scoped aliases are
//! exposed to API clients.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user profile that matched a search query.
///
/// Only the app-scoped alias is exposed — never the internal `nyx_identity_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResult {
    /// App-scoped profile UUID (safe to expose).
    pub user_id: Uuid,

    /// App-scoped alias (the user's visible handle within Uzume).
    pub alias: String,

    /// User's display name.
    pub display_name: String,

    /// CDN URL of the user's avatar image, if set.
    pub avatar_url: Option<String>,

    /// Denormalized follower count for display in search results.
    pub follower_count: i64,
}

/// A post that matched a search query or hashtag filter.
///
/// `author_id` is the profile UUID (not the Kratos identity UUID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostSearchResult {
    /// UUID of the matching post.
    pub post_id: Uuid,

    /// Profile UUID of the post author (app-scoped, safe to expose).
    pub author_id: Uuid,

    /// App-scoped alias of the post author.
    pub author_alias: String,

    /// Post caption text, if any.
    pub caption: Option<String>,

    /// CDN URL of the first media item's thumbnail, if processed.
    pub thumbnail_url: Option<String>,

    /// Denormalized like count for display in search results.
    pub like_count: i64,

    /// When the post was created.
    pub created_at: DateTime<Utc>,
}

/// The unified response type for `GET /search`.
///
/// All three result categories are returned in a single response so the client
/// can render a mixed search results page without additional round trips.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// User profiles matching the query.
    pub users: Vec<UserSearchResult>,

    /// Posts matching the query.
    pub posts: Vec<PostSearchResult>,

    /// Hashtags matching or containing the query string.
    pub hashtags: Vec<String>,

    /// The original query string, echoed back for client-side use.
    pub query: String,
}

/// The type filter for a search request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchType {
    /// Return users, posts, and hashtags.
    All,
    /// Return only user profiles.
    Users,
    /// Return only posts.
    Posts,
    /// Return only matching hashtags.
    Hashtags,
}

impl Default for SearchType {
    fn default() -> Self {
        Self::All
    }
}

/// Meilisearch document shape for indexed posts.
///
/// This struct must match the fields set by the `search_sync` worker when it
/// calls `brizo.add_documents(indexes::UZUME_POSTS, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostDocument {
    /// Meilisearch primary key (post UUID as string).
    pub id: String,
    pub author_id: String,
    pub author_alias: String,
    pub caption: String,
    pub hashtags: Vec<String>,
    pub thumbnail_url: Option<String>,
    pub like_count: i64,
    pub created_at: String,
}

/// Meilisearch document shape for indexed profiles.
///
/// This struct must match the fields set by the `search_sync` worker when it
/// calls `brizo.add_documents(indexes::UZUME_PROFILES, ...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDocument {
    /// Meilisearch primary key (profile UUID as string).
    pub id: String,
    pub alias: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub follower_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_results_excludes_identity_id() {
        let result = UserSearchResult {
            user_id: Uuid::now_v7(),
            alias: "alice".to_string(),
            display_name: "Alice".to_string(),
            avatar_url: None,
            follower_count: 100,
        };
        let json = serde_json::to_value(&result).unwrap();
        // Must not contain nyx_identity_id — only app-scoped user_id
        assert!(json.get("nyx_identity_id").is_none());
        assert_eq!(json["alias"], "alice");
    }

    #[test]
    fn search_type_default_is_all() {
        assert_eq!(SearchType::default(), SearchType::All);
    }

    #[test]
    fn search_type_deserializes_from_lowercase() {
        let t: SearchType = serde_json::from_str(r#""users""#).unwrap();
        assert_eq!(t, SearchType::Users);

        let t: SearchType = serde_json::from_str(r#""posts""#).unwrap();
        assert_eq!(t, SearchType::Posts);

        let t: SearchType = serde_json::from_str(r#""hashtags""#).unwrap();
        assert_eq!(t, SearchType::Hashtags);

        let t: SearchType = serde_json::from_str(r#""all""#).unwrap();
        assert_eq!(t, SearchType::All);
    }
}
