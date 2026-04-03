//! Health check handler for Heimdall.
//!
//! [`health_handler`] probes all configured upstream services concurrently
//! and returns a JSON summary. The HTTP status is always `200 OK` — whether
//! `status` is `"ok"` or `"degraded"` is determined by the JSON body so that
//! orchestrators can always parse the response.
//!
//! # Response shape
//!
//! ```json
//! {
//!   "status": "ok",
//!   "upstreams": {
//!     "kratos": { "reachable": true, "latency_ms": 3 },
//!     "matrix": { "reachable": false, "latency_ms": null }
//!   }
//! }
//! ```

use std::collections::HashMap;
use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use futures::future::join_all;
use serde::Serialize;
use tokio::time::{timeout, Duration};
use tracing::{debug, warn};

use crate::state::AppState;

// ── Response types ────────────────────────────────────────────────────────────

/// Overall health response returned by `GET /healthz`.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// `"ok"` if all upstreams are reachable, `"degraded"` otherwise.
    pub status: &'static str,
    /// Per-upstream probe results keyed by service name.
    pub upstreams: HashMap<String, UpstreamStatus>,
    /// Database connectivity status for PostgreSQL.
    pub database: UpstreamStatus,
}

/// Result of probing a single upstream.
#[derive(Debug, Serialize)]
pub struct UpstreamStatus {
    /// Whether the upstream responded within the probe timeout.
    pub reachable: bool,
    /// Round-trip latency in milliseconds, or `null` if unreachable.
    pub latency_ms: Option<u64>,
}

// ── health_handler ────────────────────────────────────────────────────────────

/// Axum handler for `GET /healthz`.
///
/// Probes all upstream services concurrently with a 5-second timeout per
/// probe. Returns `200 OK` regardless of upstream health — use the `status`
/// field in the JSON body to determine overall health.
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let cfg = &state.config;

    // Build the list of (name, url) pairs to probe.
    let upstreams: Vec<(&str, String)> = vec![
        ("kratos", cfg.kratos_public_url.clone()),
        ("matrix", cfg.matrix_url.clone()),
        ("uzume-profiles", cfg.uzume_profiles_url.clone()),
        ("uzume-feed", cfg.uzume_feed_url.clone()),
        ("uzume-stories", cfg.uzume_stories_url.clone()),
        ("uzume-reels", cfg.uzume_reels_url.clone()),
        ("uzume-discover", cfg.uzume_discover_url.clone()),
    ];

    // Probe all upstreams concurrently.
    let probe_futures = upstreams.iter().map(|(name, base_url)| {
        let client = state.http.clone();
        let probe_url = format!("{base_url}/healthz");
        let name = *name;
        async move {
            let start = Instant::now();
            let result = timeout(Duration::from_secs(5), client.get(&probe_url).send()).await;

            let status = match result {
                Ok(Ok(_resp)) => {
                    let latency = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                    debug!(upstream = %name, latency_ms = latency, "upstream reachable");
                    UpstreamStatus {
                        reachable: true,
                        latency_ms: Some(latency),
                    }
                }
                Ok(Err(err)) => {
                    warn!(upstream = %name, %err, "upstream unreachable");
                    UpstreamStatus {
                        reachable: false,
                        latency_ms: None,
                    }
                }
                Err(_timeout) => {
                    warn!(upstream = %name, "upstream probe timed out");
                    UpstreamStatus {
                        reachable: false,
                        latency_ms: None,
                    }
                }
            };

            (name.to_owned(), status)
        }
    });

    let results: Vec<(String, UpstreamStatus)> = join_all(probe_futures).await;
    let db_start = Instant::now();
    let db_result = timeout(
        Duration::from_secs(5),
        sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.db),
    )
    .await;

    let database = match db_result {
        Ok(Ok(_)) => UpstreamStatus {
            reachable: true,
            latency_ms: Some(u64::try_from(db_start.elapsed().as_millis()).unwrap_or(u64::MAX)),
        },
        Ok(Err(err)) => {
            warn!(%err, "database health probe failed");
            UpstreamStatus {
                reachable: false,
                latency_ms: None,
            }
        }
        Err(_timeout) => {
            warn!("database health probe timed out");
            UpstreamStatus {
                reachable: false,
                latency_ms: None,
            }
        }
    };

    let all_ok = results.iter().all(|(_, s)| s.reachable) && database.reachable;

    let upstreams_map: HashMap<String, UpstreamStatus> = results.into_iter().collect();

    let response = HealthResponse {
        status: if all_ok { "ok" } else { "degraded" },
        upstreams: upstreams_map,
        database,
    };

    (StatusCode::OK, Json(response))
}
