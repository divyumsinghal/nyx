//! JWT authentication middleware for Axum.
//!
//! [`auth_middleware`] is a `from_fn_with_state` middleware that inspects the
//! `Authorization: Bearer <token>` header. If a valid token is present it
//! inserts a [`ValidatedIdentity`] extension into the request so that
//! downstream handlers can extract the caller's identity without re-parsing
//! the JWT.
//!
//! # Public vs protected routes
//!
//! The middleware itself does NOT block requests without a token — it simply
//! leaves the extension absent. Protected route handlers must check for the
//! extension and return `401` if it is `None`. Public routes (like
//! `/api/nyx/auth/*`) can proceed normally.
//!
//! # Errors returned
//!
//! When a token *is* present but invalid the middleware returns `401` with
//! a JSON body `{ "error": "…", "code": "…" }` and does NOT call `next`.

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tracing::{debug, info, warn};

use crate::jwt::{self, JwtError};
use crate::state::AppState;

// ── ValidatedIdentity ────────────────────────────────────────────────────────

/// Request extension inserted by [`auth_middleware`] when a valid JWT is present.
///
/// Handlers on protected routes extract this via
/// `Extension<ValidatedIdentity>` (or `Option<Extension<ValidatedIdentity>>`
/// for routes that support both authenticated and public access).
#[derive(Clone, Debug)]
pub struct ValidatedIdentity {
    /// The Nyx identity ID extracted from the JWT `sub` claim.
    pub identity_id: String,
}

// ── auth_middleware ───────────────────────────────────────────────────────────

/// Axum middleware that validates a Bearer JWT and populates `ValidatedIdentity`.
///
/// Behaviour:
/// - No `Authorization` header → continues with **no** `ValidatedIdentity` extension.
/// - `Authorization: Bearer <valid_token>` → inserts `ValidatedIdentity` and continues.
/// - `Authorization: Bearer <invalid/expired_token>` → returns `401 Unauthorized`.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let Some(auth_value) = auth_header else {
        // No Authorization header — proceed without identity.
        debug!("no Authorization header; continuing as unauthenticated");
        return next.run(req).await;
    };

    // Must be a Bearer token.
    let Some(token) = auth_value.strip_prefix("Bearer ") else {
        warn!("Authorization header present but not Bearer scheme");
        // Security audit: Log failed auth attempts
        audit_log_auth_failure("invalid_auth_scheme", &req);
        return unauthorized("Bearer token required", "invalid_auth_scheme");
    };

    // Validate the JWT.
    match jwt::decode_jwt(token, &state.config.jwt_secret) {
        Ok(claims) => {
            debug!(sub = %claims.sub, "JWT validated successfully");
            req.extensions_mut().insert(ValidatedIdentity {
                identity_id: claims.sub.clone(),
            });
            // Security audit: Log successful auth
            audit_log_auth_success(&claims.sub, &req);
            next.run(req).await
        }
        Err(JwtError::Expired) => {
            warn!("JWT is expired");
            audit_log_auth_failure("token_expired", &req);
            unauthorized("Token has expired", "token_expired")
        }
        Err(JwtError::Invalid) => {
            warn!("JWT is invalid");
            audit_log_auth_failure("token_invalid", &req);
            unauthorized("Token is invalid", "token_invalid")
        }
    }
}

/// Security audit: Log successful authentication
fn audit_log_auth_success(identity_id: &str, req: &Request) {
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            req.extensions().get::<ConnectInfo<std::net::SocketAddr>>()
                .map(|c| c.0.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });

    info!(
        event = "auth_success",
        identity_id = %identity_id,
        client_ip = %client_ip,
        path = %req.uri().path(),
        "Authentication successful"
    );
}

/// Security audit: Log failed authentication attempts
fn audit_log_auth_failure(reason: &str, req: &Request) {
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            req.extensions().get::<ConnectInfo<std::net::SocketAddr>>()
                .map(|c| c.0.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });

    warn!(
        event = "auth_failure",
        reason = %reason,
        client_ip = %client_ip,
        path = %req.uri().path(),
        "Authentication failed"
    );
}

/// Build a `401 Unauthorized` response with JSON body.
fn unauthorized(message: &str, code: &str) -> Response {
    let body = json!({
        "error": message,
        "code": code
    });
    (StatusCode::UNAUTHORIZED, axum::Json(body)).into_response()
}
