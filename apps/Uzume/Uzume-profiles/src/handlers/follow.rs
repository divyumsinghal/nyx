//! Axum handlers for follow/unfollow endpoints and follower/following lists.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use nyx_api::{ApiResponse, AuthUser, CursorPagination};
use tracing::instrument;

use super::profile::ApiError;
use crate::{
    queries::{follow as follow_queries, profiles as profile_queries},
    services::follow::{build_follow_page, validate_not_self_follow},
    state::AppState,
};
use nun::{Cursor, NyxError};

// ── POST /profiles/:alias/follow ─────────────────────────────────────────────

/// Follow the profile identified by `alias`.
#[instrument(skip(state), fields(alias = %alias, user_id = %user.user_id))]
pub async fn follow_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(alias): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let viewer_profile = profile_queries::get_profile_by_identity(&state.db, user.user_id)
        .await?
        .ok_or_else(|| NyxError::not_found("profile_not_found", "Your profile was not found"))?;

    let target_profile = profile_queries::get_profile_by_alias(&state.db, &alias)
        .await?
        .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile not found"))?;

    validate_not_self_follow(viewer_profile.id, target_profile.id)?;

    follow_queries::follow(&state.db, viewer_profile.id, target_profile.id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ── DELETE /profiles/:alias/follow ───────────────────────────────────────────

/// Unfollow the profile identified by `alias`.
#[instrument(skip(state), fields(alias = %alias, user_id = %user.user_id))]
pub async fn unfollow_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(alias): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let viewer_profile = profile_queries::get_profile_by_identity(&state.db, user.user_id)
        .await?
        .ok_or_else(|| NyxError::not_found("profile_not_found", "Your profile was not found"))?;

    let target_profile = profile_queries::get_profile_by_alias(&state.db, &alias)
        .await?
        .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile not found"))?;

    follow_queries::unfollow(&state.db, viewer_profile.id, target_profile.id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ── GET /profiles/:alias/followers ───────────────────────────────────────────

/// List profiles that follow the given alias, newest first.
#[instrument(skip(state), fields(alias = %alias))]
pub async fn get_followers(
    State(state): State<AppState>,
    Path(alias): Path<String>,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let profile = profile_queries::get_profile_by_alias(&state.db, &alias)
        .await?
        .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile not found"))?;

    let (after_ts, after_id) = decode_cursor_opt(page.cursor.as_deref())?;

    let rows = follow_queries::get_followers(
        &state.db,
        profile.id,
        after_ts,
        after_id,
        page.query_limit(),
    )
    .await?;

    let page_resp = build_follow_page(rows, page.effective_limit());
    let next = page_resp.next_cursor.clone();
    let has_more = page_resp.has_more;

    Ok(ApiResponse::paginated(page_resp, next, has_more))
}

// ── GET /profiles/:alias/following ───────────────────────────────────────────

/// List profiles that the given alias follows, newest first.
#[instrument(skip(state), fields(alias = %alias))]
pub async fn get_following(
    State(state): State<AppState>,
    Path(alias): Path<String>,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let profile = profile_queries::get_profile_by_alias(&state.db, &alias)
        .await?
        .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile not found"))?;

    let (after_ts, after_id) = decode_cursor_opt(page.cursor.as_deref())?;

    let rows = follow_queries::get_following(
        &state.db,
        profile.id,
        after_ts,
        after_id,
        page.query_limit(),
    )
    .await?;

    let page_resp = build_follow_page(rows, page.effective_limit());
    let next = page_resp.next_cursor.clone();
    let has_more = page_resp.has_more;

    Ok(ApiResponse::paginated(page_resp, next, has_more))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Decode an optional cursor string into `(timestamp, uuid)` components.
fn decode_cursor_opt(
    cursor: Option<&str>,
) -> Result<(Option<chrono::DateTime<chrono::Utc>>, Option<uuid::Uuid>), NyxError> {
    match cursor {
        None => Ok((None, None)),
        Some(s) => {
            let c = Cursor::decode(s)?;
            let (ts, id) = c.as_timestamp_id()?;
            Ok((Some(ts), Some(id)))
        }
    }
}
