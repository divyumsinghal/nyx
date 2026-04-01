//! Raw SQL queries for `uzume.post_likes`.

use sqlx::PgPool;
use uuid::Uuid;

use nun::NyxError;

/// Insert a like row. Does nothing if the like already exists (idempotent).
pub async fn like_post(
    pool: &PgPool,
    post_id: Uuid,
    liker_identity_id: Uuid,
    liker_alias: &str,
) -> Result<(), NyxError> {
    sqlx::query(
        r#"
        INSERT INTO uzume.post_likes (post_id, liker_alias, liker_identity_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (post_id, liker_identity_id) DO NOTHING
        "#,
    )
    .bind(post_id)
    .bind(liker_alias)
    .bind(liker_identity_id)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;

    // Increment the denormalised like_count on the post.
    sqlx::query(
        r#"
        UPDATE uzume.posts
        SET like_count = like_count + 1
        WHERE id = $1
          AND NOT EXISTS (
              SELECT 1 FROM uzume.post_likes
              WHERE post_id = $1 AND liker_identity_id = $2
              -- The INSERT above may have been a no-op; only increment on a new row.
          )
        "#,
    )
    .bind(post_id)
    .bind(liker_identity_id)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;

    Ok(())
}

/// Remove a like row. Does nothing if the like does not exist (idempotent).
pub async fn unlike_post(
    pool: &PgPool,
    post_id: Uuid,
    liker_identity_id: Uuid,
) -> Result<(), NyxError> {
    let result = sqlx::query(
        r#"
        DELETE FROM uzume.post_likes
        WHERE post_id = $1 AND liker_identity_id = $2
        "#,
    )
    .bind(post_id)
    .bind(liker_identity_id)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;

    if result.rows_affected() > 0 {
        sqlx::query(
            r#"
            UPDATE uzume.posts
            SET like_count = GREATEST(like_count - 1, 0)
            WHERE id = $1
            "#,
        )
        .bind(post_id)
        .execute(pool)
        .await
        .map_err(NyxError::from)?;
    }

    Ok(())
}

/// Return the current like count for a post (re-read from DB).
pub async fn get_like_count(pool: &PgPool, post_id: Uuid) -> Result<i64, NyxError> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM uzume.post_likes WHERE post_id = $1
        "#,
    )
    .bind(post_id)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)?;

    Ok(row.0)
}

/// Return `true` if `liker_identity_id` has liked the given post.
pub async fn has_liked(
    pool: &PgPool,
    post_id: Uuid,
    liker_identity_id: Uuid,
) -> Result<bool, NyxError> {
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM uzume.post_likes
            WHERE post_id = $1 AND liker_identity_id = $2
        )
        "#,
    )
    .bind(post_id)
    .bind(liker_identity_id)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)?;

    Ok(row.0)
}
