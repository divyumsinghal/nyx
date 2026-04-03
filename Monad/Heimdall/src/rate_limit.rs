//! Rate limiting middleware for Heimdall.
//!
//! Uses a PostgreSQL-backed token bucket per IP address + path combination.
//! This keeps rate limiting distributed across Heimdall instances.

use std::time::Duration;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::warn;

use crate::client_ip::extract_client_ip;

#[derive(Clone)]
pub struct RateLimiter {
    db: PgPool,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests allowed per window
    /// * `window` - Time window for rate limiting
    pub fn new(db: PgPool, max_requests: u32, window: Duration) -> Self {
        Self { db, max_requests, window }
    }

    /// Check if request is allowed and consume a token.
    async fn is_allowed(&self, key: &str) -> bool {
        let now = Utc::now();
        let reset_after = now + chrono::Duration::from_std(self.window).unwrap_or_else(|_| chrono::Duration::seconds(60));

        let mut tx = match self.db.begin().await {
            Ok(tx) => tx,
            Err(_) => return false,
        };

        let row = match sqlx::query_as::<_, (i32, DateTime<Utc>)>(
            r#"
            SELECT tokens, reset_at
            FROM nyx.rate_limit_buckets
            WHERE bucket_key = $1
            FOR UPDATE
            "#,
        )
        .bind(key)
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(row) => row,
            Err(_) => return false,
        };

        let allowed = match row {
            Some((tokens, reset_at)) if reset_at > now && tokens > 0 => {
                let new_tokens = tokens - 1;
                if sqlx::query(
                    r#"
                    UPDATE nyx.rate_limit_buckets
                    SET tokens = $2, reset_at = $3, updated_at = NOW()
                    WHERE bucket_key = $1
                    "#,
                )
                .bind(key)
                .bind(new_tokens)
                .bind(reset_at)
                .execute(&mut *tx)
                .await
                .is_err()
                {
                    return false;
                }
                true
            }
            Some(_) => sqlx::query(
                r#"
                UPDATE nyx.rate_limit_buckets
                SET tokens = $2, reset_at = $3, updated_at = NOW()
                WHERE bucket_key = $1
                "#,
            )
            .bind(key)
            .bind(self.max_requests.saturating_sub(1) as i32)
            .bind(reset_after)
            .execute(&mut *tx)
            .await
            .is_ok(),
            None => sqlx::query(
                r#"
                INSERT INTO nyx.rate_limit_buckets (bucket_key, tokens, reset_at)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(key)
            .bind(self.max_requests.saturating_sub(1) as i32)
            .bind(reset_after)
            .execute(&mut *tx)
            .await
            .is_ok(),
        };

        if tx.commit().await.is_err() {
            return false;
        }

        allowed
    }
}

/// Rate limiting middleware for Axum.
///
/// Returns 429 Too Many Requests if rate limit is exceeded.
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(&req);
    let key = rate_limit_key(&client_ip, req.uri().path());

    if !limiter.is_allowed(&key).await {
        warn!("Rate limit exceeded for IP: {}", client_ip);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        )
            .into_response();
    }

    next.run(req).await
}

pub fn default_rate_limiter_with_db(db: PgPool) -> RateLimiter {
    RateLimiter::new(db, 100, Duration::from_secs(60))
}

pub fn auth_rate_limiter_with_db(db: PgPool) -> RateLimiter {
    RateLimiter::new(db, 10, Duration::from_secs(60))
}

fn rate_limit_key(client_ip: &str, path: &str) -> String {
    if path.starts_with("/api/nyx/auth/") {
        format!("{client_ip}:auth")
    } else {
        format!("{client_ip}:{path}")
    }
}
