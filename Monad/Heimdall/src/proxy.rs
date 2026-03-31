//! Core reverse-proxy logic.
//!
//! [`proxy_request`] is the single entry point. It strips the gateway path
//! prefix, copies headers (minus hop-by-hop ones per RFC 2616 §13.5.1),
//! optionally injects `X-Nyx-Identity-Id`, forwards the request body, and
//! maps the upstream response back to an Axum response.
//!
//! On connection errors the function returns `502 Bad Gateway` rather than
//! panicking.
//!
//! # Example
//!
//! ```rust,no_run
//! # async fn example() {
//! use axum::body::Body;
//! use axum::extract::Request;
//! use axum::http::Method;
//! use reqwest::Client;
//! use heimdall::proxy::proxy_request;
//!
//! let client = Client::new();
//! let req = Request::builder()
//!     .method(Method::GET)
//!     .uri("/api/uzume/profiles/me")
//!     .body(Body::empty())
//!     .unwrap();
//! let response = proxy_request(
//!     &client,
//!     "http://localhost:3001",
//!     "/api/uzume/profiles",
//!     req,
//!     Some("01920000-0000-7000-8000-000000000001"),
//! ).await;
//! # }
//! ```

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tracing::{error, info};

/// Hop-by-hop headers that must NOT be forwarded to the upstream (RFC 2616 §13.5.1).
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

/// Forward an incoming Axum request to `upstream_base`, stripping `prefix_to_strip`
/// from the path before forwarding.
///
/// # Parameters
///
/// - `http_client` — shared `reqwest::Client` (connection-pool reused).
/// - `upstream_base` — base URL of the upstream service, e.g. `"http://localhost:3001"`.
/// - `prefix_to_strip` — path prefix to remove, e.g. `"/api/uzume/profiles"`.
/// - `req` — the original Axum request.
/// - `identity_id` — if `Some`, injects an `X-Nyx-Identity-Id` header.
///
/// # Returns
///
/// An Axum `Response`. Returns `502` on connection failure.
pub async fn proxy_request(
    http_client: &reqwest::Client,
    upstream_base: &str,
    prefix_to_strip: &str,
    req: Request,
    identity_id: Option<&str>,
) -> Response {
    // ── 1. Decompose the incoming request ─────────────────────────────────
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    // ── 2. Build upstream URL ─────────────────────────────────────────────
    let path = uri.path();
    let remaining = path
        .strip_prefix(prefix_to_strip)
        .unwrap_or(path);

    // Ensure remaining path starts with '/' (strip_prefix may produce "").
    let remaining = if remaining.is_empty() { "/" } else { remaining };

    let upstream_url = match uri.query() {
        Some(q) => format!("{upstream_base}{remaining}?{q}"),
        None => format!("{upstream_base}{remaining}"),
    };

    info!(
        method = %method,
        upstream_url = %upstream_url,
        "proxying request"
    );

    // ── 3. Build upstream request ─────────────────────────────────────────
    let mut upstream_req = http_client.request(
        reqwest::Method::from_bytes(method.as_str().as_bytes())
            .unwrap_or(reqwest::Method::GET),
        &upstream_url,
    );

    // Copy headers, skipping hop-by-hop ones.
    for (name, value) in &headers {
        let name_lower = name.as_str().to_lowercase();
        if HOP_BY_HOP.contains(&name_lower.as_str()) {
            continue;
        }
        if let Ok(val_str) = value.to_str() {
            upstream_req = upstream_req.header(name.as_str(), val_str);
        }
    }

    // Inject identity header if provided.
    if let Some(id) = identity_id {
        upstream_req = upstream_req.header("X-Nyx-Identity-Id", id);
    }

    // Forward body.
    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(err) => {
            error!(%err, "failed to read request body");
            return bad_gateway("Failed to read request body");
        }
    };
    upstream_req = upstream_req.body(body_bytes);

    // ── 4. Send to upstream ───────────────────────────────────────────────
    let upstream_response = match upstream_req.send().await {
        Ok(resp) => resp,
        Err(err) => {
            error!(%err, upstream_url = %upstream_url, "upstream connection failed");
            return bad_gateway("Upstream service is unreachable");
        }
    };

    // ── 5. Map upstream response → Axum response ──────────────────────────
    let status = StatusCode::from_u16(upstream_response.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);

    let mut response_builder = axum::response::Response::builder().status(status);

    // Copy response headers (also strip hop-by-hop).
    for (name, value) in upstream_response.headers() {
        let name_lower = name.as_str().to_lowercase();
        if HOP_BY_HOP.contains(&name_lower.as_str()) {
            continue;
        }
        response_builder = response_builder.header(name.as_str(), value.as_bytes());
    }

    let response_bytes = match upstream_response.bytes().await {
        Ok(b) => b,
        Err(err) => {
            error!(%err, "failed to read upstream response body");
            return bad_gateway("Failed to read upstream response");
        }
    };

    response_builder
        .body(Body::from(response_bytes))
        .unwrap_or_else(|_| bad_gateway("Failed to construct response"))
}

/// Build a `502 Bad Gateway` response with an [`ErrorResponse`](crate) JSON body.
fn bad_gateway(message: &str) -> Response {
    let body = json!({
        "error": message,
        "code": "bad_gateway"
    });
    (
        StatusCode::BAD_GATEWAY,
        axum::Json(body),
    )
        .into_response()
}
