//! HTTP handlers for the reels resource.
//!
//! All mutation endpoints require auth (enforced by the router's middleware
//! layer). Read endpoints may be public or auth-optional.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use nun::{Cursor, NyxError};
use nyx_api::{ApiResponse, AuthUser, CursorPagination, ValidatedJson};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    handlers::ApiError,
    models::reel::{CreateReelRequest, RecordViewRequest, ReelInsert, ReelResponse},
    queries::reels as reel_queries,
    services::reels::{build_reel_feed_page, ensure_reel_owner},
    state::AppState,
};

/// `POST /reels` — Create a new reel (authenticated).
///
/// Validates the request body, inserts a new reel with `processing_state = pending`,
/// and publishes a `Uzume.reel.created` event so Oya can begin transcoding.
#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn create_reel(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<CreateReelRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let insert = ReelInsert {
        id: Uuid::now_v7(),
        author_profile_id: user.user_id,
        caption: body.caption.unwrap_or_default(),
        hashtags: body.hashtags.unwrap_or_default(),
        raw_key: body.raw_key,
        duration_ms: body.duration_ms,
        audio_id: body.audio_id,
        audio_start_ms: body.audio_start_ms.unwrap_or(0),
    };

    let row = reel_queries::create_reel(&state.db, &insert).await?;

    // If audio track referenced, increment its use_count.
    if let Some(audio_id) = insert.audio_id {
        if let Err(err) =
            crate::queries::audio::increment_audio_use_count(&state.db, audio_id).await
        {
            tracing::warn!(?err, %audio_id, "failed to increment audio use_count");
        }
    }

    // Publish reel.created event for Oya transcoding.
    let event_payload = serde_json::json!({
        "reel_id": row.id,
        "author_profile_id": row.author_profile_id,
        "raw_key": row.raw_key,
        "duration_ms": row.duration_ms,
    });
    if let Err(err) = state
        .nats
        .publish_raw(
            nyx_events::subjects::UZUME_REEL_CREATED.to_string(),
            serde_json::to_vec(&event_payload).unwrap_or_default(),
        )
        .await
    {
        tracing::warn!(?err, reel_id = %row.id, "failed to publish Uzume.reel.created");
    }

    Ok((StatusCode::CREATED, ApiResponse::ok(ReelResponse::from(row))))
}

/// `GET /reels/:id` — Fetch a single reel by ID (public).
#[instrument(skip(state), fields(reel_id = %reel_id))]
pub async fn get_reel(
    State(state): State<AppState>,
    Path(reel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let row = reel_queries::get_reel_by_id(&state.db, reel_id)
        .await?
        .ok_or_else(|| NyxError::not_found("reel_not_found", "Reel not found"))?;

    Ok(ApiResponse::ok(ReelResponse::from(row)))
}

/// `DELETE /reels/:id` — Soft-delete a reel (author only).
#[instrument(skip(state), fields(reel_id = %reel_id, user_id = %user.user_id))]
pub async fn delete_reel(
    State(state): State<AppState>,
    user: AuthUser,
    Path(reel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify ownership before attempting the delete.
    let row = reel_queries::get_reel_by_id(&state.db, reel_id)
        .await?
        .ok_or_else(|| NyxError::not_found("reel_not_found", "Reel not found"))?;

    ensure_reel_owner(&row, user.user_id)?;

    let deleted = reel_queries::delete_reel(&state.db, reel_id, user.user_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(NyxError::not_found("reel_not_found", "Reel not found").into())
    }
}

/// `GET /reels/feed` — Algorithmic reel feed, cursor-paginated by score (authenticated).
#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn get_feed(
    State(state): State<AppState>,
    user: AuthUser,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let _ = user; // Future: personalise feed based on user preferences / blocks.

    let (cursor_score, cursor_id) = decode_score_cursor_opt(page.cursor.as_deref())?;

    let rows =
        reel_queries::get_reel_feed(&state.db, cursor_score, cursor_id, page.query_limit())
            .await?;

    let page_resp = build_reel_feed_page(rows, page.effective_limit());
    let next = page_resp.next_cursor.clone();
    let has_more = page_resp.has_more;

    Ok(ApiResponse::paginated(page_resp, next, has_more))
}

/// `POST /reels/:id/like` — Like a reel (authenticated).
#[instrument(skip(state), fields(reel_id = %reel_id, user_id = %user.user_id))]
pub async fn like_reel(
    State(state): State<AppState>,
    user: AuthUser,
    Path(reel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify reel exists.
    reel_queries::get_reel_by_id(&state.db, reel_id)
        .await?
        .ok_or_else(|| NyxError::not_found("reel_not_found", "Reel not found"))?;

    let _like = reel_queries::like_reel(&state.db, reel_id, user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /reels/:id/like` — Unlike a reel (authenticated).
#[instrument(skip(state), fields(reel_id = %reel_id, user_id = %user.user_id))]
pub async fn unlike_reel(
    State(state): State<AppState>,
    user: AuthUser,
    Path(reel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    reel_queries::unlike_reel(&state.db, reel_id, user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /reels/:id/view` — Record a view with watch percentage (authenticated).
///
/// Views with `watch_percent < 10` are recorded but do not increment `view_count`
/// (scroll-past detection). The SQL layer handles the threshold at 25%.
#[instrument(skip(state), fields(reel_id = %reel_id, user_id = %user.user_id))]
pub async fn record_view(
    State(state): State<AppState>,
    user: AuthUser,
    Path(reel_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<RecordViewRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify reel exists and is ready.
    let row = reel_queries::get_reel_by_id(&state.db, reel_id)
        .await?
        .ok_or_else(|| NyxError::not_found("reel_not_found", "Reel not found"))?;

    if row.processing_state != "ready" {
        return Err(NyxError::bad_request(
            "reel_not_ready",
            "Reel is not yet available for viewing",
        )
        .into());
    }

    reel_queries::record_view(&state.db, reel_id, user.user_id, body.watch_percent).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn decode_score_cursor_opt(
    cursor: Option<&str>,
) -> Result<(Option<f64>, Option<Uuid>), NyxError> {
    match cursor {
        None => Ok((None, None)),
        Some(s) => {
            let c = Cursor::decode(s)?;
            let (score, id) = c.as_score_id()?;
            Ok((Some(score), Some(id)))
        }
    }
}
