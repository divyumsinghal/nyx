//! Notification preferences — per-user, per-app, per-event-type mute settings.

use sqlx::PgPool;
use uuid::Uuid;

/// Returns `true` if the user has muted notifications for the given app + event type.
///
/// Defaults to `false` (not muted) when no preference row exists.
pub async fn is_muted(pool: &PgPool, user_id: Uuid, app: &str, event_type: &str) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT is_muted FROM uzume.notification_preferences \
         WHERE user_id = $1 AND app = $2 AND event_type = $3",
    )
    .bind(user_id)
    .bind(app)
    .bind(event_type)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .unwrap_or(false)
}
