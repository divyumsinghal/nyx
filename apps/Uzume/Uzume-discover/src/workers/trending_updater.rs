//! Trending hashtag updater worker.
//!
//! Runs on a 5-minute interval. Each tick:
//! 1. Queries `uzume.posts` for hashtag usage in the last 48 hours.
//! 2. Scores each hashtag with the time-decay formula.
//! 3. Upserts scores into `uzume.trending_hashtags`.

use std::time::Duration;

use tracing::{error, info};

use crate::{
    queries::trending::{compute_trending_hashtags_raw, upsert_trending_hashtag},
    services::trending::compute_hashtag_trending_score,
    state::AppState,
};

/// Interval between trending update ticks.
const TRENDING_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

/// Run the trending updater loop.
///
/// This function never returns under normal operation. It loops indefinitely,
/// waking on each tick to recompute and upsert trending hashtag scores.
///
/// # Panics
///
/// This function does not panic. Errors are logged and the worker continues.
pub async fn run(state: AppState) {
    let mut ticker = tokio::time::interval(TRENDING_INTERVAL);

    loop {
        ticker.tick().await;

        match tick(&state).await {
            Ok(upserted) => {
                info!(upserted, "trending_updater: upserted hashtag scores");
            }
            Err(err) => {
                error!(?err, "trending_updater: tick failed");
            }
        }
    }
}

/// Execute one trending update cycle.
///
/// Returns the number of hashtags that were upserted.
async fn tick(state: &AppState) -> Result<usize, nun::NyxError> {
    let raw_stats = compute_trending_hashtags_raw(&state.db).await?;

    let count = raw_stats.len();

    for (hashtag, usage_count, hours_since_first_use) in raw_stats {
        let score = compute_hashtag_trending_score(usage_count, hours_since_first_use);
        if let Err(err) =
            upsert_trending_hashtag(&state.db, &hashtag, usage_count, score).await
        {
            error!(?err, %hashtag, "trending_updater: failed to upsert hashtag");
        }
    }

    Ok(count)
}
