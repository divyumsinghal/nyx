//! Raw SQL queries for the `uzume.follows` table.
//!
//! All list queries use the "fetch one extra" cursor pattern — they fetch
//! `limit + 1` rows so the caller can determine whether more pages exist.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::follow::{FollowProfileRow, FollowRow, FollowStatus};
use nun::NyxError;

/// Record a follow relationship.
///
/// Silently succeeds if the relationship already exists (idempotent via
/// `ON CONFLICT DO NOTHING`). Counter columns are updated atomically in the
/// same transaction.
pub async fn follow(pool: &PgPool, follower_id: Uuid, followee_id: Uuid) -> Result<(), NyxError> {
    let mut tx = pool.begin().await.map_err(NyxError::from)?;

    sqlx::query(
        r#"
        INSERT INTO uzume.follows (follower_id, followee_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(follower_id)
    .bind(followee_id)
    .execute(&mut *tx)
    .await
    .map_err(NyxError::from)?;

    // Increment following_count for the follower.
    sqlx::query(
        r#"
        UPDATE uzume.profiles
        SET following_count = following_count + 1
        WHERE id = $1
        "#,
    )
    .bind(follower_id)
    .execute(&mut *tx)
    .await
    .map_err(NyxError::from)?;

    // Increment follower_count for the followee.
    sqlx::query(
        r#"
        UPDATE uzume.profiles
        SET follower_count = follower_count + 1
        WHERE id = $1
        "#,
    )
    .bind(followee_id)
    .execute(&mut *tx)
    .await
    .map_err(NyxError::from)?;

    tx.commit().await.map_err(NyxError::from)?;
    Ok(())
}

/// Remove a follow relationship.
///
/// Silently succeeds if the relationship does not exist.
pub async fn unfollow(
    pool: &PgPool,
    follower_id: Uuid,
    followee_id: Uuid,
) -> Result<(), NyxError> {
    let mut tx = pool.begin().await.map_err(NyxError::from)?;

    let result = sqlx::query(
        r#"
        DELETE FROM uzume.follows
        WHERE follower_id = $1 AND followee_id = $2
        "#,
    )
    .bind(follower_id)
    .bind(followee_id)
    .execute(&mut *tx)
    .await
    .map_err(NyxError::from)?;

    if result.rows_affected() > 0 {
        // Only adjust counters when a row was actually deleted.
        sqlx::query(
            r#"
            UPDATE uzume.profiles
            SET following_count = GREATEST(0, following_count - 1)
            WHERE id = $1
            "#,
        )
        .bind(follower_id)
        .execute(&mut *tx)
        .await
        .map_err(NyxError::from)?;

        sqlx::query(
            r#"
            UPDATE uzume.profiles
            SET follower_count = GREATEST(0, follower_count - 1)
            WHERE id = $1
            "#,
        )
        .bind(followee_id)
        .execute(&mut *tx)
        .await
        .map_err(NyxError::from)?;
    }

    tx.commit().await.map_err(NyxError::from)?;
    Ok(())
}

/// Return whether `follower_id` follows `followee_id`.
pub async fn get_follow_status(
    pool: &PgPool,
    follower_id: Uuid,
    followee_id: Uuid,
) -> Result<FollowStatus, NyxError> {
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM uzume.follows
            WHERE follower_id = $1 AND followee_id = $2
        )
        "#,
    )
    .bind(follower_id)
    .bind(followee_id)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)?;

    if row.0 {
        Ok(FollowStatus::Following)
    } else {
        Ok(FollowStatus::NotFollowing)
    }
}

/// List profiles that follow the given `profile_id`.
///
/// Sorted by `follows.created_at DESC`. Uses cursor-based pagination:
/// fetch `limit + 1` rows and the caller determines `has_more`.
pub async fn get_followers(
    pool: &PgPool,
    profile_id: Uuid,
    after_created_at: Option<chrono::DateTime<chrono::Utc>>,
    after_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<FollowProfileRow>, NyxError> {
    if let (Some(after_ts), Some(after_uuid)) = (after_created_at, after_id) {
        sqlx::query_as::<_, FollowProfileRow>(
            r#"
            SELECT p.id, p.alias, p.display_name, p.avatar_url, p.is_verified,
                   f.created_at
            FROM uzume.follows f
            JOIN uzume.profiles p ON p.id = f.follower_id
            WHERE f.followee_id = $1
              AND (f.created_at, f.follower_id) < ($2, $3)
            ORDER BY f.created_at DESC, f.follower_id DESC
            LIMIT $4
            "#,
        )
        .bind(profile_id)
        .bind(after_ts)
        .bind(after_uuid)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from)
    } else {
        sqlx::query_as::<_, FollowProfileRow>(
            r#"
            SELECT p.id, p.alias, p.display_name, p.avatar_url, p.is_verified,
                   f.created_at
            FROM uzume.follows f
            JOIN uzume.profiles p ON p.id = f.follower_id
            WHERE f.followee_id = $1
            ORDER BY f.created_at DESC, f.follower_id DESC
            LIMIT $2
            "#,
        )
        .bind(profile_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from)
    }
}

/// List profiles that `profile_id` follows.
///
/// Same cursor pattern as `get_followers`.
pub async fn get_following(
    pool: &PgPool,
    profile_id: Uuid,
    after_created_at: Option<chrono::DateTime<chrono::Utc>>,
    after_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<FollowProfileRow>, NyxError> {
    if let (Some(after_ts), Some(after_uuid)) = (after_created_at, after_id) {
        sqlx::query_as::<_, FollowProfileRow>(
            r#"
            SELECT p.id, p.alias, p.display_name, p.avatar_url, p.is_verified,
                   f.created_at
            FROM uzume.follows f
            JOIN uzume.profiles p ON p.id = f.followee_id
            WHERE f.follower_id = $1
              AND (f.created_at, f.followee_id) < ($2, $3)
            ORDER BY f.created_at DESC, f.followee_id DESC
            LIMIT $4
            "#,
        )
        .bind(profile_id)
        .bind(after_ts)
        .bind(after_uuid)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from)
    } else {
        sqlx::query_as::<_, FollowProfileRow>(
            r#"
            SELECT p.id, p.alias, p.display_name, p.avatar_url, p.is_verified,
                   f.created_at
            FROM uzume.follows f
            JOIN uzume.profiles p ON p.id = f.followee_id
            WHERE f.follower_id = $1
            ORDER BY f.created_at DESC, f.followee_id DESC
            LIMIT $2
            "#,
        )
        .bind(profile_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(NyxError::from)
    }
}

/// Load a single follow row (used for existence checks in tests).
pub async fn get_follow_row(
    pool: &PgPool,
    follower_id: Uuid,
    followee_id: Uuid,
) -> Result<Option<FollowRow>, NyxError> {
    sqlx::query_as::<_, FollowRow>(
        r#"
        SELECT follower_id, followee_id, created_at
        FROM uzume.follows
        WHERE follower_id = $1 AND followee_id = $2
        "#,
    )
    .bind(follower_id)
    .bind(followee_id)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}
