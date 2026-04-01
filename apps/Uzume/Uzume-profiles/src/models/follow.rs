//! Database-mapped follow relationship types.
//!
//! These types mirror the `uzume.follows` `PostgreSQL` table.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A single follow relationship row.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FollowRow {
    pub follower_id: Uuid,
    pub followee_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// The follow relationship status between two profiles as seen from the
/// requesting user's perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FollowStatus {
    /// The viewer follows the target profile.
    Following,
    /// The viewer does not follow the target profile.
    NotFollowing,
    /// The viewer is blocked by (or has blocked) the target profile.
    Blocked,
}

/// A compact profile summary returned inside follower/following lists.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FollowProfileRow {
    pub id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}
