//! NATS subscriber that syncs newly created reels to Meilisearch.
//!
//! Listens on `Uzume.reel.created` and upserts a search document for every
//! reel whose `processing_state` transitions to `ready`.
//!
//! This worker is intentionally idempotent: re-processing the same event
//! simply re-indexes the document, which is safe.

use std::time::Duration;

use brizo::{IndexSync, SearchClient};
use serde::Deserialize;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{models::reel::ReelSearchDoc, queries::reels as reel_queries, state::AppState};

/// Meilisearch index name for reels.
pub const UZUME_REELS_INDEX: &str = "Uzume_reels";

/// Minimal payload we care about from the `Uzume.reel.created` event.
#[derive(Debug, Deserialize)]
struct ReelCreatedPayload {
    reel_id: Uuid,
}

/// Subscribe to `Uzume.reel.created` and index each reel in Meilisearch.
///
/// Retries on subscription failure with exponential back-off (capped at 60 s).
pub async fn run(state: AppState) {
    let sync = IndexSync::new(state.search.clone());
    let mut backoff = Duration::from_secs(1);

    loop {
        match run_inner(&state, &sync).await {
            Ok(()) => {
                info!("search_sync worker exited cleanly");
                break;
            }
            Err(err) => {
                error!(?err, "search_sync worker crashed, retrying in {backoff:?}");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(60));
            }
        }
    }
}

async fn run_inner(state: &AppState, sync: &IndexSync) -> Result<(), anyhow::Error> {
    let mut sub = state
        .nats
        .subscribe(nyx_events::subjects::UZUME_REEL_CREATED.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("NATS subscribe failed: {e}"))?;

    info!("search_sync worker subscribed to {}", nyx_events::subjects::UZUME_REEL_CREATED);

    while let Some(msg) = futures::StreamExt::next(&mut sub).await {
        let payload: ReelCreatedPayload = match serde_json::from_slice(&msg.payload) {
            Ok(p) => p,
            Err(err) => {
                warn!(?err, "search_sync: failed to parse reel.created payload, skipping");
                continue;
            }
        };

        match reel_queries::get_reel_by_id(&state.db, payload.reel_id).await {
            Ok(Some(row)) => {
                let doc = ReelSearchDoc::from(&row);
                if let Err(err) = sync.add_document(UZUME_REELS_INDEX, &doc).await {
                    warn!(?err, reel_id = %payload.reel_id, "search_sync: failed to index reel");
                } else {
                    info!(reel_id = %payload.reel_id, "search_sync: indexed reel");
                }
            }
            Ok(None) => {
                warn!(reel_id = %payload.reel_id, "search_sync: reel not found in DB");
            }
            Err(err) => {
                warn!(?err, reel_id = %payload.reel_id, "search_sync: DB error fetching reel");
            }
        }
    }

    Ok(())
}

/// Remove a reel from the Meilisearch index.
///
/// Called when a reel is soft-deleted. Fire-and-forget; errors are logged but
/// not propagated.
pub async fn remove_from_index(search: &SearchClient, reel_id: Uuid) {
    let sync = IndexSync::new(search.clone());
    if let Err(err) = sync
        .delete_document(UZUME_REELS_INDEX, &reel_id.to_string())
        .await
    {
        warn!(?err, %reel_id, "search_sync: failed to remove reel from index");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reel_created_payload_deserializes() {
        let id = Uuid::now_v7();
        let json = format!(r#"{{"reel_id":"{id}"}}"#);
        let payload: ReelCreatedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload.reel_id, id);
    }

    #[test]
    fn uzume_reels_index_name_matches_convention() {
        // Convention: {app}_{entity}
        assert_eq!(UZUME_REELS_INDEX, "Uzume_reels");
    }
}
