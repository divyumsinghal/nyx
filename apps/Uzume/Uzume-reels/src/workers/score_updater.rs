//! Periodic score updater worker.
//!
//! Every 5 minutes, this worker queries recent reels and re-computes their
//! algorithmic ranking score using [`crate::services::reel_ranker::compute_score`].
//! The score is written back to PostgreSQL.
//!
//! This approach is "good enough" for the current scale. At higher scale, scores
//! can be computed in a streaming fashion via the NATS view/like events.

use std::time::Duration;

use tracing::{error, info, warn};

use crate::{
    queries::reels as reel_queries,
    services::reel_ranker::{compute_score, RankerConfig, ReelMetrics},
    state::AppState,
};

const SCORE_UPDATE_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

/// Run the periodic score updater loop.
pub async fn run(state: AppState) {
    let config = RankerConfig::default();
    let mut ticker = tokio::time::interval(SCORE_UPDATE_INTERVAL);

    loop {
        ticker.tick().await;

        // Fetch the top 500 ready reels ordered by creation time to re-score.
        match reel_queries::get_reel_feed(&state.db, None, None, 500).await {
            Ok(rows) => {
                let now = chrono::Utc::now();
                for row in rows {
                    let hours = (now - row.created_at).num_minutes() as f64 / 60.0;
                    let metrics = ReelMetrics {
                        likes: row.like_count,
                        views: row.view_count,
                        // Simplified: assume avg 50% completion rate when we don't
                        // have per-view aggregates handy. The full implementation
                        // would aggregate from reel_views.
                        avg_watch_duration_ms: f64::from(row.duration_ms) * 0.5,
                        reel_duration_ms: f64::from(row.duration_ms),
                        hours_since_posted: hours.max(0.0),
                    };

                    let new_score = compute_score(&metrics, &config);

                    if let Err(err) =
                        reel_queries::update_reel_score(&state.db, row.id, new_score).await
                    {
                        warn!(?err, reel_id = %row.id, "score_updater: failed to update score");
                    }
                }
                info!("score_updater: tick complete");
            }
            Err(err) => {
                error!(?err, "score_updater: failed to fetch reels for scoring");
            }
        }
    }
}
