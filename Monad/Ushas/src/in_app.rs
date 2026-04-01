//! In-app notification storage — PostgreSQL persistence for notification records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::UshasError;

/// A stored in-app notification record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    /// Unique notification ID.
    pub id: Uuid,
    /// Recipient user ID.
    pub user_id: Uuid,
    /// App scope (e.g. `"uzume"`).
    pub app: String,
    /// Event type (e.g. `"post.liked"`).
    pub event_type: String,
    /// User that triggered the event.
    pub actor_id: Uuid,
    /// Entity the event relates to (post ID, comment ID, etc.).
    pub entity_id: Option<Uuid>,
    /// Human-readable notification body.
    pub body: String,
    /// Whether the recipient has read this notification.
    pub is_read: bool,
    /// When the notification was created.
    pub created_at: DateTime<Utc>,
}

/// Persist a notification record to the database.
///
/// Uses `ON CONFLICT DO NOTHING` so duplicate dispatches are idempotent.
pub async fn store(pool: &PgPool, n: &Notification) -> Result<(), UshasError> {
    sqlx::query(
        "INSERT INTO uzume.notifications \
         (id, user_id, app, event_type, actor_id, entity_id, body, is_read, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(n.id)
    .bind(n.user_id)
    .bind(&n.app)
    .bind(&n.event_type)
    .bind(n.actor_id)
    .bind(n.entity_id)
    .bind(&n.body)
    .bind(n.is_read)
    .bind(n.created_at)
    .execute(pool)
    .await
    .map_err(UshasError::Database)?;
    Ok(())
}

/// Fetch unread notifications for a user, newest first.
pub async fn get_unread(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<Notification>, UshasError> {
    sqlx::query_as::<_, Notification>(
        "SELECT id, user_id, app, event_type, actor_id, entity_id, body, is_read, created_at \
         FROM uzume.notifications \
         WHERE user_id = $1 AND NOT is_read \
         ORDER BY created_at DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(UshasError::Database)
}

/// Mark a single notification as read.
pub async fn mark_read(
    pool: &PgPool,
    user_id: Uuid,
    notification_id: Uuid,
) -> Result<(), UshasError> {
    sqlx::query(
        "UPDATE uzume.notifications SET is_read = true WHERE id = $1 AND user_id = $2",
    )
    .bind(notification_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(UshasError::Database)?;
    Ok(())
}
