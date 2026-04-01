//! Database-mapped profile types.
//!
//! These types mirror the `uzume.profiles` `PostgreSQL` table and are used
//! exclusively by the queries layer. Handler and service layers work with
//! the domain types defined in `lib.rs`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A full profile row as stored in `uzume.profiles`.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProfileRow {
    pub id: Uuid,
    pub identity_id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub is_private: bool,
    pub is_verified: bool,
    pub follower_count: i64,
    pub following_count: i64,
    pub post_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload for inserting a new profile row.
#[derive(Debug, Clone)]
pub struct ProfileInsert {
    pub id: Uuid,
    pub identity_id: Uuid,
    pub alias: String,
    pub display_name: String,
}

/// Partial update applied to an existing profile.
#[derive(Debug, Clone, Default)]
pub struct ProfileUpdate {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub is_private: Option<bool>,
}
