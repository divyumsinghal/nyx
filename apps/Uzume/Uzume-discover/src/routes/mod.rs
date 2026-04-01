//! Axum router wiring for the discover service.

use axum::{
    middleware,
    routing::get,
    Router,
};
use nyx_api::middleware::auth::auth;

use crate::{
    handlers::{
        explore::{get_explore, get_suggested_users, get_trending},
        search::{search, search_by_hashtag},
    },
    state::AppState,
};

/// Build and return the full router for the service.
pub fn router(state: AppState) -> Router {
    let authed = Router::new()
        .route("/explore/suggested-users", get(get_suggested_users))
        .route_layer(middleware::from_fn(auth));

    let public = Router::new()
        .route("/explore", get(get_explore))
        .route("/explore/trending", get(get_trending))
        .route("/search", get(search))
        .route("/search/hashtag/{tag}", get(search_by_hashtag));

    Router::new().merge(authed).merge(public).with_state(state)
}
