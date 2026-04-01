use axum::{
    Router,
    middleware,
    routing::{delete, get, post},
};
use nyx_api::middleware::auth::auth;

use crate::{
    handlers::{
        highlights::{add_story, create_highlight, delete_highlight, list_highlights, remove_story},
        stories::{create_story, delete_story, get_feed, get_story, get_viewers, mark_view},
    },
    state::AppState,
};

/// Build and return the full router for the service.
pub fn router(state: AppState) -> Router {
    let authed = Router::new()
        .route("/stories", post(create_story))
        .route("/stories/feed", get(get_feed))
        .route("/stories/:id", delete(delete_story))
        .route("/stories/:id/view", post(mark_view))
        .route("/stories/:id/viewers", get(get_viewers))
        .route("/highlights", post(create_highlight))
        .route("/highlights/:id", delete(delete_highlight))
        .route("/highlights/:id/stories/:story_id", post(add_story))
        .route("/highlights/:id/stories/:story_id", delete(remove_story))
        .route_layer(middleware::from_fn(auth));

    let public = Router::new()
        .route("/stories/:id", get(get_story))
        .route("/profiles/:alias/highlights", get(list_highlights));

    Router::new().merge(authed).merge(public).with_state(state)
}
