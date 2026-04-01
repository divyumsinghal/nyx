//! Raw SQL queries for the `uzume.stories` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::story::{StoryInsert, StoryRow};
use nun::NyxError;

const SELECT_COLS: &str = r#"
    id, author_identity_id, author_alias, media_url,
    media_type AS "media_type: MediaType",
    duration_secs, status AS "status: StoryStatus",
    view_count, expires_at, created_at
"#;

/// Insert a new story with status `pending` and `expires_at = NOW() + 24h`.
pub async fn create_story(pool: &PgPool, insert: &StoryInsert) -> Result<StoryRow, NyxError> {
    sqlx::query_as::<_, StoryRow>(
        r#"
        INSERT INTO uzume.stories
            (id, author_identity_id, author_alias, media_type, duration_secs, expires_at)
        VALUES ($1, $2, $3, $4, $5, NOW() + INTERVAL '24 hours')
        RETURNING
            id, author_identity_id, author_alias, media_url,
            media_type AS "media_type: MediaType",
            duration_secs, status AS "status: StoryStatus",
            view_count, expires_at, created_at
        "#,
    )
    .bind(insert.id)
    .bind(insert.author_identity_id)
    .bind(&insert.author_alias)
    .bind(insert.media_type)
    .bind(insert.duration_secs)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch a single story by its UUID.
///
/// Returns `None` if no story exists with that ID.
pub async fn get_story_by_id(pool: &PgPool, id: Uuid) -> Result<Option<StoryRow>, NyxError> {
    sqlx::query_as::<_, StoryRow>(&format!(
        r#"
        SELECT {SELECT_COLS}
        FROM uzume.stories
        WHERE id = $1
        "#
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch active stories from accounts the viewer follows, cursor-paginated.
///
/// Returns at most `limit` rows. Fetch `limit + 1` (pass `query_limit`) to
/// detect whether more pages exist. Sorted by `created_at DESC, id DESC`.
pub async fn get_stories_feed(
    pool: &PgPool,
    viewer_identity_id: Uuid,
    cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
    cursor_id: Option<Uuid>,
    query_limit: i64,
) -> Result<Vec<StoryRow>, NyxError> {
    // When a cursor is provided, paginate past it; otherwise start from the top.
    match (cursor_created_at, cursor_id) {
        (Some(ts), Some(cid)) => sqlx::query_as::<_, StoryRow>(&format!(
            r#"
                SELECT {SELECT_COLS}
                FROM uzume.stories s
                WHERE s.status = 'active'
                  AND s.author_identity_id IN (
                      SELECT followed_identity_id
                      FROM uzume.follows
                      WHERE follower_identity_id = $1
                  )
                  AND (s.created_at, s.id) < ($2, $3)
                ORDER BY s.created_at DESC, s.id DESC
                LIMIT $4
                "#
        ))
        .bind(viewer_identity_id)
        .bind(ts)
        .bind(cid)
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
        _ => sqlx::query_as::<_, StoryRow>(&format!(
            r#"
                SELECT {SELECT_COLS}
                FROM uzume.stories s
                WHERE s.status = 'active'
                  AND s.author_identity_id IN (
                      SELECT followed_identity_id
                      FROM uzume.follows
                      WHERE follower_identity_id = $1
                  )
                ORDER BY s.created_at DESC, s.id DESC
                LIMIT $2
                "#
        ))
        .bind(viewer_identity_id)
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
    }
}

/// Fetch all stories authored by the given identity (all statuses), cursor-paginated.
pub async fn get_my_stories(
    pool: &PgPool,
    author_identity_id: Uuid,
    cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
    cursor_id: Option<Uuid>,
    query_limit: i64,
) -> Result<Vec<StoryRow>, NyxError> {
    match (cursor_created_at, cursor_id) {
        (Some(ts), Some(cid)) => sqlx::query_as::<_, StoryRow>(&format!(
            r#"
                SELECT {SELECT_COLS}
                FROM uzume.stories
                WHERE author_identity_id = $1
                  AND (created_at, id) < ($2, $3)
                ORDER BY created_at DESC, id DESC
                LIMIT $4
                "#
        ))
        .bind(author_identity_id)
        .bind(ts)
        .bind(cid)
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
        _ => sqlx::query_as::<_, StoryRow>(&format!(
            r#"
                SELECT {SELECT_COLS}
                FROM uzume.stories
                WHERE author_identity_id = $1
                ORDER BY created_at DESC, id DESC
                LIMIT $2
                "#
        ))
        .bind(author_identity_id)
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
    }
}

/// Update story status to `active` and set `media_url` once Oya completes processing.
///
/// This is idempotent: if the story is already `active` with the same URL the
/// update is a no-op at the database level.
pub async fn activate_story(
    pool: &PgPool,
    id: Uuid,
    media_url: &str,
) -> Result<Option<StoryRow>, NyxError> {
    sqlx::query_as::<_, StoryRow>(&format!(
        r#"
        UPDATE uzume.stories
        SET status = 'active', media_url = $2
        WHERE id = $1 AND status = 'pending'
        RETURNING {SELECT_COLS}
        "#
    ))
    .bind(id)
    .bind(media_url)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Transition all active stories whose `expires_at` has passed to `expired`.
///
/// Returns the IDs of stories that were just expired so callers can publish
/// `Uzume.story.expired` events. This function is idempotent.
pub async fn expire_old_stories(pool: &PgPool) -> Result<Vec<Uuid>, NyxError> {
    let rows = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE uzume.stories
        SET status = 'expired'
        WHERE status = 'active' AND expires_at < NOW()
        RETURNING id
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(rows)
}

/// Hard-delete expired stories whose `expires_at` is older than 7 days.
///
/// Cascade deletes propagate to `story_views` and `highlight_stories` via
/// foreign key constraints. This is idempotent.
pub async fn delete_expired_stories(pool: &PgPool) -> Result<u64, NyxError> {
    let result = sqlx::query(
        r#"
        DELETE FROM uzume.stories
        WHERE status = 'expired'
          AND expires_at < NOW() - INTERVAL '7 days'
        "#,
    )
    .execute(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(result.rows_affected())
}

/// Delete a single story owned by the given identity.
///
/// Returns `true` if a row was deleted, `false` if the story was not found or
/// is owned by a different identity.
pub async fn delete_story(
    pool: &PgPool,
    id: Uuid,
    author_identity_id: Uuid,
) -> Result<bool, NyxError> {
    let result = sqlx::query(
        r#"
        DELETE FROM uzume.stories
        WHERE id = $1 AND author_identity_id = $2
        "#,
    )
    .bind(id)
    .bind(author_identity_id)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the SQL column list constant is non-empty and contains expected fields.
    #[test]
    fn select_cols_includes_required_fields() {
        assert!(SELECT_COLS.contains("author_identity_id"));
        assert!(SELECT_COLS.contains("author_alias"));
        assert!(SELECT_COLS.contains("media_url"));
        assert!(SELECT_COLS.contains("status"));
        assert!(SELECT_COLS.contains("expires_at"));
    }
}
