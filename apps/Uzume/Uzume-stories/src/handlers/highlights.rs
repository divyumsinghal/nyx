use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use nun::NyxError;
use nyx_api::{ApiResponse, AuthUser, ValidatedJson};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    handlers::ApiError,
    models::highlight::{CreateHighlightRequest, HighlightInsert, HighlightResponse},
    queries::{highlights as highlight_queries, stories as story_queries},
    services::highlights::{
        build_highlight_page, ensure_highlight_owner, validate_highlight_title,
    },
    state::AppState,
};

#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn create_highlight(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<CreateHighlightRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_highlight_title(&body.title)?;

    let insert = HighlightInsert {
        id: Uuid::now_v7(),
        owner_identity_id: user.user_id,
        owner_alias: user.user_id.to_string(),
        title: body.title.trim().to_string(),
    };

    let row = highlight_queries::create_highlight(&state.db, &insert).await?;
    Ok((
        StatusCode::CREATED,
        ApiResponse::ok(HighlightResponse::from(row)),
    ))
}

#[instrument(skip(state), fields(alias = %alias))]
pub async fn list_highlights(
    State(state): State<AppState>,
    Path(alias): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let rows = highlight_queries::get_highlights_for_user(&state.db, &alias).await?;
    let page = build_highlight_page(rows);
    Ok(ApiResponse::ok(page))
}

#[instrument(skip(state), fields(highlight_id = %highlight_id, story_id = %story_id, user_id = %user.user_id))]
pub async fn add_story(
    State(state): State<AppState>,
    user: AuthUser,
    Path((highlight_id, story_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let highlight = highlight_queries::get_highlight_by_id(&state.db, highlight_id)
        .await?
        .ok_or_else(|| NyxError::not_found("highlight_not_found", "Highlight not found"))?;
    ensure_highlight_owner(&highlight, user.user_id)?;

    let story = story_queries::get_story_by_id(&state.db, story_id)
        .await?
        .ok_or_else(|| NyxError::not_found("story_not_found", "Story not found"))?;
    if story.author_identity_id != user.user_id {
        return Err(NyxError::forbidden(
            "story_not_owned",
            "Only your stories can be added to your highlights",
        )
        .into());
    }

    highlight_queries::add_story_to_highlight(&state.db, highlight_id, story_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(state), fields(highlight_id = %highlight_id, story_id = %story_id, user_id = %user.user_id))]
pub async fn remove_story(
    State(state): State<AppState>,
    user: AuthUser,
    Path((highlight_id, story_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let highlight = highlight_queries::get_highlight_by_id(&state.db, highlight_id)
        .await?
        .ok_or_else(|| NyxError::not_found("highlight_not_found", "Highlight not found"))?;
    ensure_highlight_owner(&highlight, user.user_id)?;

    highlight_queries::remove_story_from_highlight(&state.db, highlight_id, story_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(state), fields(highlight_id = %highlight_id, user_id = %user.user_id))]
pub async fn delete_highlight(
    State(state): State<AppState>,
    user: AuthUser,
    Path(highlight_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted =
        highlight_queries::delete_highlight(&state.db, highlight_id, user.user_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(NyxError::not_found("highlight_not_found", "Highlight not found").into())
    }
}
