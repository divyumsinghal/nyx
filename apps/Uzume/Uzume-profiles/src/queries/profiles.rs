//! Raw SQL queries for the `uzume.profiles` table.
//!
//! All functions use `sqlx::query_as` for type-safe row mapping. The
//! `sqlx::query_as!` compile-time macro variant requires a live database URL
//! at build time — use `SQLX_OFFLINE=true` with a populated `.sqlx/` cache,
//! or run `cargo sqlx prepare` before building in CI.
//!
//! Currently we use `sqlx::query_as::<_, Row>(...)` with explicit type
//! annotation instead of the compile-time macro so the crate builds without
//! a live database.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::profile::{ProfileInsert, ProfileRow, ProfileUpdate};
use nun::NyxError;

/// Fetch a profile by its app-scoped alias.
///
/// Returns `None` when no matching profile exists.
pub async fn get_profile_by_alias(
    pool: &PgPool,
    alias: &str,
) -> Result<Option<ProfileRow>, NyxError> {
    sqlx::query_as::<_, ProfileRow>(
        r"
        SELECT id, identity_id, alias, display_name, bio, avatar_url,
               is_private, is_verified,
               follower_count, following_count, post_count,
               created_at, updated_at
        FROM uzume.profiles
        WHERE alias = $1
         ",
    )
    .bind(alias)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch a profile by its internal UUID.
///
/// Returns `None` when no matching profile exists.
pub async fn get_profile_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ProfileRow>, NyxError> {
    sqlx::query_as::<_, ProfileRow>(
        r"
        SELECT id, identity_id, alias, display_name, bio, avatar_url,
               is_private, is_verified,
               follower_count, following_count, post_count,
               created_at, updated_at
        FROM uzume.profiles
        WHERE id = $1
         ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch a profile by the Kratos identity UUID.
///
/// Returns `None` when the identity has no profile in Uzume yet.
pub async fn get_profile_by_identity(
    pool: &PgPool,
    identity_id: Uuid,
) -> Result<Option<ProfileRow>, NyxError> {
    sqlx::query_as::<_, ProfileRow>(
        r"
        SELECT id, identity_id, alias, display_name, bio, avatar_url,
               is_private, is_verified,
               follower_count, following_count, post_count,
               created_at, updated_at
        FROM uzume.profiles
        WHERE identity_id = $1
         ",
    )
    .bind(identity_id)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Insert a new profile row.
///
/// The `identity_id` must match an existing Kratos identity. The `alias` must
/// be unique within `uzume.profiles`.
pub async fn create_profile(pool: &PgPool, insert: &ProfileInsert) -> Result<ProfileRow, NyxError> {
    sqlx::query_as::<_, ProfileRow>(
        r"
        INSERT INTO uzume.profiles (id, identity_id, alias, display_name)
        VALUES ($1, $2, $3, $4)
        RETURNING id, identity_id, alias, display_name, bio, avatar_url,
                  is_private, is_verified,
                  follower_count, following_count, post_count,
                  created_at, updated_at
        ",
    )
    .bind(insert.id)
    .bind(insert.identity_id)
    .bind(&insert.alias)
    .bind(&insert.display_name)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)
}

/// Apply a partial update to an existing profile.
///
/// Only non-`None` fields in `update` are written. The `updated_at` column is
/// set to `now()` by the query.
pub async fn update_profile(
    pool: &PgPool,
    id: Uuid,
    update: &ProfileUpdate,
) -> Result<ProfileRow, NyxError> {
    sqlx::query_as::<_, ProfileRow>(
        r"
        UPDATE uzume.profiles
        SET display_name  = COALESCE($2, display_name),
            bio           = COALESCE($3, bio),
            avatar_url    = COALESCE($4, avatar_url),
            is_private    = COALESCE($5, is_private),
            updated_at    = now()
        WHERE id = $1
        RETURNING id, identity_id, alias, display_name, bio, avatar_url,
                  is_private, is_verified,
                  follower_count, following_count, post_count,
                  created_at, updated_at
        ",
    )
    .bind(id)
    .bind(&update.display_name)
    .bind(&update.bio)
    .bind(&update.avatar_url)
    .bind(update.is_private)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)
}
