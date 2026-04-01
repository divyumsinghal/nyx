//! NATS worker: `Uzume.profile.updated` → sync profile to Meilisearch.
//!
//! Each time a profile is updated, this worker receives the event and upserts
//! a search document in the `Uzume_profiles` Meilisearch index. This keeps
//! the search index eventually consistent with the database without requiring
//! a synchronous search write in the hot path.
//!
//! # Index document shape
//!
//! ```json
//! {
//!   "id":            "<profile UUID>",
//!   "alias":         "alice",
//!   "display_name":  "Alice Smith",
//!   "bio":           "...",
//!   "avatar_url":    "https://...",
//!   "is_private":    false,
//!   "is_verified":   true
//! }
//! ```

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::state::AppState;
use brizo::indexes;
use nyx_events::{subjects, Subscriber};

/// Search document indexed in Meilisearch for each profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSearchDocument {
    pub id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub is_private: bool,
    pub is_verified: bool,
}

/// Payload shape for the `Uzume.profile.updated` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdatedPayload {
    pub id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub is_private: bool,
    pub is_verified: bool,
}

/// Spawn the `Uzume.profile.updated` subscriber task.
///
/// Runs indefinitely in the background — caller should `tokio::spawn` this.
pub async fn run(state: AppState) {
    let subscriber = Subscriber::new(state.nats.clone());

    let mut stream = match subscriber
        .subscribe::<ProfileUpdatedPayload>(subjects::UZUME_PROFILE_UPDATED)
        .await
    {
        Ok(s) => s,
        Err(err) => {
            error!(?err, "search_sync worker: failed to subscribe");
            return;
        }
    };

    info!(
        "search_sync worker: listening on {}",
        subjects::UZUME_PROFILE_UPDATED
    );

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => sync_profile(&state, event.payload).await,
            Err(err) => {
                error!(?err, "search_sync worker: failed to receive event");
            }
        }
    }
}

#[instrument(skip(state), fields(profile_id = %payload.id, alias = %payload.alias))]
async fn sync_profile(state: &AppState, payload: ProfileUpdatedPayload) {
    // Private profiles are excluded from the public search index.
    if payload.is_private {
        let index = state.search.index(indexes::UZUME_PROFILES);
        let id_str = payload.id.to_string();
        if let Err(err) = index.delete_document(&id_str).await {
            warn!(
                ?err,
                "search_sync: failed to remove private profile from index"
            );
        } else {
            info!(profile_id = %payload.id, "search_sync: removed private profile from index");
        }
        return;
    }

    let doc = ProfileSearchDocument {
        id: payload.id,
        alias: payload.alias,
        display_name: payload.display_name,
        bio: payload.bio,
        avatar_url: payload.avatar_url,
        is_private: payload.is_private,
        is_verified: payload.is_verified,
    };

    let index = state.search.index(indexes::UZUME_PROFILES);
    match index.add_or_replace(&[doc], Some("id")).await {
        Ok(_) => {
            info!(profile_id = %payload.id, "search_sync: profile synced to index");
        }
        Err(err) => {
            error!(?err, "search_sync: failed to upsert profile document");
        }
    }
}
