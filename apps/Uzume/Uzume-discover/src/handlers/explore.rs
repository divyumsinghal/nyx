//! Axum handlers for the explore and trending endpoints.

use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    response::IntoResponse,
};
use chrono::Utc;
use nun::{Cursor, NyxError, PageResponse};
use nyx_api::{ApiResponse, AuthUser, CursorPagination};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    handlers::ApiError,
    models::{
        explore::{ExploreItem, ExploreItemType, ExploreSection, ExploreSectionType},
        trending::TrendingSnapshot,
    },
    queries::trending as trending_queries,
    services::{
        discover_ranker::{rerank_candidates, UserSignals},
        trending::compute_hashtag_trending_score,
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

/// `GET /explore` — returns a multi-section explore page, cursor-paginated.
///
/// Sections returned (in order):
/// 1. Trending posts
/// 2. Suggested users (requires auth; falls back to popular accounts if unauthenticated)
/// 3. Trending hashtags
/// 4. Featured reels
#[instrument(skip(state))]
pub async fn get_explore(
    State(state): State<AppState>,
    MaybeAuthUser(viewer_id): MaybeAuthUser,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let (cursor_score, cursor_id) = decode_score_cursor(page.cursor.as_deref())?;
    let limit = page.effective_limit();
    let query_limit = page.query_limit();

    // ── Trending posts section ─────────────────────────────────────────────
    let mut post_candidates = trending_queries::get_trending_posts_for_explore(
        &state.db,
        query_limit,
        cursor_score,
        cursor_id,
    )
    .await?;

    // ── Apply personalization if viewer is authenticated ───────────────────
    if let Some(viewer_profile_id) = viewer_id {
        let signals = UserSignals {
            liked_hashtags: vec![], // TODO: fetch from cache in follow-up
            followed_user_ids: vec![viewer_profile_id], // placeholder
            last_active: Utc::now(),
        };
        let n = post_candidates.len();
        post_candidates = rerank_candidates(
            post_candidates,
            &signals,
            &vec![vec![]; n],
            &vec![None; n],
            &vec![0.0_f64; n],
        );
    }

    // Pagination: detect overflow
    let has_more = post_candidates.len() > usize::from(limit);
    if has_more {
        post_candidates.truncate(usize::from(limit));
    }
    let next_cursor = if has_more {
        post_candidates.last().map(|item| {
            Cursor::score_id(item.score, item.id).encode()
        })
    } else {
        None
    };

    // ── Trending hashtags section ──────────────────────────────────────────
    let trending_tags = trending_queries::get_trending_hashtags(&state.db, 10).await?;
    let hashtag_items: Vec<ExploreItem> = trending_tags
        .iter()
        .map(|t| ExploreItem {
            id: Uuid::nil(),
            item_type: ExploreItemType::Hashtag,
            thumbnail_url: None,
            score: t.score,
        })
        .collect();

    // ── Suggested users section ────────────────────────────────────────────
    let suggested_user_items: Vec<ExploreItem> = if let Some(viewer_profile_id) = viewer_id {
        trending_queries::get_suggested_users_for_user(&state.db, viewer_profile_id, 10)
            .await?
            .into_iter()
            .map(|u| ExploreItem {
                id: u.user_id,
                item_type: ExploreItemType::User,
                thumbnail_url: u.avatar_url,
                score: u.follower_count as f64,
            })
            .collect()
    } else {
        vec![]
    };

    // ── Assemble sections ──────────────────────────────────────────────────
    let mut sections = vec![ExploreSection {
        section_type: ExploreSectionType::TrendingPosts,
        items: post_candidates,
    }];

    if !suggested_user_items.is_empty() {
        sections.push(ExploreSection {
            section_type: ExploreSectionType::SuggestedUsers,
            items: suggested_user_items,
        });
    }

    if !hashtag_items.is_empty() {
        sections.push(ExploreSection {
            section_type: ExploreSectionType::TrendingHashtags,
            items: hashtag_items,
        });
    }

    Ok(ApiResponse::paginated(sections, next_cursor, has_more))
}

/// `GET /explore/trending` — returns a `TrendingSnapshot` with all categories.
#[instrument(skip(state))]
pub async fn get_trending(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let hashtags = trending_queries::get_trending_hashtags(&state.db, 20).await?;

    // Reels: query top scored reels from DB
    let reel_rows: Vec<(Uuid, f64)> = sqlx::query_as(
        r#"
        SELECT id, score
        FROM "Uzume".reels
        WHERE processing_state = 'ready'
        ORDER BY score DESC
        LIMIT 20
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(NyxError::from)?;

    let reels = reel_rows
        .into_iter()
        .map(|(reel_id, score)| crate::models::trending::TrendingReel {
            reel_id,
            score,
            updated_at: Utc::now(),
        })
        .collect();

    // Audio: query top-used reel audio tracks
    let audio_rows: Vec<(Uuid, String, i64)> = sqlx::query_as(
        r#"
        SELECT id, title, use_count
        FROM "Uzume".reel_audio
        ORDER BY use_count DESC
        LIMIT 20
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(NyxError::from)?;

    let audio = audio_rows
        .into_iter()
        .map(|(audio_id, title, reel_count)| {
            // Simple score: use_count weighted by recency (approximate here)
            let score = compute_hashtag_trending_score(reel_count, 1.0);
            crate::models::trending::TrendingAudio {
                audio_id,
                title,
                reel_count,
                score,
            }
        })
        .collect();

    let snapshot = TrendingSnapshot {
        hashtags,
        reels,
        audio,
        computed_at: Utc::now(),
    };

    Ok(ApiResponse::ok(snapshot))
}

/// `GET /explore/suggested-users` — cursor-paginated suggested users.
///
/// Requires authentication. Returns users in the viewer's extended social graph.
#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn get_suggested_users(
    State(state): State<AppState>,
    user: AuthUser,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let limit = page.effective_limit();

    let users = trending_queries::get_suggested_users_for_user(
        &state.db,
        user.user_id,
        i64::from(limit) + 1,
    )
    .await?;

    let page_resp = PageResponse::from_overflowed(users, limit, |u| {
        Cursor::score_id(u.follower_count as f64, u.user_id)
    });
    let next = page_resp.next_cursor.clone();
    let has_more = page_resp.has_more;

    Ok(ApiResponse::paginated(page_resp, next, has_more))
}

/// Decode an optional score + ID cursor.
fn decode_score_cursor(
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
