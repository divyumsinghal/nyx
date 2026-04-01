use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    middleware,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower::ServiceExt;
use validator::Validate;

use nyx_api::{
    extract::ValidatedJson,
    middleware::request_id::{request_id, REQUEST_ID_HEADER},
    openapi::build_openapi,
    response::ApiResponse,
};

#[derive(Debug, Deserialize, Validate)]
struct CreateStoryPayload {
    #[validate(length(min = 1, max = 280))]
    text: String,
}

#[derive(Debug, Serialize)]
struct StoryDto {
    id: String,
}

#[test]
fn payload_validation_contract_rejects_empty_text() {
    let payload = CreateStoryPayload {
        text: String::new(),
    };
    assert!(payload.validate().is_err());
}

#[tokio::test]
async fn validated_json_rejects_invalid_payload_with_422() {
    async fn handler(
        ValidatedJson(_payload): ValidatedJson<CreateStoryPayload>,
    ) -> impl IntoResponse {
        StatusCode::CREATED
    }

    let app = Router::new().route("/stories", post(handler));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/stories")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "text": "" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn request_id_middleware_preserves_header() {
    async fn handler() -> impl IntoResponse {
        StatusCode::OK
    }

    let app = Router::new()
        .route("/ping", post(handler))
        .layer(middleware::from_fn(request_id));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ping")
                .header(REQUEST_ID_HEADER.clone(), "req-fixed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(REQUEST_ID_HEADER.clone()).unwrap(),
        &"req-fixed"
    );
}

#[tokio::test]
async fn request_id_middleware_generates_when_missing() {
    async fn handler() -> impl IntoResponse {
        StatusCode::OK
    }

    let app = Router::new()
        .route("/ping", post(handler))
        .layer(middleware::from_fn(request_id));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ping")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let rid = response
        .headers()
        .get(REQUEST_ID_HEADER.clone())
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(!rid.is_empty());
}

#[tokio::test]
async fn api_response_envelope_serializes() {
    let response = ApiResponse::paginated(
        vec![StoryDto {
            id: "story-1".to_string(),
        }],
        Some("cursor-1".to_string()),
        true,
    )
    .into_response();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(json["data"][0]["id"], "story-1");
    assert_eq!(json["pagination"]["next_cursor"], "cursor-1");
    assert_eq!(json["pagination"]["has_more"], true);
}

#[test]
fn openapi_builder_sets_info() {
    let doc = build_openapi("uzume-stories", "1.0.0");
    assert_eq!(doc.info.title, "uzume-stories");
    assert_eq!(doc.info.version, "1.0.0");
}
