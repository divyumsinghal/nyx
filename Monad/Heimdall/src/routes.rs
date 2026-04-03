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

use axum::extract::{Extension, Json as ExtractJson, Query, Request, State};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::middleware;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{any, get};
use axum::Router;
use heka::NyxIdStatus;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

use crate::auth_layer::{auth_middleware, ValidatedIdentity};
use crate::auth_throttle::{auth_failure_middleware, AuthFailureThrottle};
use crate::proxy::proxy_request;
use crate::rate_limit::{
    auth_rate_limiter_with_db, default_rate_limiter_with_db, rate_limit_middleware,
};
use crate::state::AppState;
use crate::{health, websocket};

/// Construct the complete Axum [`Router`] with all routes and middleware layers.
///
/// Middleware is applied in this order (outermost → innermost):
/// 1. CORS (strict origin allowlist)
/// 2. Rate limiting (DDoS protection)
/// 3. Request ID injection (`X-Request-Id`)
/// 4. Tracing (`TraceLayer`)
/// 5. Auth middleware (JWT validation, populates `ValidatedIdentity` extension)
pub fn build_router(state: AppState) -> Router {
    // Security: Strict CORS - loaded from env-driven config only.
    let allowed_origins = AllowOrigin::list(
        state
            .config
            .cors_allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect::<Vec<HeaderValue>>(),
    );

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-session-token"),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(86400));

    // Security: Rate limiting - stricter for auth endpoints
    let default_limiter = default_rate_limiter_with_db(state.db.clone());
    let auth_limiter = auth_rate_limiter_with_db(state.db.clone());
    let auth_failure_throttle = AuthFailureThrottle::new(state.db.clone());
    let security_headers = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static(
                "default-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'",
            ),
        ));

    let public_routes = Router::new()
        // ── Health (public) ───────────────────────────────────────────────
        .route("/healthz", get(health::health_handler))
        // ── Nyx ID check (public - no auth required) ──────────────────────
        .route(
            "/api/nyx/id/check-availability",
            get(check_nyx_id_availability_get).post(check_nyx_id_availability_post),
        )
        .layer(middleware::from_fn_with_state(
            default_limiter.clone(),
            rate_limit_middleware,
        ));

    let auth_routes = Router::new()
        // ── Nyx auth (public — no JWT enforcement, strict rate limit) ───────
        .route("/api/nyx/auth/{*path}", any(nyx_auth_proxy))
        .layer(middleware::from_fn_with_state(
            auth_limiter,
            rate_limit_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_failure_throttle,
            auth_failure_middleware,
        ));

    let protected_routes = Router::new()
        // ── Protected routes (standard rate limit) ──────────────────────────
        .route("/api/nyx/account/{*path}", any(nyx_account_proxy))
        .route("/api/nyx/messaging/{*path}", any(nyx_messaging_proxy))
        .route("/api/uzume/profiles/{*path}", any(uzume_profiles_proxy))
        .route("/api/uzume/feed/{*path}", any(uzume_feed_proxy))
        .route("/api/uzume/stories/{*path}", any(uzume_stories_proxy))
        .route("/api/uzume/reels/{*path}", any(uzume_reels_proxy))
        .route("/api/uzume/discover/{*path}", any(uzume_discover_proxy))
        .route("/api/uzume/ws", get(websocket::ws_handler))
        .layer(middleware::from_fn_with_state(
            default_limiter,
            rate_limit_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    public_routes
        .merge(auth_routes)
        .merge(protected_routes)
        // ── Middleware stack (applied outermost last) ─────────────────────
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .layer(security_headers)
        .layer(cors)
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

// ── Nyx ID handlers ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CheckNyxIdQuery {
    id: String,
}

#[derive(Debug, Deserialize)]
struct CheckNyxIdRequest {
    id: String,
}

#[derive(Debug, Serialize)]
struct CheckNyxIdResponse {
    available: bool,
    id: String,
    reason: Option<String>,
}

/// GET /api/nyx/id/check-availability?id={nyx_id}
async fn check_nyx_id_availability_get(
    State(state): State<AppState>,
    Query(query): Query<CheckNyxIdQuery>,
) -> Result<Json<CheckNyxIdResponse>, StatusCode> {
    check_nyx_id_availability_impl(&state, &query.id).await
}

/// POST /api/nyx/id/check-availability { "id": "..." }
async fn check_nyx_id_availability_post(
    State(state): State<AppState>,
    ExtractJson(body): ExtractJson<CheckNyxIdRequest>,
) -> Result<Json<CheckNyxIdResponse>, StatusCode> {
    check_nyx_id_availability_impl(&state, &body.id).await
}

async fn check_nyx_id_availability_impl(
    state: &AppState,
    nyx_id: &str,
) -> Result<Json<CheckNyxIdResponse>, StatusCode> {
    let trimmed = nyx_id.trim();
    if trimmed.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match state.nyx_id_registry.check_availability(trimmed).await {
        Ok(status) => {
            let (available, reason) = match status {
                NyxIdStatus::Available => (true, None),
                NyxIdStatus::Taken => (false, Some("This Nyx ID is already taken".to_string())),
                NyxIdStatus::Invalid { reason } => (false, Some(reason)),
            };

            Ok(Json(CheckNyxIdResponse {
                available,
                id: trimmed.to_string(),
                reason,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to check Nyx ID availability: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
