//! Health handler tests — Cycle 5.

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use serde_json::Value;
use serial_test::serial;
use tokio::net::TcpListener;
use tower::ServiceExt;

use heimdall::config::HeimdallConfig;
use heimdall::health::health_handler;
use heimdall::state::AppState;

/// Spawn a simple OK upstream server.
async fn spawn_ok_upstream() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let app = Router::new().route("/healthz", get(|| async { "ok" }));
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

/// Build a `HeimdallConfig` pointing all upstreams at the given address.
fn make_config_with_upstream(addr: SocketAddr) -> HeimdallConfig {
    let base = format!("http://{addr}");
    // Set all env vars.
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret-key-at-least-32chars!!");
        std::env::set_var("KRATOS_PUBLIC_URL", &base);
        std::env::set_var("MATRIX_URL", &base);
        std::env::set_var("UZUME_PROFILES_URL", &base);
        std::env::set_var("UZUME_FEED_URL", &base);
        std::env::set_var("UZUME_STORIES_URL", &base);
        std::env::set_var("UZUME_REELS_URL", &base);
        std::env::set_var("UZUME_DISCOVER_URL", &base);
    }
    HeimdallConfig::from_env().unwrap()
}

/// Build a `HeimdallConfig` where one upstream is unreachable.
fn make_config_with_bad_upstream(good_addr: SocketAddr) -> HeimdallConfig {
    let good = format!("http://{good_addr}");
    // Port 1 is guaranteed unreachable.
    let bad = "http://127.0.0.1:1".to_owned();
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret-key-at-least-32chars!!");
        std::env::set_var("KRATOS_PUBLIC_URL", &good);
        std::env::set_var("MATRIX_URL", &bad);
        std::env::set_var("UZUME_PROFILES_URL", &good);
        std::env::set_var("UZUME_FEED_URL", &good);
        std::env::set_var("UZUME_STORIES_URL", &good);
        std::env::set_var("UZUME_REELS_URL", &good);
        std::env::set_var("UZUME_DISCOVER_URL", &good);
    }
    HeimdallConfig::from_env().unwrap()
}

fn make_app(config: HeimdallConfig) -> Router {
    let state = AppState::new(config);
    Router::new()
        .route("/healthz", get(health_handler))
        .with_state(state)
}

// 1. All upstreams respond → /healthz returns 200 with status:"ok".
#[tokio::test]
#[serial]
async fn test_all_upstreams_healthy_returns_ok() {
    let addr = spawn_ok_upstream().await;
    let app = make_app(make_config_with_upstream(addr));

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

// 2. One upstream unreachable → /healthz returns 200 with status:"degraded".
#[tokio::test]
#[serial]
async fn test_one_upstream_unreachable_returns_degraded() {
    let good_addr = spawn_ok_upstream().await;
    let app = make_app(make_config_with_bad_upstream(good_addr));

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    // Still 200 — degraded is informational, not a hard failure.
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "degraded");
}

// 3. Response is valid JSON with "upstreams" object.
#[tokio::test]
#[serial]
async fn test_response_has_upstreams_object() {
    let addr = spawn_ok_upstream().await;
    let app = make_app(make_config_with_upstream(addr));

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json["upstreams"].is_object(),
        "response must contain 'upstreams' object"
    );
}

// 4. Response includes latency_ms for each upstream.
#[tokio::test]
#[serial]
async fn test_response_includes_latency_ms() {
    let addr = spawn_ok_upstream().await;
    let app = make_app(make_config_with_upstream(addr));

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let upstreams = json["upstreams"].as_object().unwrap();
    assert!(!upstreams.is_empty(), "upstreams object must not be empty");

    // Every reachable upstream must have a latency_ms value.
    for (name, status) in upstreams {
        if status["reachable"].as_bool().unwrap_or(false) {
            assert!(
                !status["latency_ms"].is_null(),
                "reachable upstream '{name}' must have latency_ms"
            );
        }
    }
}
