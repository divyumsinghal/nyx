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

use axum::extract::{OriginalUri, Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::client_ip::extract_client_ip;
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
    _uri: OriginalUri,
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
        audit_log_auth_failure(&state, "invalid_auth_scheme", &req);
        return unauthorized("Bearer token required", "invalid_auth_scheme");
    };

    // Validate the JWT.
    match jwt::decode_jwt(
        token,
        &state.config.jwt_secret,
        state.config.jwt_public_key_pem.as_deref(),
    ) {
        Ok(claims) => {
            match jwt::is_jti_revoked(&state.db, &claims.jti).await {
                Ok(true) => {
                    warn!("JWT has been revoked");
                    audit_log_auth_failure(&state, "token_revoked", &req);
                    return unauthorized("Token has been revoked", "token_revoked");
                }
                Ok(false) => {}
                Err(err) => {
                    warn!(%err, "failed to check JWT revocation");
                    return unauthorized("Authentication unavailable", "auth_unavailable");
                }
            }
            debug!(identity_id = %redact_identity(&claims.sub), "JWT validated successfully");
            req.extensions_mut().insert(ValidatedIdentity {
                identity_id: claims.sub.clone(),
            });
            // Security audit: Log successful auth
            let user_agent = req
                .headers()
                .get(header::USER_AGENT)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("unknown")
                .to_owned();
            let client_ip = extract_client_ip(&req);
            track_known_device(&state, &claims.sub, &user_agent, &client_ip).await;
            audit_log_auth_success(&state, &claims.sub, &req);
            next.run(req).await
        }
        Err(JwtError::Expired) => {
            warn!("JWT is expired");
            audit_log_auth_failure(&state, "token_expired", &req);
            unauthorized("Token has expired", "token_expired")
        }
        Err(JwtError::Invalid) => {
            warn!("JWT is invalid");
            audit_log_auth_failure(&state, "token_invalid", &req);
            unauthorized("Token is invalid", "token_invalid")
        }
    }
}

/// Security audit: Log successful authentication
fn audit_log_auth_success(state: &AppState, identity_id: &str, req: &Request) {
    let client_ip = extract_client_ip(req);
    let redacted_identity = redact_identity(identity_id);
    persist_auth_audit_event(
        state,
        "auth_success",
        Some(identity_id),
        &client_ip,
        req.uri().path(),
        None,
    );

    info!(
        event = "auth_success",
        identity_id = %redacted_identity,
        client_ip = %client_ip,
        path = %req.uri().path(),
        "Authentication successful"
    );
}

/// Security audit: Log failed authentication attempts
fn audit_log_auth_failure(state: &AppState, reason: &str, req: &Request) {
    let client_ip = extract_client_ip(req);
    persist_auth_audit_event(
        state,
        "auth_failure",
        None,
        &client_ip,
        req.uri().path(),
        Some(reason),
    );

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

fn redact_identity(identity_id: &str) -> String {
    let prefix: String = identity_id.chars().take(8).collect();
    format!("{prefix}...")
}

async fn track_known_device(
    state: &AppState,
    identity_id: &str,
    user_agent: &str,
    client_ip: &str,
) {
    let fingerprint = device_fingerprint(identity_id, user_agent, &client_ip);

    let known_before = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM nyx.known_devices
            WHERE identity_id = $1
              AND device_fingerprint = $2
        )
        "#,
    )
    .bind(identity_id)
    .bind(&fingerprint)
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);

    if sqlx::query(
        r#"
        INSERT INTO nyx.known_devices (identity_id, device_fingerprint, user_agent)
        VALUES ($1, $2, $3)
        ON CONFLICT (identity_id, device_fingerprint)
        DO UPDATE SET last_seen = NOW(), user_agent = EXCLUDED.user_agent
        "#,
    )
    .bind(identity_id)
    .bind(&fingerprint)
    .bind(user_agent)
    .execute(&state.db)
    .await
    .is_ok()
        && !known_before
    {
        warn!(
            identity_id = %redact_identity(identity_id),
            client_ip = %client_ip,
            "new device fingerprint observed"
        );
    }
}

fn device_fingerprint(identity_id: &str, user_agent: &str, client_ip: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(identity_id.as_bytes());
    hasher.update(b"|");
    hasher.update(user_agent.as_bytes());
    hasher.update(b"|");
    hasher.update(client_ip.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn persist_auth_audit_event(
    state: &AppState,
    event: &str,
    identity_id: Option<&str>,
    client_ip: &str,
    path: &str,
    reason: Option<&str>,
) {
    let db = state.db.clone();
    let event = event.to_owned();
    let identity_id = identity_id.map(String::from);
    let client_ip = client_ip.to_owned();
    let path = path.to_owned();
    let reason = reason.map(String::from);

    tokio::spawn(async move {
        let _ = sqlx::query(
            r#"
            INSERT INTO nyx.auth_audit_events (event, identity_id, client_ip, path, reason)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(event)
        .bind(identity_id)
        .bind(client_ip)
        .bind(path)
        .bind(reason)
        .execute(&db)
        .await;
    });
}
