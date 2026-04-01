//! HTTP handlers for the reel audio resource.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use nun::NyxError;
use nyx_api::{ApiResponse, ValidatedJson};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    handlers::ApiError,
    models::reel_audio::{CreateReelAudioRequest, ReelAudioInsert, ReelAudioResponse},
    queries::audio as audio_queries,
    state::AppState,
};

/// `POST /audio` — Create an audio track (authenticated via outer middleware).
#[instrument(skip(state))]
pub async fn create_audio(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateReelAudioRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let insert = ReelAudioInsert {
        id: Uuid::now_v7(),
        title: body.title,
        artist_name: body.artist_name,
        original_reel_id: body.original_reel_id,
        audio_key: body.audio_key,
        duration_ms: body.duration_ms,
    };

    let row = audio_queries::create_audio(&state.db, &insert).await?;

    Ok((
        StatusCode::CREATED,
        ApiResponse::ok(ReelAudioResponse::from(row)),
    ))
}

/// `GET /audio/:id` — Fetch a single audio track by ID (public).
#[instrument(skip(state), fields(audio_id = %audio_id))]
pub async fn get_audio(
    State(state): State<AppState>,
    Path(audio_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let row = audio_queries::get_audio_by_id(&state.db, audio_id)
        .await?
        .ok_or_else(|| NyxError::not_found("audio_not_found", "Audio track not found"))?;

    Ok(ApiResponse::ok(ReelAudioResponse::from(row)))
}

/// `GET /audio/trending` — Fetch the trending audio tracks (public).
///
/// Returns the top 50 most-used audio tracks, sorted by `use_count DESC`.
#[instrument(skip(state))]
pub async fn list_trending_audio(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let rows = audio_queries::list_trending_audio(&state.db, 50).await?;
    let items: Vec<ReelAudioResponse> = rows.into_iter().map(ReelAudioResponse::from).collect();
    Ok(ApiResponse::ok(items))
}
