//! Axum router for the `uzume-profiles` service.
//!
//! Route table:
//!
//! | Method   | Path                          | Auth | Handler              |
//! |----------|-------------------------------|------|----------------------|
//! | GET      | /profiles/me                  | yes  | `get_my_profile`     |
//! | PATCH    | /profiles/me                  | yes  | `patch_my_profile`   |
//! | GET      | /profiles/:alias              | opt  | `get_profile`        |
//! | POST     | /profiles/:alias/follow       | yes  | `follow_user`        |
//! | DELETE   | /profiles/:alias/follow       | yes  | `unfollow_user`      |
//! | GET      | /profiles/:alias/followers    | no   | `get_followers`      |
//! | GET      | /profiles/:alias/following    | no   | `get_following`      |
//!
//! The `/profiles/me` routes must be registered **before** `/profiles/:alias`
//! so that Axum matches the literal `me` segment first.

use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};

use crate::{
    handlers::{
        follow::{follow_user, get_followers, get_following, unfollow_user},
        profile::{get_my_profile, get_profile, patch_my_profile},
    },
    state::AppState,
};
use nyx_api::middleware::auth::auth;

/// Build and return the full router for this service.
///
/// The caller (main.rs) wraps this in `NyxServer::builder().with_routes(...)`.
pub fn router(state: AppState) -> Router {
    // Authenticated routes — require a valid Bearer token.
    let authed = Router::new()
        .route("/profiles/me", get(get_my_profile))
        .route("/profiles/me", patch(patch_my_profile))
        .route("/profiles/:alias/follow", post(follow_user))
        .route("/profiles/:alias/follow", delete(unfollow_user))
        .route_layer(middleware::from_fn(auth));

    // Public routes — no auth required (but auth is optional for get_profile).
    let public = Router::new()
        .route("/profiles/:alias", get(get_profile))
        .route("/profiles/:alias/followers", get(get_followers))
        .route("/profiles/:alias/following", get(get_following));

    Router::new().merge(authed).merge(public).with_state(state)
}
