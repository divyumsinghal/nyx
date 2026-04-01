//! Raw SQL queries for the `uzume.story_views` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::viewer::StoryViewRow;
use nun::NyxError;

/// Upsert a view record — safe to call multiple times (idempotent via PRIMARY KEY).
///
/// The `viewer_identity_id` and `viewer_alias` are the viewer's private Kratos
/// UUID and their app-scoped alias respectively.
pub async fn record_view(
    pool: &PgPool,
    story_id: Uuid,
    viewer_identity_id: Uuid,
    viewer_alias: &str,
) -> Result<(), NyxError> {
    sqlx::query(
        r#"
        INSERT INTO uzume.story_views (story_id, viewer_identity_id, viewer_alias)
        VALUES ($1, $2, $3)
        ON CONFLICT (story_id, viewer_identity_id) DO NOTHING
        "#,
    )
    .bind(story_id)
    .bind(viewer_identity_id)
    .bind(viewer_alias)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(())
}

/// Fetch paginated viewers for a story (only the story author should call this).
///
/// Sorted by `viewed_at DESC`. Fetch `query_limit` rows and truncate to detect
/// whether more pages exist.
pub async fn get_viewers(
    pool: &PgPool,
    story_id: Uuid,
    cursor_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    cursor_viewer_id: Option<Uuid>,
    query_limit: i64,
) -> Result<Vec<StoryViewRow>, NyxError> {
    match (cursor_viewed_at, cursor_viewer_id) {
        (Some(ts), Some(vid)) => sqlx::query_as::<_, StoryViewRow>(
            r#"
                SELECT story_id, viewer_identity_id, viewer_alias, viewed_at
                FROM uzume.story_views
                WHERE story_id = $1
                  AND (viewed_at, viewer_identity_id) < ($2, $3)
                ORDER BY viewed_at DESC, viewer_identity_id DESC
                LIMIT $4
                "#,
        )
        .bind(story_id)
        .bind(ts)
        .bind(vid)
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
        _ => sqlx::query_as::<_, StoryViewRow>(
            r#"
                SELECT story_id, viewer_identity_id, viewer_alias, viewed_at
                FROM uzume.story_views
                WHERE story_id = $1
                ORDER BY viewed_at DESC, viewer_identity_id DESC
                LIMIT $2
                "#,
        )
        .bind(story_id)
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
    }
}

/// Return the number of distinct viewers for a story.
pub async fn get_view_count(pool: &PgPool, story_id: Uuid) -> Result<i64, NyxError> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM uzume.story_views WHERE story_id = $1
        "#,
    )
    .bind(story_id)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(count)
}
