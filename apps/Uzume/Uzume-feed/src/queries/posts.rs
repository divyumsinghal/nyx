//! Raw SQL queries for `uzume.posts`.
//!
//! All list queries use the "fetch one extra" pattern for cursor-based
//! pagination: fetch `limit + 1` rows, check overflow, truncate to `limit`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::post::{PostInsert, PostRow};
use nun::NyxError;

/// Fetch a single post by its UUID.
///
/// Returns `None` when no matching post exists.
pub async fn get_post_by_id(pool: &PgPool, id: Uuid) -> Result<Option<PostRow>, NyxError> {
    sqlx::query_as::<_, PostRow>(
        r#"
        SELECT id, identity_id, author_alias, caption, like_count, comment_count, created_at
        FROM uzume.posts
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Insert a new post row and return it.
pub async fn create_post(pool: &PgPool, insert: &PostInsert) -> Result<PostRow, NyxError> {
    sqlx::query_as::<_, PostRow>(
        r#"
        INSERT INTO uzume.posts (id, identity_id, author_alias, caption)
        VALUES ($1, $2, $3, $4)
        RETURNING id, identity_id, author_alias, caption, like_count, comment_count, created_at
        "#,
    )
    .bind(insert.id)
    .bind(insert.identity_id)
    .bind(&insert.author_alias)
    .bind(&insert.caption)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)
}

/// Delete a post by its UUID.
///
/// Returns `true` if the row was deleted, `false` if no row matched.
pub async fn delete_post(pool: &PgPool, id: Uuid) -> Result<bool, NyxError> {
    let result = sqlx::query("DELETE FROM uzume.posts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(NyxError::from)?;

    Ok(result.rows_affected() > 0)
}

/// Fetch the global chronological home timeline (newest first).
///
/// Uses cursor-based pagination via `(created_at, id)` keyset. Pass
/// `None` for both cursor fields to get the first page.
pub async fn get_home_timeline(
    pool: &PgPool,
    after_ts: Option<DateTime<Utc>>,
    after_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<PostRow>, NyxError> {
    match (after_ts, after_id) {
        (Some(ts), Some(id)) => {
            sqlx::query_as::<_, PostRow>(
                r#"
                SELECT id, identity_id, author_alias, caption, like_count, comment_count, created_at
                FROM uzume.posts
                WHERE (created_at, id) < ($1, $2)
                ORDER BY created_at DESC, id DESC
                LIMIT $3
                "#,
            )
            .bind(ts)
            .bind(id)
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(NyxError::from)
        }
        _ => {
            sqlx::query_as::<_, PostRow>(
                r#"
                SELECT id, identity_id, author_alias, caption, like_count, comment_count, created_at
                FROM uzume.posts
                ORDER BY created_at DESC, id DESC
                LIMIT $1
                "#,
            )
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(NyxError::from)
        }
    }
}

/// Fetch posts by a specific author alias (their user timeline), newest first.
///
/// Uses cursor-based pagination via `(created_at, id)` keyset.
pub async fn get_user_timeline(
    pool: &PgPool,
    author_alias: &str,
    after_ts: Option<DateTime<Utc>>,
    after_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<PostRow>, NyxError> {
    match (after_ts, after_id) {
        (Some(ts), Some(id)) => {
            sqlx::query_as::<_, PostRow>(
                r#"
                SELECT id, identity_id, author_alias, caption, like_count, comment_count, created_at
                FROM uzume.posts
                WHERE author_alias = $1
                  AND (created_at, id) < ($2, $3)
                ORDER BY created_at DESC, id DESC
                LIMIT $4
                "#,
            )
            .bind(author_alias)
            .bind(ts)
            .bind(id)
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(NyxError::from)
        }
        _ => {
            sqlx::query_as::<_, PostRow>(
                r#"
                SELECT id, identity_id, author_alias, caption, like_count, comment_count, created_at
                FROM uzume.posts
                WHERE author_alias = $1
                ORDER BY created_at DESC, id DESC
                LIMIT $2
                "#,
            )
            .bind(author_alias)
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(NyxError::from)
        }
    }
}
