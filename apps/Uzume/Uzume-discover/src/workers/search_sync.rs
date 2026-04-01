//! NATS subscriber that syncs content to Meilisearch indexes.
//!
//! Listens to:
//! - `Uzume.post.created` — indexes the new post document in `Uzume_posts`
//! - `Uzume.profile.updated` — indexes the updated profile in `Uzume_profiles`

use brizo::indexes;
use nyx_events::subjects;
use serde::Deserialize;
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::{
    models::search_result::{PostDocument, ProfileDocument},
    state::AppState,
};

/// Payload for the `Uzume.post.created` NATS event.
#[derive(Debug, Deserialize)]
struct PostCreatedPayload {
    pub post_id: String,
    pub author_id: String,
    pub author_alias: String,
    pub caption: String,
    pub hashtags: Vec<String>,
}

/// Payload for the `Uzume.profile.updated` NATS event.
#[derive(Debug, Deserialize)]
struct ProfileUpdatedPayload {
    pub profile_id: String,
    pub alias: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub follower_count: i64,
}

/// Run the search sync worker.
///
/// Subscribes to NATS subjects and forwards relevant events to Meilisearch.
/// This function never returns under normal operation.
///
/// # Panics
///
/// Does not panic. Individual message errors are logged and processing
/// continues.
pub async fn run(state: AppState) {
    tokio::join!(
        run_post_sync(state.clone()),
        run_profile_sync(state),
    );
}

/// Subscribe to `Uzume.post.created` and index posts in Meilisearch.
async fn run_post_sync(state: AppState) {
    let subject = subjects::UZUME_POST_CREATED;
    info!(%subject, "search_sync: subscribing to post.created events");

    let mut subscriber = match state.nats.subscribe(subject.to_string()).await {
        Ok(s) => s,
        Err(err) => {
            error!(?err, %subject, "search_sync: failed to subscribe to post.created");
            return;
        }
    };

    use futures::StreamExt as _;
    while let Some(msg) = subscriber.next().await {
        handle_post_created(&state, &msg.payload).await;
    }
}

/// Subscribe to `Uzume.profile.updated` and index profiles in Meilisearch.
async fn run_profile_sync(state: AppState) {
    let subject = subjects::UZUME_PROFILE_UPDATED;
    info!(%subject, "search_sync: subscribing to profile.updated events");

    let mut subscriber = match state.nats.subscribe(subject.to_string()).await {
        Ok(s) => s,
        Err(err) => {
            error!(?err, %subject, "search_sync: failed to subscribe to profile.updated");
            return;
        }
    };

    use futures::StreamExt as _;
    while let Some(msg) = subscriber.next().await {
        handle_profile_updated(&state, &msg.payload).await;
    }
}

#[instrument(skip(state, payload))]
async fn handle_post_created(state: &AppState, payload: &[u8]) {
    let event: PostCreatedPayload = match serde_json::from_slice(payload) {
        Ok(e) => e,
        Err(err) => {
            error!(?err, "search_sync: failed to parse post.created payload");
            return;
        }
    };

    // Fetch thumbnail from DB — use None as fallback if not ready yet
    let thumbnail_url = fetch_post_thumbnail(state, &event.post_id).await;

    let doc = PostDocument {
        id: event.post_id.clone(),
        author_id: event.author_id,
        author_alias: event.author_alias,
        caption: event.caption,
        hashtags: event.hashtags,
        thumbnail_url,
        like_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    if let Err(err) = state.search.add_documents(indexes::UZUME_POSTS, &[doc]).await {
        error!(?err, post_id = %event.post_id, "search_sync: failed to index post");
    } else {
        info!(post_id = %event.post_id, "search_sync: indexed post");
    }
}

#[instrument(skip(state, payload))]
async fn handle_profile_updated(state: &AppState, payload: &[u8]) {
    let event: ProfileUpdatedPayload = match serde_json::from_slice(payload) {
        Ok(e) => e,
        Err(err) => {
            error!(?err, "search_sync: failed to parse profile.updated payload");
            return;
        }
    };

    let doc = ProfileDocument {
        id: event.profile_id.clone(),
        alias: event.alias,
        display_name: event.display_name,
        avatar_url: event.avatar_url,
        follower_count: event.follower_count,
    };

    if let Err(err) = state
        .search
        .add_documents(indexes::UZUME_PROFILES, &[doc])
        .await
    {
        error!(?err, profile_id = %event.profile_id, "search_sync: failed to index profile");
    } else {
        info!(profile_id = %event.profile_id, "search_sync: indexed profile");
    }
}

/// Attempt to fetch the thumbnail URL for a post from the DB.
///
/// Returns `None` if the post media has not been processed yet or if the query
/// fails (non-fatal — the document can be re-indexed when media is ready).
async fn fetch_post_thumbnail(state: &AppState, post_id: &str) -> Option<String> {
    let uuid = Uuid::parse_str(post_id).ok()?;
    sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT thumbnail_key
        FROM "Uzume".post_media
        WHERE post_id = $1 AND display_order = 0
        LIMIT 1
        "#,
    )
    .bind(uuid)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .flatten()
}
