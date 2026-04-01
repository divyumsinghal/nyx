use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct InMemoryRateLimiter {
    buckets: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    pub limit: u32,
    pub window: Duration,
}

impl InMemoryRateLimiter {
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            buckets: Arc::default(),
            limit,
            window,
        }
    }
}

pub async fn rate_limit(
    State(limiter): State<InMemoryRateLimiter>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = format!("{}:{}", addr.ip(), req.uri().path());
    let mut map = limiter.buckets.lock().await;
    let now = Instant::now();
    let entry = map.entry(key).or_insert((0, now + limiter.window));

    if now > entry.1 {
        *entry = (0, now + limiter.window);
    }

    if entry.0 >= limiter.limit {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    entry.0 += 1;
    drop(map);
    Ok(next.run(req).await)
}
