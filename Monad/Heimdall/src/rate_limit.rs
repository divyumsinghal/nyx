//! Rate limiting middleware for Heimdall.
//!
//! Uses an in-memory token bucket per IP address + path combination.
//! This is a basic protection against DDoS and brute force attacks.
//! For production scale, use a Redis-backed rate limiter.

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tokio::sync::Mutex;
use tracing::warn;

/// Per-IP + per-path rate limiter using token bucket algorithm.
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    max_requests: u32,
    window: Duration,
}

struct TokenBucket {
    tokens: u32,
    last_updated: Instant,
    reset_at: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests allowed per window
    /// * `window` - Time window for rate limiting
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    /// Check if request is allowed and consume a token.
    async fn is_allowed(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();

        let bucket = buckets.entry(key.to_string()).or_insert_with(|| TokenBucket {
            tokens: self.max_requests,
            last_updated: now,
            reset_at: now + self.window,
        });

        // Reset bucket if window has passed
        if now > bucket.reset_at {
            bucket.tokens = self.max_requests;
            bucket.reset_at = now + self.window;
        }

        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            true
        } else {
            false
        }
    }
}

/// Rate limiting middleware for Axum.
///
/// Returns 429 Too Many Requests if rate limit is exceeded.
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    // Use IP + path as the rate limit key
    let key = format!("{}:{}", addr.ip(), req.uri().path());

    if !limiter.is_allowed(&key).await {
        warn!("Rate limit exceeded for IP: {}", addr.ip());
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        )
            .into_response();
    }

    next.run(req).await
}

/// Default rate limiter: 100 requests per minute per IP
pub fn default_rate_limiter() -> RateLimiter {
    RateLimiter::new(100, Duration::from_secs(60))
}

/// Stricter rate limiter for auth endpoints: 10 requests per minute per IP
pub fn auth_rate_limiter() -> RateLimiter {
    RateLimiter::new(10, Duration::from_secs(60))
}
