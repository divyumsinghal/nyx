//! Database-mapped post types.
//!
//! These types mirror the `uzume.posts` PostgreSQL table and are used
//! exclusively by the queries layer. The `identity_id` column maps to
//! the Kratos identity UUID — it must never appear in HTTP responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A full post row as stored in `uzume.posts`.
///
/// The `identity_id` field is the internal Kratos identity — never expose
/// it in HTTP responses. Use [`PostRow::into_response`] to obtain the
/// safe public representation.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PostRow {
    pub id: Uuid,
    /// Internal Kratos identity UUID. SECURITY: never include in API responses.
    pub identity_id: Uuid,
    pub author_alias: String,
    pub caption: String,
    pub like_count: i64,
    pub comment_count: i64,
    pub created_at: DateTime<Utc>,
}

/// Payload for inserting a new post row.
#[derive(Debug, Clone)]
pub struct PostInsert {
    pub id: Uuid,
    pub identity_id: Uuid,
    pub author_alias: String,
    pub caption: String,
}
