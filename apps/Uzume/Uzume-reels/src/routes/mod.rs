//! Axum router for the uzume-reels service.

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use nyx_api::middleware::auth::auth;

use crate::{
    handlers::{
        audio::{create_audio, get_audio, list_trending_audio},
        reels::{
            create_reel, delete_reel, get_feed, get_reel, like_reel, record_view, unlike_reel,
        },
    },
    state::AppState,
};

/// Build and return the full router for the service.
pub fn router(state: AppState) -> Router {
    // Routes that require authentication.
    let authed = Router::new()
        .route("/reels", post(create_reel))
        .route("/reels/feed", get(get_feed))
        .route("/reels/:id", delete(delete_reel))
        .route("/reels/:id/like", post(like_reel))
        .route("/reels/:id/like", delete(unlike_reel))
        .route("/reels/:id/view", post(record_view))
        .route("/audio", post(create_audio))
        .route_layer(middleware::from_fn(auth));

    // Public (read-only) routes.
    let public = Router::new()
        .route("/reels/:id", get(get_reel))
        .route("/audio/trending", get(list_trending_audio))
        .route("/audio/:id", get(get_audio));

    Router::new().merge(authed).merge(public).with_state(state)
}
