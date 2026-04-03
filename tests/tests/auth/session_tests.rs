//! Tests for session creation, validation, and the Heka KratosClient integration.

use reqwest::StatusCode;

use super::helpers::{KratosTestClient, TEST_PASSWORD, random_email, random_nyx_id, register_user_password};
use crate::require_stack;

#[tokio::test]
async fn session_is_valid_after_registration() {
    require_stack!();

    let k      = KratosTestClient::new();
    let email  = random_email();
    let nyx_id = random_nyx_id();

    let (session_token, _) = register_user_password(&email, &nyx_id, TEST_PASSWORD).await;

    let (status, body) = k.whoami(&session_token).await;

    assert_eq!(status, StatusCode::OK, "Session should be valid. Body: {body}");
    assert_eq!(body["active"].as_bool(), Some(true), "Session should be active");

    // Identity should include our traits
    let traits = &body["identity"]["traits"];
    assert_eq!(traits["email"].as_str(), Some(email.as_str()));
    assert_eq!(traits["nyx_id"].as_str(), Some(nyx_id.as_str()));
}

#[tokio::test]
async fn session_is_valid_after_login() {
    require_stack!();

    let k      = KratosTestClient::new();
    let email  = random_email();
    let nyx_id = random_nyx_id();

    // Register
    register_user_password(&email, &nyx_id, TEST_PASSWORD).await;

    // Login to get a fresh session
    let flow    = k.init_login_flow().await;
    let flow_id = flow["id"].as_str().unwrap().to_owned();

    let (login_status, login_body) =
        k.submit_login_password(&flow_id, &email, TEST_PASSWORD).await;

    assert_eq!(login_status, StatusCode::OK);

    let session_token = login_body["session_token"]
        .as_str()
        .expect("no session_token");

    // Validate the session
    let (status, body) = k.whoami(session_token).await;

    assert_eq!(status, StatusCode::OK, "Login session should be valid. Body: {body}");
    assert_eq!(body["active"].as_bool(), Some(true));
}

#[tokio::test]
async fn invalid_session_token_is_rejected() {
    require_stack!();

    let k = KratosTestClient::new();

    let (status, _body) = k.whoami("this-is-not-a-real-session-token").await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "Invalid session token should return 401"
    );
}

#[tokio::test]
async fn empty_session_token_is_rejected() {
    require_stack!();

    let k = KratosTestClient::new();
    let (status, _) = k.whoami("").await;

    assert!(
        status == StatusCode::UNAUTHORIZED || status == StatusCode::BAD_REQUEST,
        "Empty session token should be rejected"
    );
}

/// Test that the Heka `KratosClient` (our Rust wrapper) correctly parses
/// a real Kratos session and returns a well-formed `NyxIdentity`.
#[tokio::test]
async fn heka_client_parses_real_kratos_session() {
    require_stack!();

    let email  = random_email();
    let nyx_id = random_nyx_id();

    let (session_token, _) =
        register_user_password(&email, &nyx_id, TEST_PASSWORD).await;

    // Use the real KratosClient (Heka crate) — not a mock
    let kratos_client = heka::KratosClient::new(
        super::helpers::kratos_public(),
        super::helpers::kratos_admin(),
    );

    let identity = kratos_client
        .validate_session(&session_token)
        .await
        .expect("KratosClient should parse real session without error");

    // Verify the identity is correctly populated
    assert_eq!(
        identity.email.as_deref(),
        Some(email.as_str()),
        "KratosClient should return the correct email"
    );
    assert_eq!(
        identity.nyx_id.as_deref(),
        Some(nyx_id.as_str()),
        "KratosClient should return the correct nyx_id"
    );
    assert!(
        !identity.id.to_string().is_empty(),
        "KratosClient should return a valid UUID identity ID"
    );
}

#[tokio::test]
async fn heka_client_rejects_invalid_session() {
    require_stack!();

    let kratos_client = heka::KratosClient::new(
        super::helpers::kratos_public(),
        super::helpers::kratos_admin(),
    );

    let result = kratos_client.validate_session("bad-session-token").await;

    assert!(
        result.is_err(),
        "KratosClient should return Err for invalid session"
    );
}
