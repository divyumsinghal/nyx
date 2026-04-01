//! WebSocket upgrade handler for Heimdall.
//!
//! `GET /api/uzume/ws` upgrades authenticated connections to WebSocket.
//! The relay to the upstream feed service is a future TODO; for now the
//! handler echoes received messages back to the client so that the upgrade
//! itself can be tested end-to-end.
//!
//! # Authentication
//!
//! A `ValidatedIdentity` extension must be present (inserted by
//! [`auth_middleware`](crate::auth_layer::auth_middleware)). If absent the
//! handler returns `401` without upgrading the connection.

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use serde_json::json;
use tracing::{debug, info, warn};

use crate::auth_layer::ValidatedIdentity;
use crate::state::AppState;

/// Axum handler for `GET /api/uzume/ws`.
///
/// Requires a valid JWT delivered via the standard `Authorization: Bearer`
/// header (validated by the global auth middleware). Upgrades to WebSocket on
/// success.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
    identity: Option<Extension<ValidatedIdentity>>,
) -> Response {
    let Some(Extension(identity)) = identity else {
        warn!("WebSocket upgrade rejected: no authenticated identity");
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "error": "Authentication required",
                "code": "unauthorized"
            })),
        )
            .into_response();
    };

    info!(identity_id = %identity.identity_id, "WebSocket upgrade accepted");

    ws.on_upgrade(move |socket| handle_socket(socket, identity.identity_id))
}

/// Handle a single WebSocket connection.
///
/// Current behaviour: echo messages back to the sender.
///
/// TODO: implement bidirectional relay to the upstream `Uzume-feed` service
/// for real-time event streaming.
async fn handle_socket(mut socket: WebSocket, identity_id: String) {
    debug!(identity_id = %identity_id, "WebSocket connection opened");

    while let Some(result) = socket.recv().await {
        match result {
            Ok(msg) => {
                debug!(identity_id = %identity_id, "WebSocket message received; echoing back");
                if socket.send(msg).await.is_err() {
                    break;
                }
            }
            Err(err) => {
                warn!(identity_id = %identity_id, %err, "WebSocket receive error");
                break;
            }
        }
    }

    debug!(identity_id = %identity_id, "WebSocket connection closed");
}
