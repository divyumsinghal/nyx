//! Raw SQL queries for the `"Uzume".reels`, `"Uzume".reel_likes`, and
//! `"Uzume".reel_views` tables.

use sqlx::PgPool;
use uuid::Uuid;

use nun::NyxError;

use crate::models::reel::{ReelInsert, ReelLike, ReelRow, ReelView};

const SELECT_COLS: &str = r#"
    id, author_profile_id, caption, hashtags,
    raw_key, media_key, thumbnail_key,
    duration_ms, processing_state,
    audio_id, audio_start_ms,
    view_count, like_count, comment_count, share_count,
    score, created_at, updated_at
"#;

/// Insert a new reel with `processing_state = 'pending'`.
pub async fn create_reel(pool: &PgPool, insert: &ReelInsert) -> Result<ReelRow, NyxError> {
    sqlx::query_as::<_, ReelRow>(&format!(
        r#"
        INSERT INTO "Uzume".reels
            (id, author_profile_id, caption, hashtags, raw_key, duration_ms,
             audio_id, audio_start_ms)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING {SELECT_COLS}
        "#
    ))
    .bind(insert.id)
    .bind(insert.author_profile_id)
    .bind(&insert.caption)
    .bind(&insert.hashtags)
    .bind(&insert.raw_key)
    .bind(insert.duration_ms)
    .bind(insert.audio_id)
    .bind(insert.audio_start_ms)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch a single reel by its UUID. Returns `None` if not found.
pub async fn get_reel_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ReelRow>, NyxError> {
    sqlx::query_as::<_, ReelRow>(&format!(
        r#"
        SELECT {SELECT_COLS}
        FROM "Uzume".reels
        WHERE id = $1
        "#
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Soft-delete a reel by setting `processing_state = 'failed'` and clearing
/// media keys, but only if the reel is owned by `author_profile_id`.
///
/// Returns `true` if the reel was found and deleted, `false` otherwise.
pub async fn delete_reel(
    pool: &PgPool,
    reel_id: Uuid,
    author_profile_id: Uuid,
) -> Result<bool, NyxError> {
    let result = sqlx::query(
        r#"
        UPDATE "Uzume".reels
        SET processing_state = 'failed',
            media_key = NULL,
            thumbnail_key = NULL,
            updated_at = NOW()
        WHERE id = $1
          AND author_profile_id = $2
          AND processing_state != 'failed'
        "#,
    )
    .bind(reel_id)
    .bind(author_profile_id)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(result.rows_affected() > 0)
}

/// Like a reel. Idempotent — returns `Conflict` if already liked.
pub async fn like_reel(
    pool: &PgPool,
    reel_id: Uuid,
    profile_id: Uuid,
) -> Result<ReelLike, NyxError> {
    // Insert like row and increment denormalized counter atomically.
    let mut tx = pool.begin().await.map_err(NyxError::from)?;

    let like = sqlx::query_as::<_, ReelLike>(
        r#"
        INSERT INTO "Uzume".reel_likes (reel_id, profile_id)
        VALUES ($1, $2)
        RETURNING reel_id, profile_id, created_at
        "#,
    )
    .bind(reel_id)
    .bind(profile_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(NyxError::from)?;

    sqlx::query(
        r#"
        UPDATE "Uzume".reels
        SET like_count = like_count + 1, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(reel_id)
    .execute(&mut *tx)
    .await
    .map_err(NyxError::from)?;

    tx.commit().await.map_err(NyxError::from)?;
    Ok(like)
}

/// Remove a like from a reel.
///
/// Returns `Ok(())` whether or not the like existed (idempotent removal).
pub async fn unlike_reel(
    pool: &PgPool,
    reel_id: Uuid,
    profile_id: Uuid,
) -> Result<(), NyxError> {
    let mut tx = pool.begin().await.map_err(NyxError::from)?;

    let result = sqlx::query(
        r#"
        DELETE FROM "Uzume".reel_likes
        WHERE reel_id = $1 AND profile_id = $2
        "#,
    )
    .bind(reel_id)
    .bind(profile_id)
    .execute(&mut *tx)
    .await
    .map_err(NyxError::from)?;

    if result.rows_affected() > 0 {
        sqlx::query(
            r#"
            UPDATE "Uzume".reels
            SET like_count = GREATEST(like_count - 1, 0), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(reel_id)
        .execute(&mut *tx)
        .await
        .map_err(NyxError::from)?;
    }

    tx.commit().await.map_err(NyxError::from)?;
    Ok(())
}

/// Record a reel view and increment the view counter.
///
/// A view is always appended (no deduplication at DB level). The caller should
/// apply business rules (e.g. minimum watch percent) before calling this.
pub async fn record_view(
    pool: &PgPool,
    reel_id: Uuid,
    viewer_profile_id: Uuid,
    watch_percent: i16,
) -> Result<ReelView, NyxError> {
    let mut tx = pool.begin().await.map_err(NyxError::from)?;

    let view = sqlx::query_as::<_, ReelView>(
        r#"
        INSERT INTO "Uzume".reel_views (reel_id, viewer_profile_id, watch_percent)
        VALUES ($1, $2, $3)
        RETURNING id, reel_id, viewer_profile_id, watch_percent, viewed_at
        "#,
    )
    .bind(reel_id)
    .bind(viewer_profile_id)
    .bind(watch_percent)
    .fetch_one(&mut *tx)
    .await
    .map_err(NyxError::from)?;

    // Only count as a view for engagement scoring if watch percent ≥ 25%
    if watch_percent >= 25 {
        sqlx::query(
            r#"
            UPDATE "Uzume".reels
            SET view_count = view_count + 1, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(reel_id)
        .execute(&mut *tx)
        .await
        .map_err(NyxError::from)?;
    }

    tx.commit().await.map_err(NyxError::from)?;
    Ok(view)
}

/// Update the algorithmic ranking score for a reel.
pub async fn update_reel_score(
    pool: &PgPool,
    reel_id: Uuid,
    score: f64,
) -> Result<(), NyxError> {
    sqlx::query(
        r#"
        UPDATE "Uzume".reels
        SET score = $2, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(reel_id)
    .bind(score)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(())
}

/// Fetch the algorithmic reel feed, sorted by score DESC then created_at DESC.
///
/// Only returns reels with `processing_state = 'ready'`. Cursor is
/// `(score, id)` for stable keyset pagination.
pub async fn get_reel_feed(
    pool: &PgPool,
    cursor_score: Option<f64>,
    cursor_id: Option<Uuid>,
    query_limit: i64,
) -> Result<Vec<ReelRow>, NyxError> {
    match (cursor_score, cursor_id) {
        (Some(score), Some(cid)) => sqlx::query_as::<_, ReelRow>(&format!(
            r#"
            SELECT {SELECT_COLS}
            FROM "Uzume".reels
            WHERE processing_state = 'ready'
              AND (score, id) < ($1, $2)
            ORDER BY score DESC, id DESC
            LIMIT $3
            "#
        ))
        .bind(score)
        .bind(cid)
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
        _ => sqlx::query_as::<_, ReelRow>(&format!(
            r#"
            SELECT {SELECT_COLS}
            FROM "Uzume".reels
            WHERE processing_state = 'ready'
            ORDER BY score DESC, id DESC
            LIMIT $1
            "#
        ))
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
    }
}

/// Update `media_key` and `thumbnail_key` after Oya completes transcoding.
///
/// Transitions the reel from `pending` / `processing` to `ready`.
/// Idempotent — if already `ready` with the same keys, this is a no-op.
pub async fn update_reel_media_urls(
    pool: &PgPool,
    reel_id: Uuid,
    media_key: &str,
    thumbnail_key: &str,
) -> Result<Option<ReelRow>, NyxError> {
    sqlx::query_as::<_, ReelRow>(&format!(
        r#"
        UPDATE "Uzume".reels
        SET media_key = $2,
            thumbnail_key = $3,
            processing_state = 'ready',
            updated_at = NOW()
        WHERE id = $1
          AND processing_state IN ('pending', 'processing')
        RETURNING {SELECT_COLS}
        "#
    ))
    .bind(reel_id)
    .bind(media_key)
    .bind(thumbnail_key)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch all reels authored by a given profile, cursor-paginated by creation time.
pub async fn get_reels_by_author(
    pool: &PgPool,
    author_profile_id: Uuid,
    cursor_created_at: Option<chrono::DateTime<chrono::Utc>>,
    cursor_id: Option<Uuid>,
    query_limit: i64,
) -> Result<Vec<ReelRow>, NyxError> {
    match (cursor_created_at, cursor_id) {
        (Some(ts), Some(cid)) => sqlx::query_as::<_, ReelRow>(&format!(
            r#"
            SELECT {SELECT_COLS}
            FROM "Uzume".reels
            WHERE author_profile_id = $1
              AND processing_state = 'ready'
              AND (created_at, id) < ($2, $3)
            ORDER BY created_at DESC, id DESC
            LIMIT $4
            "#
        ))
        .bind(author_profile_id)
        .bind(ts)
        .bind(cid)
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
        _ => sqlx::query_as::<_, ReelRow>(&format!(
            r#"
            SELECT {SELECT_COLS}
            FROM "Uzume".reels
            WHERE author_profile_id = $1
              AND processing_state = 'ready'
            ORDER BY created_at DESC, id DESC
            LIMIT $2
            "#
        ))
        .bind(author_profile_id)
        .bind(query_limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_cols_includes_required_fields() {
        assert!(SELECT_COLS.contains("author_profile_id"));
        assert!(SELECT_COLS.contains("caption"));
        assert!(SELECT_COLS.contains("processing_state"));
        assert!(SELECT_COLS.contains("score"));
        assert!(SELECT_COLS.contains("like_count"));
        assert!(SELECT_COLS.contains("view_count"));
    }
}
