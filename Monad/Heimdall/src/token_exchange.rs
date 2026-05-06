//! POST /api/nyx/auth/token — exchange a Kratos session token for a Nyx JWT.
//!
//! # Flow
//!
//! 1. Client POSTs `{ "session_token": "ory_st_xxx" }`.
//! 2. Heimdall validates the token with Kratos `GET /sessions/whoami`.
//! 3. On success, Heimdall issues a signed JWT (`sub` = Kratos identity UUID)
//!    and returns it as `{ "access_token": "…", "token_type": "Bearer",
//!    "expires_in": <secs> }`.
//!
//! The JWT is then used as `Authorization: Bearer <token>` on all protected
//! endpoints (`/api/nyx/account/*`, `/api/uzume/*`, …).
//!
//! # Security
//!
//! - The endpoint is rate-limited and auth-failure-throttled alongside all
//!   other `/api/nyx/auth/*` routes.
//! - An invalid or expired Kratos session returns `401`.
//! - Kratos is called over the internal Docker network (never user-supplied URL).

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tracing::{info, warn};

use crate::jwt::encode_jwt;
use crate::state::AppState;

// ── Request body ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TokenExchangeRequest {
    session_token: String,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// Axum handler for `POST /api/nyx/auth/token`.
pub async fn token_exchange_handler(
    State(state): State<AppState>,
    Json(body): Json<TokenExchangeRequest>,
) -> Response {
    let session_token = body.session_token.trim().to_owned();

    if session_token.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "session_token is required",
                "code": "missing_session_token"
            })),
        )
            .into_response();
    }

    // ── Validate with Kratos /sessions/whoami ─────────────────────────────────
    let whoami_url = format!("{}/sessions/whoami", state.config.kratos_public_url);

    let kratos_resp = match state
        .http
        .get(&whoami_url)
        .header("X-Session-Token", &session_token)
        .send()
        .await
    {
        Ok(r) => r,
        Err(err) => {
            warn!(%err, "Kratos unreachable during token exchange");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Authentication provider unreachable",
                    "code": "auth_unavailable"
                })),
            )
                .into_response();
        }
    };

    match kratos_resp.status().as_u16() {
        200 => {} // valid session — continue below
        401 | 403 => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Session is invalid or expired",
                    "code": "session_invalid"
                })),
            )
                .into_response();
        }
        status => {
            warn!(status, "Kratos whoami returned unexpected status");
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": "Authentication provider error",
                    "code": "auth_provider_error"
                })),
            )
                .into_response();
        }
    }

    // ── Parse identity ID from whoami response ────────────────────────────────
    let session_body: serde_json::Value = match kratos_resp.json().await {
        Ok(v) => v,
        Err(err) => {
            warn!(%err, "failed to parse Kratos whoami response");
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": "Authentication provider returned invalid response",
                    "code": "auth_provider_error"
                })),
            )
                .into_response();
        }
    };

    let identity_id = match session_body["identity"]["id"].as_str() {
        Some(id) => id.to_owned(),
        None => {
            warn!("Kratos whoami response missing identity.id");
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": "Authentication provider returned incomplete response",
                    "code": "auth_provider_error"
                })),
            )
                .into_response();
        }
    };

    // ── Issue Nyx JWT ─────────────────────────────────────────────────────────
    let token = match encode_jwt(
        &identity_id,
        &state.config.jwt_secret,
        state.config.jwt_private_key_pem.as_deref(),
        state.config.jwt_expiry_secs,
    ) {
        Ok(t) => t,
        Err(err) => {
            warn!(%err, "failed to encode JWT");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to issue access token",
                    "code": "token_issue_error"
                })),
            )
                .into_response();
        }
    };

    let redacted = &identity_id[..identity_id.len().min(8)];
    info!(identity_id = %redacted, "issued JWT via session exchange");

    (
        StatusCode::OK,
        Json(json!({
            "access_token": token,
            "token_type": "Bearer",
            "expires_in": state.config.jwt_expiry_secs,
        })),
    )
        .into_response()
}
