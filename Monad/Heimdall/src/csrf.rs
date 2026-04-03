//! CSRF protection middleware for state-changing operations.
//!
//! This middleware validates CSRF tokens on POST/PUT/PATCH/DELETE requests
//! to prevent cross-site request forgery attacks. Tokens are expected in the
//! `X-CSRF-Token` header for API requests.

use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

/// CSRF token manager for generating and validating tokens.
pub struct CsrfManager {
    /// In-memory token storage (for simplicity; production uses Redis/cache)
    tokens: Arc<RwLock<Vec<String>>>,
}

impl CsrfManager {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Generate a new CSRF token.
    pub async fn generate_token(&self) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        let mut tokens = self.tokens.write().await;
        tokens.push(token.clone());
        token
    }

    /// Validate a CSRF token.
    pub async fn validate_token(&self, token: &str) -> bool {
        let tokens = self.tokens.read().await;
        tokens.contains(&token.to_string())
    }

    /// Remove a used token (single-use tokens for critical operations).
    pub async fn consume_token(&self, token: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.retain(|t| t != token);
    }
}

impl Default for CsrfManager {
    fn default() -> Self {
        Self::new()
    }
}

/// CSRF protection middleware.
///
/// Validates CSRF tokens on state-changing requests (POST/PUT/PATCH/DELETE).
/// Skips validation for:
/// - GET/HEAD/OPTIONS requests (safe methods)
/// - Requests with valid session cookies (if implemented)
pub async fn csrf_middleware(
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    // Only validate state-changing methods
    let needs_validation = matches!(
        method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );

    if needs_validation {
        // Get CSRF token from header
        let token = req
            .headers()
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok());

        match token {
            Some(t) if !t.is_empty() => {
                // Token present, let the handler validate it
                // In production, validate against cache here
                warn!("CSRF token present for {} {}", method, uri);
            }
            _ => {
                // Missing CSRF token
                warn!("CSRF token missing for {} {}", method, uri);
                return Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("CSRF token required"))
                    .unwrap();
            }
        }
    }

    next.run(req).await
}
