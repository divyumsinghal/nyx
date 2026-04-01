//! Database-mapped comment types.
//!
//! These types mirror the `uzume.comments` PostgreSQL table and are used
//! exclusively by the queries layer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A full comment row as stored in `uzume.comments`.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CommentRow {
    pub id: Uuid,
    pub post_id: Uuid,
    /// Internal Kratos identity UUID. SECURITY: never include in API responses.
    pub author_identity_id: Uuid,
    pub author_alias: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Payload for inserting a new comment row.
#[derive(Debug, Clone)]
pub struct CommentInsert {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_identity_id: Uuid,
    pub author_alias: String,
    pub content: String,
}
