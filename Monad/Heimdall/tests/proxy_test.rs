//! Proxy logic tests — Cycle 3.
//!
//! Each test spins up a real `TcpListener` + minimal `axum::Router` acting as
//! the upstream, then calls `proxy_request` and verifies the behaviour.

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::routing::get;
use axum::Router;
use reqwest::Client;
use tokio::net::TcpListener;

use heimdall::proxy::proxy_request;

/// Spawn a minimal upstream server and return its address.
async fn spawn_upstream(router: Router) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind failed");
    let addr = listener.local_addr().expect("local_addr failed");
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    addr
}

/// Build a shared `reqwest::Client`.
fn client() -> Client {
    Client::new()
}

// 1. GET /api/uzume/profiles/me → upstream receives GET /me (prefix stripped).
#[tokio::test]
async fn test_prefix_stripped_from_path() {
    use std::sync::{Arc, Mutex};

    let captured_path: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let captured_path_clone = captured_path.clone();

    let upstream = Router::new().route(
        "/me",
        get(move |req: Request| {
            let captured = captured_path_clone.clone();
            async move {
                *captured.lock().unwrap() = req.uri().path().to_owned();
                (StatusCode::OK, "ok")
            }
        }),
    );

    let addr = spawn_upstream(upstream).await;
    let base = format!("http://{addr}");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/uzume/profiles/me")
        .body(Body::empty())
        .unwrap();

    let response = proxy_request(&client(), &base, "/api/uzume/profiles", req, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*captured_path.lock().unwrap(), "/me");
}

// 2. POST /api/uzume/feed/posts → upstream receives POST /posts.
#[tokio::test]
async fn test_post_prefix_stripped() {
    use axum::routing::post;

    let upstream = Router::new().route(
        "/posts",
        post(|| async { (StatusCode::CREATED, "created") }),
    );

    let addr = spawn_upstream(upstream).await;
    let base = format!("http://{addr}");

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/uzume/feed/posts")
        .body(Body::empty())
        .unwrap();

    let response = proxy_request(&client(), &base, "/api/uzume/feed", req, None).await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

// 3. identity_id injected as X-Nyx-Identity-Id header.
#[tokio::test]
async fn test_identity_id_injected_as_header() {
    use std::sync::{Arc, Mutex};

    let captured_header: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let captured_clone = captured_header.clone();

    let upstream = Router::new().route(
        "/profile",
        get(move |headers: axum::http::HeaderMap| {
            let captured = captured_clone.clone();
            async move {
                let val = headers
                    .get("x-nyx-identity-id")
                    .and_then(|v| v.to_str().ok())
                    .map(ToOwned::to_owned);
                *captured.lock().unwrap() = val;
                (StatusCode::OK, "ok")
            }
        }),
    );

    let addr = spawn_upstream(upstream).await;
    let base = format!("http://{addr}");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/uzume/profiles/profile")
        .body(Body::empty())
        .unwrap();

    let identity_id = "01920000-0000-7000-8000-000000000001";
    let response = proxy_request(
        &client(),
        &base,
        "/api/uzume/profiles",
        req,
        Some(identity_id),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        *captured_header.lock().unwrap(),
        Some(identity_id.to_owned())
    );
}

// 4. Hop-by-hop headers (Connection, Transfer-Encoding) stripped from forwarded request.
#[tokio::test]
async fn test_hop_by_hop_headers_stripped() {
    use std::sync::{Arc, Mutex};

    let captured_headers: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = captured_headers.clone();

    let upstream = Router::new().route(
        "/data",
        get(move |headers: axum::http::HeaderMap| {
            let captured = captured_clone.clone();
            async move {
                let names: Vec<String> =
                    headers.keys().map(|k| k.as_str().to_lowercase()).collect();
                *captured.lock().unwrap() = names;
                (StatusCode::OK, "ok")
            }
        }),
    );

    let addr = spawn_upstream(upstream).await;
    let base = format!("http://{addr}");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/uzume/profiles/data")
        .header("Connection", "keep-alive")
        .header("Transfer-Encoding", "chunked")
        .header("X-Custom-Header", "preserved")
        .body(Body::empty())
        .unwrap();

    let response = proxy_request(&client(), &base, "/api/uzume/profiles", req, None).await;
    assert_eq!(response.status(), StatusCode::OK);

    let names = captured_headers.lock().unwrap().clone();
    assert!(
        !names.contains(&"connection".to_owned()),
        "Connection header must be stripped"
    );
    assert!(
        !names.contains(&"transfer-encoding".to_owned()),
        "Transfer-Encoding header must be stripped"
    );
    assert!(
        names.contains(&"x-custom-header".to_owned()),
        "Custom headers must be preserved"
    );
}

// 5. Upstream 404 → gateway returns 404 (status forwarded).
#[tokio::test]
async fn test_upstream_404_forwarded() {
    let upstream = Router::new().route("/exists", get(|| async { (StatusCode::OK, "ok") }));

    let addr = spawn_upstream(upstream).await;
    let base = format!("http://{addr}");

    // Request a path that doesn't exist on the upstream.
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/uzume/profiles/does-not-exist")
        .body(Body::empty())
        .unwrap();

    let response = proxy_request(&client(), &base, "/api/uzume/profiles", req, None).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// 6. Upstream unreachable (nothing listening) → gateway returns 502.
#[tokio::test]
async fn test_upstream_unreachable_returns_502() {
    // Port 1 is reserved and guaranteed to be unreachable.
    let base = "http://127.0.0.1:1";

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/uzume/profiles/me")
        .body(Body::empty())
        .unwrap();

    let response = proxy_request(&client(), base, "/api/uzume/profiles", req, None).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

// 7. Upstream response headers forwarded (Content-Type preserved).
#[tokio::test]
async fn test_upstream_response_headers_forwarded() {
    let upstream = Router::new().route(
        "/json",
        get(|| async {
            (
                StatusCode::OK,
                [(
                    axum::http::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                )],
                r#"{"ok":true}"#,
            )
        }),
    );

    let addr = spawn_upstream(upstream).await;
    let base = format!("http://{addr}");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/uzume/profiles/json")
        .body(Body::empty())
        .unwrap();

    let response = proxy_request(&client(), &base, "/api/uzume/profiles", req, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let ct = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("application/json"),
        "Content-Type must be forwarded"
    );
}

// 8. Query string preserved (?limit=10&cursor=abc).
#[tokio::test]
async fn test_query_string_preserved() {
    use std::sync::{Arc, Mutex};

    let captured_query: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let captured_clone = captured_query.clone();

    let upstream = Router::new().route(
        "/posts",
        get(move |req: Request| {
            let captured = captured_clone.clone();
            async move {
                let query = req.uri().query().unwrap_or("").to_owned();
                *captured.lock().unwrap() = query;
                (StatusCode::OK, "ok")
            }
        }),
    );

    let addr = spawn_upstream(upstream).await;
    let base = format!("http://{addr}");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/uzume/feed/posts?limit=10&cursor=abc")
        .body(Body::empty())
        .unwrap();

    let response = proxy_request(&client(), &base, "/api/uzume/feed", req, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(*captured_query.lock().unwrap(), "limit=10&cursor=abc");
}
