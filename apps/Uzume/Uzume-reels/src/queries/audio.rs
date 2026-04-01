//! Raw SQL queries for the `"Uzume".reel_audio` table.

use sqlx::PgPool;
use uuid::Uuid;

use nun::NyxError;

use crate::models::reel_audio::{ReelAudioInsert, ReelAudioRow};

const SELECT_COLS: &str = r#"
    id, title, artist_name, original_reel_id,
    audio_key, duration_ms, use_count, created_at
"#;

/// Insert a new audio track.
pub async fn create_audio(
    pool: &PgPool,
    insert: &ReelAudioInsert,
) -> Result<ReelAudioRow, NyxError> {
    sqlx::query_as::<_, ReelAudioRow>(&format!(
        r#"
        INSERT INTO "Uzume".reel_audio
            (id, title, artist_name, original_reel_id, audio_key, duration_ms)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING {SELECT_COLS}
        "#
    ))
    .bind(insert.id)
    .bind(&insert.title)
    .bind(&insert.artist_name)
    .bind(insert.original_reel_id)
    .bind(&insert.audio_key)
    .bind(insert.duration_ms)
    .fetch_one(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch a single audio track by its UUID. Returns `None` if not found.
pub async fn get_audio_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ReelAudioRow>, NyxError> {
    sqlx::query_as::<_, ReelAudioRow>(&format!(
        r#"
        SELECT {SELECT_COLS}
        FROM "Uzume".reel_audio
        WHERE id = $1
        "#
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(NyxError::from)
}

/// Fetch the trending audio tracks, sorted by `use_count DESC`.
///
/// Trending audio is defined as the most-used audio tracks overall.
/// `limit` controls the number of results.
pub async fn list_trending_audio(pool: &PgPool, limit: i64) -> Result<Vec<ReelAudioRow>, NyxError> {
    sqlx::query_as::<_, ReelAudioRow>(&format!(
        r#"
        SELECT {SELECT_COLS}
        FROM "Uzume".reel_audio
        ORDER BY use_count DESC, created_at DESC
        LIMIT $1
        "#
    ))
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(NyxError::from)
}

/// Increment the `use_count` for an audio track.
///
/// Called when a new reel is created referencing this audio track.
pub async fn increment_audio_use_count(pool: &PgPool, audio_id: Uuid) -> Result<(), NyxError> {
    sqlx::query(
        r#"
        UPDATE "Uzume".reel_audio
        SET use_count = use_count + 1
        WHERE id = $1
        "#,
    )
    .bind(audio_id)
    .execute(pool)
    .await
    .map_err(NyxError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_cols_includes_required_fields() {
        assert!(SELECT_COLS.contains("title"));
        assert!(SELECT_COLS.contains("audio_key"));
        assert!(SELECT_COLS.contains("use_count"));
        assert!(SELECT_COLS.contains("duration_ms"));
        assert!(SELECT_COLS.contains("artist_name"));
    }
}
