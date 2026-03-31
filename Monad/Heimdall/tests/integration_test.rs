//! Integration tests — Cycle 6.
//!
//! Tests the full router (routes + middleware stack) using mock upstreams.

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::routing::any;
use axum::Router;
use tokio::net::TcpListener;
use tower::ServiceExt;

use heimdall::config::HeimdallConfig;
use heimdall::jwt::encode_jwt;
use heimdall::routes::build_router;
use heimdall::state::AppState;

const SECRET: &str = "integration-test-secret-32chars!!";
const IDENTITY: &str = "01920000-0000-7000-8000-000000000001";

/// Spawn a simple "ok" upstream server.
async fn spawn_upstream() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/healthz", any(|| async { "ok" }))
            .fallback(any(|| async { (StatusCode::OK, "upstream-ok") }));
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

async fn make_app() -> Router {
    let upstream = spawn_upstream().await;
    let base = format!("http://{upstream}");

    unsafe {
        std::env::set_var("JWT_SECRET", SECRET);
        std::env::set_var("KRATOS_PUBLIC_URL", &base);
        std::env::set_var("MATRIX_URL", &base);
        std::env::set_var("UZUME_PROFILES_URL", &base);
        std::env::set_var("UZUME_FEED_URL", &base);
        std::env::set_var("UZUME_STORIES_URL", &base);
        std::env::set_var("UZUME_REELS_URL", &base);
        std::env::set_var("UZUME_DISCOVER_URL", &base);
    }

    let config = HeimdallConfig::from_env().unwrap();
    let state = AppState::new(config);
    build_router(state)
}

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

// 1. GET /healthz → 200 (no auth needed).
#[tokio::test]
async fn test_healthz_no_auth_required() {
    let app = make_app().await;

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// 2. GET /api/nyx/auth/login → proxied without auth check (even without JWT).
#[tokio::test]
async fn test_nyx_auth_routes_public() {
    let app = make_app().await;

    let req = Request::builder()
        .uri("/api/nyx/auth/login")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    // Must not be 401 — auth routes are public.
    assert_ne!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "/api/nyx/auth/* must be accessible without JWT"
    );
}

// 3. GET /api/uzume/profiles/me without JWT → 401.
#[tokio::test]
async fn test_protected_route_without_jwt_returns_401() {
    let app = make_app().await;

    let req = Request::builder()
        .uri("/api/uzume/profiles/me")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// 4. GET /api/uzume/profiles/me with valid JWT → proxied to upstream (200).
#[tokio::test]
async fn test_protected_route_with_valid_jwt_proxied() {
    let app = make_app().await;

    let token = encode_jwt(IDENTITY, SECRET, 3600).unwrap();

    let req = Request::builder()
        .uri("/api/uzume/profiles/me")
        .header("Authorization", bearer(&token))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// 5. Unknown path → 404.
#[tokio::test]
async fn test_unknown_path_returns_404() {
    let app = make_app().await;

    let req = Request::builder()
        .uri("/api/does-not-exist/at-all")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// 6. X-Request-ID header generated and present in response.
#[tokio::test]
async fn test_request_id_header_present_in_response() {
    let app = make_app().await;

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert!(
        resp.headers().contains_key("x-request-id"),
        "x-request-id header must be present in response"
    );
}

// 7. /api/nyx/account/* requires JWT.
#[tokio::test]
async fn test_nyx_account_requires_jwt() {
    let app = make_app().await;

    let req = Request::builder()
        .uri("/api/nyx/account/settings")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// 8. /api/uzume/feed/* requires JWT.
#[tokio::test]
async fn test_uzume_feed_requires_jwt() {
    let app = make_app().await;

    let req = Request::builder()
        .uri("/api/uzume/feed/posts")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// 9. Correct upstream chosen: /api/uzume/stories/* with JWT → upstream hit.
#[tokio::test]
async fn test_correct_upstream_chosen_for_stories() {
    let app = make_app().await;
    let token = encode_jwt(IDENTITY, SECRET, 3600).unwrap();

    let req = Request::builder()
        .uri("/api/uzume/stories/active")
        .header("Authorization", bearer(&token))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
