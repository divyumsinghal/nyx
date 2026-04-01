//! Raw SQL queries for `uzume.comments`.
//!
//! List queries use the "fetch one extra" pattern for cursor-based pagination.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::comment::{CommentInsert, CommentRow};
use nun::NyxError;

/// Insert a new comment and return it.
pub async fn create_comment(
    pool: &PgPool,
    insert: &CommentInsert,
) -> Result<CommentRow, NyxError> {
    sqlx::query_as::<_, CommentRow>(
        r#"
        INSERT INTO uzume.comments
            (id, post_id, author_alias, author_identity_id, content)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, post_id, author_alias, author_identity_id, content, created_at
        "#,
    )
    .bind(insert.id)
    .bind(insert.post_id)
    .bind(&insert.author_alias)
    .bind(insert.author_identity_id)
    .bind(&insert.content)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch comments for a post, oldest-first within the page, using keyset pagination.
pub async fn get_comments(
    pool: &PgPool,
    post_id: Uuid,
    after_ts: Option<DateTime<Utc>>,
    after_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<CommentRow>, NyxError> {
    match (after_ts, after_id) {
        (Some(ts), Some(id)) => {
            sqlx::query_as::<_, CommentRow>(
                r#"
                SELECT id, post_id, author_alias, author_identity_id, content, created_at
                FROM uzume.comments
                WHERE post_id = $1
                  AND (created_at, id) > ($2, $3)
                ORDER BY created_at ASC, id ASC
                LIMIT $4
                "#,
            )
            .bind(post_id)
            .bind(ts)
            .bind(id)
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(NyxError::from)
        }
        _ => {
            sqlx::query_as::<_, CommentRow>(
                r#"
                SELECT id, post_id, author_alias, author_identity_id, content, created_at
                FROM uzume.comments
                WHERE post_id = $1
                ORDER BY created_at ASC, id ASC
                LIMIT $2
                "#,
            )
            .bind(post_id)
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(NyxError::from)
        }
    }
}

/// Delete a comment by its UUID.
///
/// Returns `true` if the row was deleted, `false` if no row matched.
pub async fn delete_comment(pool: &PgPool, comment_id: Uuid) -> Result<bool, NyxError> {
    let result = sqlx::query("DELETE FROM uzume.comments WHERE id = $1")
        .bind(comment_id)
        .execute(pool)
        .await
        .map_err(NyxError::from)?;

    Ok(result.rows_affected() > 0)
}
