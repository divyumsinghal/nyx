//! Auth middleware tests — Cycle 4.
//!
//! Uses `tower::ServiceExt::oneshot` to send individual requests through the
//! middleware stack and inspect the response or request extensions.

use axum::body::Body;
use axum::extract::{Extension, Request};
use axum::http::{header, StatusCode};
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde_json::Value;
use tower::ServiceExt;

use heimdall::auth_layer::{auth_middleware, ValidatedIdentity};
use heimdall::config::HeimdallConfig;
use heimdall::jwt::encode_jwt;
use heimdall::state::AppState;

const SECRET: &str = "test-secret-key-at-least-32chars!!";
const IDENTITY: &str = "01920000-0000-7000-8000-000000000001";

fn make_state() -> AppState {
    // Build a minimal config with only jwt_secret filled in.
    // SAFETY: These are test-only env vars set under a unique prefix.
    unsafe {
        std::env::set_var("JWT_SECRET", SECRET);
        std::env::set_var("KRATOS_PUBLIC_URL", "http://kratos:4433");
        std::env::set_var("MATRIX_URL", "http://matrix:8448");
        std::env::set_var("UZUME_PROFILES_URL", "http://profiles:3001");
        std::env::set_var("UZUME_FEED_URL", "http://feed:3002");
        std::env::set_var("UZUME_STORIES_URL", "http://stories:3003");
        std::env::set_var("UZUME_REELS_URL", "http://reels:3004");
        std::env::set_var("UZUME_DISCOVER_URL", "http://discover:3005");
    }
    let config = HeimdallConfig::from_env().expect("config should load");
    AppState::new(config)
}

/// Build a simple echo router that returns the ValidatedIdentity extension as JSON.
fn echo_identity_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/test",
            get(
                |identity: Option<Extension<ValidatedIdentity>>| async move {
                    match identity {
                        Some(Extension(id)) => {
                            (StatusCode::OK, format!("id:{}", id.identity_id)).into_response()
                        }
                        None => (StatusCode::OK, "no-identity").into_response(),
                    }
                },
            ),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

// 1. No Authorization header → extension is None (request proceeds without identity).
#[tokio::test]
async fn test_no_auth_header_proceeds_without_identity() {
    let state = make_state();
    let app = echo_identity_router(state);

    let req = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"no-identity");
}

// 2. Valid Bearer JWT → extension has ValidatedIdentity { identity_id }.
#[tokio::test]
async fn test_valid_bearer_sets_identity_extension() {
    let state = make_state();
    let app = echo_identity_router(state);

    let token = encode_jwt(IDENTITY, SECRET, 3600).expect("encode should succeed");
    let auth_value = format!("Bearer {token}");

    let req = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, auth_value)
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();
    assert_eq!(body_str, format!("id:{IDENTITY}"));
}

// 3. Expired Bearer JWT → 401 with ErrorResponse JSON.
#[tokio::test]
async fn test_expired_jwt_returns_401() {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct RawClaims {
        sub: String,
        iat: i64,
        exp: i64,
        jti: String,
    }

    let past = chrono::Utc::now().timestamp() - 7200;
    let raw = RawClaims {
        sub: IDENTITY.to_owned(),
        iat: past,
        exp: past + 3600,
        jti: "test-jti".to_owned(),
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &raw,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .unwrap();

    let state = make_state();
    let app = echo_identity_router(state);

    let req = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).expect("body should be JSON");
    assert!(
        json.get("error").is_some(),
        "response must have 'error' field"
    );
    assert!(
        json.get("code").is_some(),
        "response must have 'code' field"
    );
}

// 4. Malformed Bearer JWT → 401.
#[tokio::test]
async fn test_malformed_jwt_returns_401() {
    let state = make_state();
    let app = echo_identity_router(state);

    let req = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "Bearer not.a.valid.token")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// 5. Bearer with wrong secret → 401.
#[tokio::test]
async fn test_wrong_secret_returns_401() {
    let token = encode_jwt(IDENTITY, "wrong-secret-key-totally-different!!", 3600)
        .expect("encode should succeed");

    let state = make_state();
    let app = echo_identity_router(state);

    let req = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
