//! Axum handlers for post interaction endpoints (likes and comments).
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
};
use nyx_api::{ApiResponse, AuthUser, CursorPagination, ValidatedJson};
use nyx_events::{subjects, Publisher};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    models::comment::CommentInsert,
    queries::{comments as comment_queries, likes as like_queries},
    services::interactions::{build_comment_page, CommentResponse, CreateCommentRequest, LikeResponse},
    state::AppState,
};
use nun::{Cursor, NyxError};

use super::ApiError;

// ── POST /feed/posts/:id/like ─────────────────────────────────────────────────

/// Like a post. Idempotent — liking twice has no extra effect.
#[instrument(skip(state), fields(post_id = %post_id, user_id = %user.user_id))]
pub async fn like_post(
    State(state): State<AppState>,
    user: AuthUser,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let liker_alias = user.user_id.to_string();

    like_queries::like_post(&state.db, post_id, user.user_id, &liker_alias).await?;

    let like_count = like_queries::get_like_count(&state.db, post_id).await?;

    let publisher = Publisher::new(state.nats.clone(), "Uzume");
    let payload = serde_json::json!({
        "post_id": post_id,
        "liker_alias": liker_alias,
    });
    if let Err(err) = publisher.publish(subjects::UZUME_POST_LIKED, payload).await {
        tracing::warn!(?err, "failed to publish post.liked event");
    }

    Ok(ApiResponse::ok(LikeResponse {
        post_id,
        liked: true,
        like_count,
    }))
}

// ── DELETE /feed/posts/:id/like ───────────────────────────────────────────────

/// Unlike a post. Idempotent — unliking a non-liked post has no effect.
#[instrument(skip(state), fields(post_id = %post_id, user_id = %user.user_id))]
pub async fn unlike_post(
    State(state): State<AppState>,
    user: AuthUser,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    like_queries::unlike_post(&state.db, post_id, user.user_id).await?;

    let like_count = like_queries::get_like_count(&state.db, post_id).await?;

    Ok(ApiResponse::ok(LikeResponse {
        post_id,
        liked: false,
        like_count,
    }))
}

// ── POST /feed/posts/:id/comments ────────────────────────────────────────────

/// Add a comment to a post.
#[instrument(skip(state), fields(post_id = %post_id, user_id = %user.user_id))]
pub async fn create_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(post_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateCommentRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let author_alias = user.user_id.to_string();

    let insert = CommentInsert {
        id: Uuid::now_v7(),
        post_id,
        author_identity_id: user.user_id,
        author_alias: author_alias.clone(),
        content: body.content.clone(),
    };

    let row = comment_queries::create_comment(&state.db, &insert).await?;

    // Increment comment_count on the post.
    sqlx::query("UPDATE uzume.posts SET comment_count = comment_count + 1 WHERE id = $1")
        .bind(post_id)
        .execute(&state.db)
        .await
        .map_err(NyxError::from)?;

    let publisher = Publisher::new(state.nats.clone(), "Uzume");
    let payload = serde_json::json!({
        "post_id": post_id,
        "comment_id": row.id,
        "author_alias": author_alias,
    });
    if let Err(err) = publisher
        .publish(subjects::UZUME_COMMENT_CREATED, payload)
        .await
    {
        tracing::warn!(?err, "failed to publish comment.created event");
    }

    Ok((StatusCode::CREATED, ApiResponse::ok(CommentResponse::from(row))))
}

// ── GET /feed/posts/:id/comments ─────────────────────────────────────────────

/// List comments on a post, oldest-first.
#[instrument(skip(state), fields(post_id = %post_id))]
pub async fn get_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let (after_ts, after_id) = decode_cursor_opt(page.cursor.as_deref())?;

    let rows = comment_queries::get_comments(
        &state.db,
        post_id,
        after_ts,
        after_id,
        page.query_limit(),
    )
    .await?;

    let page_resp = build_comment_page(rows, page.effective_limit());
    let next = page_resp.next_cursor.clone();
    let has_more = page_resp.has_more;

    Ok(ApiResponse::paginated(page_resp, next, has_more))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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
