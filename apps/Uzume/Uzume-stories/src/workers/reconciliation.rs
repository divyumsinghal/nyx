use std::time::Duration;

use tracing::{error, info};

use crate::{queries::stories as story_queries, state::AppState};

const RECONCILE_INTERVAL: Duration = Duration::from_secs(600);

/// Periodically reconciles stale data by deleting long-expired stories.
///
/// This loop is retry-safe and idempotent: each tick can be repeated safely.
pub async fn run(state: AppState) {
    let mut ticker = tokio::time::interval(RECONCILE_INTERVAL);

    loop {
        ticker.tick().await;

        match story_queries::delete_expired_stories(&state.db).await {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    info!(deleted_count, "reconciled long-expired stories");
                }
            }
            Err(err) => {
                error!(?err, "reconciliation worker tick failed");
            }
        }
    }
}
