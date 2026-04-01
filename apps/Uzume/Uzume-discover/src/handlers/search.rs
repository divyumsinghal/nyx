//! Axum handlers for the search endpoints.

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use brizo::{indexes, SearchRequest};
use nun::{Cursor, NyxError, PageResponse};
use nyx_api::{ApiResponse, CursorPagination};
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    handlers::ApiError,
    models::{
        explore::{ExploreItem, ExploreItemType},
        search_result::{
            PostDocument, PostSearchResult, ProfileDocument, SearchResults, SearchType,
            UserSearchResult,
        },
    },
    state::AppState,
};

/// Query parameters for `GET /search`.
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// The search query string.
    pub q: String,

    /// Filter to a specific result type.
    #[serde(default)]
    pub r#type: SearchType,

    /// Page size, defaults to 20.
    #[serde(default = "default_search_limit")]
    pub limit: usize,

    /// Offset for pagination.
    #[serde(default)]
    pub offset: usize,
}

fn default_search_limit() -> usize {
    20
}

/// `GET /search?q=...&type=all|users|posts|hashtags`
///
/// Unified search via Meilisearch. Returns users, posts, and/or hashtags
/// depending on the `type` parameter.
///
/// Returns `400 Bad Request` if the query string is empty.
#[instrument(skip(state), fields(q = %params.q))]
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    if params.q.trim().is_empty() {
        return Err(NyxError::bad_request(
            "query_required",
            "Search query must not be empty",
        )
        .into());
    }

    let query = params.q.trim().to_string();
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset;
    let search_type = params.r#type;

    // ── Users ──────────────────────────────────────────────────────────────
    let users = if matches!(search_type, SearchType::All | SearchType::Users) {
        let req = SearchRequest::new(&query).with_limit(limit).with_offset(offset);
        let resp = state
            .search
            .search::<ProfileDocument>(indexes::UZUME_PROFILES, req)
            .await?;
        resp.hits
            .into_iter()
            .map(|hit| UserSearchResult {
                user_id: Uuid::parse_str(&hit.document.id)
                    .unwrap_or(Uuid::nil()),
                alias: hit.document.alias,
                display_name: hit.document.display_name,
                avatar_url: hit.document.avatar_url,
                follower_count: hit.document.follower_count,
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    // ── Posts ──────────────────────────────────────────────────────────────
    let posts = if matches!(search_type, SearchType::All | SearchType::Posts) {
        let req = SearchRequest::new(&query).with_limit(limit).with_offset(offset);
        let resp = state
            .search
            .search::<PostDocument>(indexes::UZUME_POSTS, req)
            .await?;
        resp.hits
            .into_iter()
            .map(|hit| {
                let doc = hit.document;
                PostSearchResult {
                    post_id: Uuid::parse_str(&doc.id).unwrap_or(Uuid::nil()),
                    author_id: Uuid::parse_str(&doc.author_id).unwrap_or(Uuid::nil()),
                    author_alias: doc.author_alias,
                    caption: if doc.caption.is_empty() { None } else { Some(doc.caption) },
                    thumbnail_url: doc.thumbnail_url,
                    like_count: doc.like_count,
                    created_at: doc.created_at.parse().unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    // ── Hashtags ───────────────────────────────────────────────────────────
    let hashtags = if matches!(search_type, SearchType::All | SearchType::Hashtags) {
        // Search trending hashtag names that contain the query string
        let q_lower = query.to_lowercase();
        let all_tags: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT hashtag, score
            FROM "Uzume".trending_hashtags
            WHERE hashtag ILIKE $1
            ORDER BY score DESC
            LIMIT 10
            "#,
        )
        .bind(format!("%{q_lower}%"))
        .fetch_all(&state.db)
        .await
        .map_err(NyxError::from)?;
        all_tags.into_iter().map(|(h, _)| h).collect()
    } else {
        vec![]
    };

    Ok(ApiResponse::ok(SearchResults {
        users,
        posts,
        hashtags,
        query,
    }))
}

/// `GET /search/hashtag/:tag` — posts for a specific hashtag, cursor-paginated.
///
/// Returns posts containing the given hashtag, ordered by engagement score
/// descending.
#[instrument(skip(state), fields(tag = %tag))]
pub async fn search_by_hashtag(
    State(state): State<AppState>,
    Path(tag): Path<String>,
    CursorPagination(page): CursorPagination,
) -> Result<impl IntoResponse, ApiError> {
    let limit = page.effective_limit();
    let (cursor_score, cursor_id) = decode_score_cursor(page.cursor.as_deref())?;

    let rows: Vec<(Uuid, Option<String>, f64)> = match (cursor_score, cursor_id) {
        (Some(score), Some(cid)) => {
            sqlx::query_as(
                r#"
                SELECT
                    p.id,
                    pm.thumbnail_key,
                    (p.like_count * 3.0 + p.comment_count * 5.0 + p.save_count * 2.0) AS score
                FROM "Uzume".posts p
                LEFT JOIN "Uzume".post_media pm
                    ON pm.post_id = p.id AND pm.display_order = 0
                WHERE $1 = ANY(p.hashtags)
                  AND ((p.like_count * 3.0 + p.comment_count * 5.0 + p.save_count * 2.0) < $2
                   OR ((p.like_count * 3.0 + p.comment_count * 5.0 + p.save_count * 2.0) = $2
                       AND p.id < $3))
                ORDER BY score DESC, p.id DESC
                LIMIT $4
                "#,
            )
            .bind(&tag)
            .bind(score)
            .bind(cid)
            .bind(i64::from(limit) + 1)
            .fetch_all(&state.db)
            .await
            .map_err(NyxError::from)?
        }
        _ => {
            sqlx::query_as(
                r#"
                SELECT
                    p.id,
                    pm.thumbnail_key,
                    (p.like_count * 3.0 + p.comment_count * 5.0 + p.save_count * 2.0) AS score
                FROM "Uzume".posts p
                LEFT JOIN "Uzume".post_media pm
                    ON pm.post_id = p.id AND pm.display_order = 0
                WHERE $1 = ANY(p.hashtags)
                ORDER BY score DESC, p.id DESC
                LIMIT $2
                "#,
            )
            .bind(&tag)
            .bind(i64::from(limit) + 1)
            .fetch_all(&state.db)
            .await
            .map_err(NyxError::from)?
        }
    };

    let items: Vec<ExploreItem> = rows
        .into_iter()
        .map(|(id, thumbnail_url, score)| ExploreItem {
            id,
            item_type: ExploreItemType::Post,
            thumbnail_url,
            score,
        })
        .collect();

    let page_resp = PageResponse::from_overflowed(items, limit, |item| {
        Cursor::score_id(item.score, item.id)
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
