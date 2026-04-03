//! Track repeated failed auth attempts so Heimdall can block brute-force traffic.

use std::time::Duration;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::client_ip::extract_client_ip;

const FAILURE_WINDOW: Duration = Duration::from_secs(15 * 60);
const FAILURE_THRESHOLD: u32 = 5;

#[derive(Clone)]
pub struct AuthFailureThrottle {
    db: PgPool,
}

impl AuthFailureThrottle {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    async fn is_blocked(&self, key: &str) -> bool {
        let now = Utc::now();
        match sqlx::query_as::<_, (i32, DateTime<Utc>)>(
            r#"
            SELECT failures, blocked_until
            FROM nyx.auth_failures
            WHERE failure_key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.db)
        .await
        {
            Ok(Some((failures, blocked_until))) => failures >= FAILURE_THRESHOLD as i32 && blocked_until > now,
            _ => false,
        }
    }

    async fn register_failure(&self, key: &str) {
        let now = Utc::now();
        let blocked_until = now + chrono::Duration::from_std(FAILURE_WINDOW).unwrap_or_else(|_| chrono::Duration::minutes(15));

        let _ = sqlx::query(
            r#"
            INSERT INTO nyx.auth_failures (failure_key, failures, blocked_until)
            VALUES ($1, 1, $2)
            ON CONFLICT (failure_key) DO UPDATE
            SET failures = CASE
                    WHEN nyx.auth_failures.blocked_until < NOW() THEN 1
                    ELSE nyx.auth_failures.failures + 1
                END,
                blocked_until = CASE
                    WHEN nyx.auth_failures.blocked_until < NOW() THEN EXCLUDED.blocked_until
                    ELSE GREATEST(nyx.auth_failures.blocked_until, EXCLUDED.blocked_until)
                END,
                updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(blocked_until)
        .execute(&self.db)
        .await;
    }
}

pub async fn auth_failure_middleware(
    axum::extract::State(throttle): axum::extract::State<AuthFailureThrottle>,
    req: Request,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(&req);
    let key = format!("{client_ip}:{}", req.uri().path());

    if throttle.is_blocked(&key).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many failed auth attempts. Try again later.",
        )
            .into_response();
    }

    let response = next.run(req).await;

    if matches!(response.status(), StatusCode::UNAUTHORIZED | StatusCode::UNPROCESSABLE_ENTITY) {
        throttle.register_failure(&key).await;
    }

    response
}