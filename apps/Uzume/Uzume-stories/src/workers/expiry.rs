use std::time::Duration;

use tracing::{error, info};

use crate::{queries::stories as story_queries, state::AppState};

const EXPIRY_INTERVAL: Duration = Duration::from_secs(60);

/// Periodically expires old stories.
///
/// This loop is retry-safe and idempotent because `expire_old_stories`
/// updates only currently active rows whose TTL has elapsed.
pub async fn run(state: AppState) {
    let mut ticker = tokio::time::interval(EXPIRY_INTERVAL);

    loop {
        ticker.tick().await;

        match story_queries::expire_old_stories(&state.db).await {
            Ok(expired_ids) => {
                if !expired_ids.is_empty() {
                    info!(count = expired_ids.len(), "expired stories");
                }
            }
            Err(err) => {
                error!(?err, "expiry worker tick failed");
            }
        }
    }
}
