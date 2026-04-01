//! Axum router for Heimdall.
//!
//! [`build_router`] wires together all route handlers, auth middleware,
//! request-ID injection, tracing, and CORS into a single [`Router`].
//!
//! # Route table
//!
//! | Method | Path | Auth | Upstream |
//! |---|---|---|---|
//! | GET | `/healthz` | none | internal |
//! | ANY | `/api/nyx/auth/*` | none | Kratos |
//! | ANY | `/api/nyx/account/*` | JWT required | Kratos |
//! | ANY | `/api/nyx/messaging/*` | JWT required | Continuwuity |
//! | ANY | `/api/uzume/profiles/*` | JWT required | Uzume-profiles |
//! | ANY | `/api/uzume/feed/*` | JWT required | Uzume-feed |
//! | ANY | `/api/uzume/stories/*` | JWT required | Uzume-stories |
//! | ANY | `/api/uzume/reels/*` | JWT required | Uzume-reels |
//! | ANY | `/api/uzume/discover/*` | JWT required | Uzume-discover |
//! | GET | `/api/uzume/ws` | JWT required | WebSocket relay |

use axum::extract::{Extension, Request, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::Router;
use serde_json::json;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

use crate::auth_layer::{auth_middleware, ValidatedIdentity};
use crate::proxy::proxy_request;
use crate::state::AppState;
use crate::{health, websocket};

/// Construct the complete Axum [`Router`] with all routes and middleware layers.
///
/// Middleware is applied in this order (outermost → innermost):
/// 1. CORS
/// 2. Request ID injection (`X-Request-Id`)
/// 3. Tracing (`TraceLayer`)
/// 4. Auth middleware (JWT validation, populates `ValidatedIdentity` extension)
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // ── Health (public) ───────────────────────────────────────────────
        .route("/healthz", get(health::health_handler))
        // ── Nyx auth (public — no JWT enforcement) ────────────────────────
        .route("/api/nyx/auth/{*path}", any(nyx_auth_proxy))
        // ── Protected routes ──────────────────────────────────────────────
        .route("/api/nyx/account/{*path}", any(nyx_account_proxy))
        .route("/api/nyx/messaging/{*path}", any(nyx_messaging_proxy))
        .route("/api/uzume/profiles/{*path}", any(uzume_profiles_proxy))
        .route("/api/uzume/feed/{*path}", any(uzume_feed_proxy))
        .route("/api/uzume/stories/{*path}", any(uzume_stories_proxy))
        .route("/api/uzume/reels/{*path}", any(uzume_reels_proxy))
        .route("/api/uzume/discover/{*path}", any(uzume_discover_proxy))
        .route("/api/uzume/ws", get(websocket::ws_handler))
        // ── Middleware stack (applied outermost last) ─────────────────────
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ── Public proxy handlers ─────────────────────────────────────────────────────

/// Proxy `ANY /api/nyx/auth/*` → Kratos (no auth required).
async fn nyx_auth_proxy(State(state): State<AppState>, req: Request) -> Response {
    proxy_request(
        &state.http,
        &state.config.kratos_public_url,
        "/api/nyx/auth",
        req,
        None,
    )
    .await
}

// ── Protected proxy handlers ──────────────────────────────────────────────────

/// Proxy `ANY /api/nyx/account/*` → Kratos (JWT required).
async fn nyx_account_proxy(
    State(state): State<AppState>,
    identity: Option<Extension<ValidatedIdentity>>,
    req: Request,
) -> Response {
    let Some(Extension(identity)) = identity else {
        return require_auth();
    };
    proxy_request(
        &state.http,
        &state.config.kratos_public_url,
        "/api/nyx/account",
        req,
        Some(&identity.identity_id),
    )
    .await
}

/// Proxy `ANY /api/nyx/messaging/*` → Continuwuity (JWT required).
async fn nyx_messaging_proxy(
    State(state): State<AppState>,
    identity: Option<Extension<ValidatedIdentity>>,
    req: Request,
) -> Response {
    let Some(Extension(identity)) = identity else {
        return require_auth();
    };
    proxy_request(
        &state.http,
        &state.config.matrix_url,
        "/api/nyx/messaging",
        req,
        Some(&identity.identity_id),
    )
    .await
}

/// Proxy `ANY /api/uzume/profiles/*` → Uzume-profiles (JWT required).
async fn uzume_profiles_proxy(
    State(state): State<AppState>,
    identity: Option<Extension<ValidatedIdentity>>,
    req: Request,
) -> Response {
    let Some(Extension(identity)) = identity else {
        return require_auth();
    };
    proxy_request(
        &state.http,
        &state.config.uzume_profiles_url,
        "/api/uzume/profiles",
        req,
        Some(&identity.identity_id),
    )
    .await
}

/// Proxy `ANY /api/uzume/feed/*` → Uzume-feed (JWT required).
async fn uzume_feed_proxy(
    State(state): State<AppState>,
    identity: Option<Extension<ValidatedIdentity>>,
    req: Request,
) -> Response {
    let Some(Extension(identity)) = identity else {
        return require_auth();
    };
    proxy_request(
        &state.http,
        &state.config.uzume_feed_url,
        "/api/uzume/feed",
        req,
        Some(&identity.identity_id),
    )
    .await
}

/// Proxy `ANY /api/uzume/stories/*` → Uzume-stories (JWT required).
async fn uzume_stories_proxy(
    State(state): State<AppState>,
    identity: Option<Extension<ValidatedIdentity>>,
    req: Request,
) -> Response {
    let Some(Extension(identity)) = identity else {
        return require_auth();
    };
    proxy_request(
        &state.http,
        &state.config.uzume_stories_url,
        "/api/uzume/stories",
        req,
        Some(&identity.identity_id),
    )
    .await
}

/// Proxy `ANY /api/uzume/reels/*` → Uzume-reels (JWT required).
async fn uzume_reels_proxy(
    State(state): State<AppState>,
    identity: Option<Extension<ValidatedIdentity>>,
    req: Request,
) -> Response {
    let Some(Extension(identity)) = identity else {
        return require_auth();
    };
    proxy_request(
        &state.http,
        &state.config.uzume_reels_url,
        "/api/uzume/reels",
        req,
        Some(&identity.identity_id),
    )
    .await
}

/// Proxy `ANY /api/uzume/discover/*` → Uzume-discover (JWT required).
async fn uzume_discover_proxy(
    State(state): State<AppState>,
    identity: Option<Extension<ValidatedIdentity>>,
    req: Request,
) -> Response {
    let Some(Extension(identity)) = identity else {
        return require_auth();
    };
    proxy_request(
        &state.http,
        &state.config.uzume_discover_url,
        "/api/uzume/discover",
        req,
        Some(&identity.identity_id),
    )
    .await
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return `401 Unauthorized` for protected routes without a valid identity.
fn require_auth() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({
            "error": "Authentication required",
            "code": "unauthorized"
        })),
    )
        .into_response()
}
