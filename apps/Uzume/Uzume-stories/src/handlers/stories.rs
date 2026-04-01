use axum::{
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};
use nun::{Cursor, NyxError};
use nyx_api::{ApiResponse, AuthUser, CursorPagination, ValidatedJson};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    handlers::ApiError,
    models::story::{CreateStoryRequest, StoryInsert, StoryResponse},
    models::viewer::ViewerResponse,
    queries::{stories as story_queries, viewers as viewer_queries},
    services::stories::{
        build_story_page, build_viewer_page, ensure_story_owner, ensure_story_visible,
        media_type_from_content_type, should_record_view,
    },
    state::AppState,
};

/// Optional auth extractor for public endpoints that may personalize behavior.
pub struct MaybeAuthUser(pub Option<Uuid>);

impl<S: Send + Sync> FromRequestParts<S> for MaybeAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let maybe = AuthUser::from_request_parts(parts, state).await.ok();
        Ok(Self(maybe.map(|u| u.user_id)))
    }
}

#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn create_story(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<CreateStoryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let media_type = media_type_from_content_type(&body.content_type)?;

    let duration_secs = body
        .duration_secs
        .map(i32::try_from)
        .transpose()
        .map_err(|_| NyxError::bad_request("duration_invalid", "Duration is too large"))?;

    let insert = StoryInsert {
        id: Uuid::now_v7(),
        author_identity_id: user.user_id,
        author_alias: user.user_id.to_string(),
        media_type,
        duration_secs,
    };

    let row = story_queries::create_story(&state.db, &insert).await?;
    Ok((
        StatusCode::CREATED,
        ApiResponse::ok(StoryResponse::from(row)),
    ))
}

#[instrument(skip(state), fields(story_id = %story_id))]
pub async fn get_story(
    State(state): State<AppState>,
    Path(story_id): Path<Uuid>,
    MaybeAuthUser(viewer_identity_id): MaybeAuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let row = story_queries::get_story_by_id(&state.db, story_id)
        .await?
        .ok_or_else(|| NyxError::not_found("story_not_found", "Story not found"))?;

    ensure_story_visible(&row, viewer_identity_id)?;

    if let Some(viewer_id) = viewer_identity_id {
        if should_record_view(&row, viewer_id) {
            let viewer_alias = viewer_id.to_string();
            if let Err(err) =
                viewer_queries::record_view(&state.db, story_id, viewer_id, &viewer_alias).await
            {
                tracing::warn!(?err, %story_id, "failed to record story view");
            }
        }
    }

    Ok(ApiResponse::ok(StoryResponse::from(row)))
}

#[instrument(skip(state), fields(story_id = %story_id, user_id = %user.user_id))]
pub async fn delete_story(
    State(state): State<AppState>,
    user: AuthUser,
    Path(story_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = story_queries::delete_story(&state.db, story_id, user.user_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(NyxError::not_found("story_not_found", "Story not found").into())
    }
}

#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn get_feed(
    State(state): State<AppState>,
    user: AuthUser,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let (after_ts, after_id) = decode_cursor_opt(page.cursor.as_deref())?;
    let rows = story_queries::get_stories_feed(
        &state.db,
        user.user_id,
        after_ts,
        after_id,
        page.query_limit(),
    )
    .await?;

    let page_resp = build_story_page(rows, page.effective_limit());
    let next = page_resp.next_cursor.clone();
    let has_more = page_resp.has_more;

    Ok(ApiResponse::paginated(page_resp, next, has_more))
}

#[instrument(skip(state), fields(story_id = %story_id, user_id = %user.user_id))]
pub async fn mark_view(
    State(state): State<AppState>,
    user: AuthUser,
    Path(story_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let row = story_queries::get_story_by_id(&state.db, story_id)
        .await?
        .ok_or_else(|| NyxError::not_found("story_not_found", "Story not found"))?;

    ensure_story_visible(&row, Some(user.user_id))?;

    if should_record_view(&row, user.user_id) {
        let viewer_alias = user.user_id.to_string();
        viewer_queries::record_view(&state.db, story_id, user.user_id, &viewer_alias).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(state), fields(story_id = %story_id, user_id = %user.user_id))]
pub async fn get_viewers(
    State(state): State<AppState>,
    user: AuthUser,
    Path(story_id): Path<Uuid>,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let story = story_queries::get_story_by_id(&state.db, story_id)
        .await?
        .ok_or_else(|| NyxError::not_found("story_not_found", "Story not found"))?;

    ensure_story_owner(&story, user.user_id)?;

    let (after_ts, after_id) = decode_cursor_opt(page.cursor.as_deref())?;
    let rows =
        viewer_queries::get_viewers(&state.db, story_id, after_ts, after_id, page.query_limit())
            .await?;

    let page_resp = build_viewer_page(rows, page.effective_limit());
    let next = page_resp.next_cursor.clone();
    let has_more = page_resp.has_more;

    Ok(ApiResponse::paginated(page_resp, next, has_more))
}

fn decode_cursor_opt(
    cursor: Option<&str>,
) -> Result<(Option<chrono::DateTime<chrono::Utc>>, Option<Uuid>), NyxError> {
    match cursor {
        None => Ok((None, None)),
        Some(s) => {
            let c = Cursor::decode(s)?;
            let (ts, id) = c.as_timestamp_id()?;
            Ok((Some(ts), Some(id)))
        }
    }
}

#[allow(dead_code)]
fn _type_use(_: Vec<ViewerResponse>) {}
