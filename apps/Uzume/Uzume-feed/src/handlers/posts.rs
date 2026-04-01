//! Axum handlers for post CRUD and timeline endpoints.
//!
//! Each handler follows the pattern:
//! 1. Extract validated inputs from the request.
//! 2. Call queries to load / persist data.
//! 3. Apply domain logic from the services layer.
//! 4. Publish any domain events.
//! 5. Return an `ApiResponse<T>`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use nyx_api::{ApiResponse, AuthUser, CursorPagination, ValidatedJson};
use nyx_events::{subjects, Publisher};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    models::post::PostInsert,
    queries::posts as post_queries,
    services::feed::{assert_is_author, build_post_page, CreatePostRequest, PostResponse},
    state::AppState,
};
use nun::{Cursor, NyxError};

// ── GET /feed/posts/:id ───────────────────────────────────────────────────────

/// Return a single post by UUID.
#[instrument(skip(state), fields(post_id = %post_id))]
pub async fn get_post(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let row = post_queries::get_post_by_id(&state.db, post_id)
        .await?
        .ok_or_else(|| NyxError::not_found("post_not_found", "Post not found"))?;

    Ok(ApiResponse::ok(PostResponse::from(row)))
}

// ── POST /feed/posts ──────────────────────────────────────────────────────────

/// Create a new post for the authenticated user.
#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn create_post(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<CreatePostRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // We need the author's alias. For now we use a placeholder lookup via the
    // identity_id stored in the post. In production, Heka resolves the alias.
    // The alias is passed as a header or resolved from the JWT claims — here we
    // derive it from the request extension if available, otherwise fall back to
    // the identity UUID string representation.
    //
    // NOTE: Real production flow: Heimdall resolves alias and injects it as
    // `X-Nyx-Alias` header. For the stub implementation we use identity_id.
    let author_alias = user.user_id.to_string();

    let insert = PostInsert {
        id: Uuid::now_v7(),
        identity_id: user.user_id,
        author_alias: author_alias.clone(),
        caption: body.caption.clone(),
    };

    let row = post_queries::create_post(&state.db, &insert).await?;

    // Publish Uzume.post.created for downstream consumers (Oya, Brizo, etc.).
    let publisher = Publisher::new(state.nats.clone(), "Uzume");
    let payload = serde_json::json!({
        "post_id": row.id,
        "author_alias": row.author_alias,
        "caption": row.caption,
        "created_at": row.created_at,
    });
    if let Err(err) = publisher.publish(subjects::UZUME_POST_CREATED, payload).await {
        tracing::warn!(?err, "failed to publish post.created event");
    }

    Ok((StatusCode::CREATED, ApiResponse::ok(PostResponse::from(row))))
}

// ── DELETE /feed/posts/:id ────────────────────────────────────────────────────

/// Delete a post. Only the post author may delete.
#[instrument(skip(state), fields(post_id = %post_id, user_id = %user.user_id))]
pub async fn delete_post(
    State(state): State<AppState>,
    user: AuthUser,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let row = post_queries::get_post_by_id(&state.db, post_id)
        .await?
        .ok_or_else(|| NyxError::not_found("post_not_found", "Post not found"))?;

    assert_is_author(&row, user.user_id)?;

    post_queries::delete_post(&state.db, post_id).await?;

    // Publish Uzume.post.deleted for downstream consumers.
    let publisher = Publisher::new(state.nats.clone(), "Uzume");
    let payload = serde_json::json!({
        "post_id": post_id,
        "author_alias": row.author_alias,
    });
    if let Err(err) = publisher.publish(subjects::UZUME_POST_DELETED, payload).await {
        tracing::warn!(?err, "failed to publish post.deleted event");
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── GET /feed/timeline ────────────────────────────────────────────────────────

/// Return the global chronological home timeline for the authenticated user.
#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn get_home_timeline(
    State(state): State<AppState>,
    user: AuthUser,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let (after_ts, after_id) = decode_cursor_opt(page.cursor.as_deref())?;

    let rows = post_queries::get_home_timeline(
        &state.db,
        after_ts,
        after_id,
        page.query_limit(),
    )
    .await?;

    let page_resp = build_post_page(rows, page.effective_limit());
    let next = page_resp.next_cursor.clone();
    let has_more = page_resp.has_more;

    Ok(ApiResponse::paginated(page_resp, next, has_more))
}

// ── GET /feed/users/:alias/posts ──────────────────────────────────────────────

/// Return posts by a specific user alias (their profile timeline).
#[instrument(skip(state), fields(alias = %alias))]
pub async fn get_user_timeline(
    State(state): State<AppState>,
    Path(alias): Path<String>,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let (after_ts, after_id) = decode_cursor_opt(page.cursor.as_deref())?;

    let rows = post_queries::get_user_timeline(
        &state.db,
        &alias,
        after_ts,
        after_id,
        page.query_limit(),
    )
    .await?;

    let page_resp = build_post_page(rows, page.effective_limit());
    let next = page_resp.next_cursor.clone();
    let has_more = page_resp.has_more;

    Ok(ApiResponse::paginated(page_resp, next, has_more))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Decode an optional cursor string into `(timestamp, uuid)` components.
fn decode_cursor_opt(
    cursor: Option<&str>,
) -> Result<
    (
        Option<chrono::DateTime<chrono::Utc>>,
        Option<Uuid>,
    ),
    NyxError,
> {
    match cursor {
        None => Ok((None, None)),
        Some(s) => {
            let c = Cursor::decode(s)?;
            let (ts, id) = c.as_timestamp_id()?;
            Ok((Some(ts), Some(id)))
        }
    }
}

// ── Error conversion ──────────────────────────────────────────────────────────

/// Wrapper that converts `NyxError` into an Axum HTTP response.
pub struct ApiError(NyxError);

impl From<NyxError> for ApiError {
    fn from(err: NyxError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status =
            StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = Json(self.0.to_error_response(None));
        (status, body).into_response()
    }
}
