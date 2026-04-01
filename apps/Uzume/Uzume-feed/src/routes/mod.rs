//! Axum router for the `uzume-feed` service.
//!
//! Route table:
//!
//! | Method   | Path                            | Auth | Handler              |
//! |----------|---------------------------------|------|----------------------|
//! | GET      | /feed/posts/{id}                 | no   | `get_post`           |
//! | POST     | /feed/posts                     | yes  | `create_post`        |
//! | DELETE   | /feed/posts/{id}                 | yes  | `delete_post`        |
//! | GET      | /feed/timeline                  | yes  | `get_home_timeline`  |
//! | GET      | /feed/users/{alias}/posts        | no   | `get_user_timeline`  |
//! | POST     | /feed/posts/{id}/like            | yes  | `like_post`          |
//! | DELETE   | /feed/posts/{id}/like            | yes  | `unlike_post`        |
//! | POST     | /feed/posts/{id}/comments        | yes  | `create_comment`     |
//! | GET      | /feed/posts/{id}/comments        | no   | `get_comments`       |

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use nyx_api::middleware::auth::auth;

use crate::{
    handlers::{
        interactions::{create_comment, get_comments, like_post, unlike_post},
        posts::{create_post, delete_post, get_home_timeline, get_post, get_user_timeline},
    },
    state::AppState,
};

/// Build and return the full router for this service.
pub fn router(state: AppState) -> Router {
    // Authenticated routes — require a valid Bearer token.
    let authed = Router::new()
        .route("/feed/posts", post(create_post))
        .route("/feed/posts/{id}", delete(delete_post))
        .route("/feed/timeline", get(get_home_timeline))
        .route("/feed/posts/{id}/like", post(like_post))
        .route("/feed/posts/{id}/like", delete(unlike_post))
        .route("/feed/posts/{id}/comments", post(create_comment))
        .route_layer(middleware::from_fn(auth));

    // Public routes — no auth required.
    let public = Router::new()
        .route("/feed/posts/{id}", get(get_post))
        .route("/feed/users/{alias}/posts", get(get_user_timeline))
        .route("/feed/posts/{id}/comments", get(get_comments));

    Router::new().merge(authed).merge(public).with_state(state)
}
