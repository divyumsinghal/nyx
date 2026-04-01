//! Raw SQL queries for the `uzume.highlights` and `uzume.highlight_stories` tables.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::highlight::{HighlightInsert, HighlightRow};
use nun::NyxError;

/// Insert a new highlight row.
pub async fn create_highlight(
    pool: &PgPool,
    insert: &HighlightInsert,
) -> Result<HighlightRow, NyxError> {
    sqlx::query_as::<_, HighlightRow>(
        r#"
        INSERT INTO uzume.highlights (id, owner_identity_id, owner_alias, title)
        VALUES ($1, $2, $3, $4)
        RETURNING id, owner_identity_id, owner_alias, title, cover_url, created_at
        "#,
    )
    .bind(insert.id)
    .bind(insert.owner_identity_id)
    .bind(&insert.owner_alias)
    .bind(&insert.title)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch a highlight by its UUID.
///
/// Returns `None` if not found.
pub async fn get_highlight_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<HighlightRow>, NyxError> {
    sqlx::query_as::<_, HighlightRow>(
        r#"
        SELECT id, owner_identity_id, owner_alias, title, cover_url, created_at
        FROM uzume.highlights
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch all highlights owned by the given alias.
pub async fn get_highlights_for_user(
    pool: &PgPool,
    owner_alias: &str,
) -> Result<Vec<HighlightRow>, NyxError> {
    sqlx::query_as::<_, HighlightRow>(
        r#"
        SELECT id, owner_identity_id, owner_alias, title, cover_url, created_at
        FROM uzume.highlights
        WHERE owner_alias = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(owner_alias)
    .fetch_all(pool)
    .await
    .map_err(NyxError::from)
}

/// Add a story to a highlight.
///
/// Uses `ON CONFLICT DO NOTHING` so the operation is idempotent.
pub async fn add_story_to_highlight(
    pool: &PgPool,
    highlight_id: Uuid,
    story_id: Uuid,
) -> Result<(), NyxError> {
    sqlx::query(
        r#"
        INSERT INTO uzume.highlight_stories (highlight_id, story_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(highlight_id)
    .bind(story_id)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(())
}

/// Remove a story from a highlight.
///
/// Returns `true` if the association existed and was removed.
pub async fn remove_story_from_highlight(
    pool: &PgPool,
    highlight_id: Uuid,
    story_id: Uuid,
) -> Result<bool, NyxError> {
    let result = sqlx::query(
        r#"
        DELETE FROM uzume.highlight_stories
        WHERE highlight_id = $1 AND story_id = $2
        "#,
    )
    .bind(highlight_id)
    .bind(story_id)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(result.rows_affected() > 0)
}

/// Delete a highlight and all its story associations (via cascade).
///
/// Returns `true` if the highlight existed and was deleted.
pub async fn delete_highlight(
    pool: &PgPool,
    id: Uuid,
    owner_identity_id: Uuid,
) -> Result<bool, NyxError> {
    let result = sqlx::query(
        r#"
        DELETE FROM uzume.highlights
        WHERE id = $1 AND owner_identity_id = $2
        "#,
    )
    .bind(id)
    .bind(owner_identity_id)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(result.rows_affected() > 0)
}
