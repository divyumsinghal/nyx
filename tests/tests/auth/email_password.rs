//! Tests for email + password registration and login flows.

use reqwest::StatusCode;

use super::helpers::{
    KratosTestClient, TEST_PASSWORD, random_email, random_nyx_id, register_user_password,
};
use crate::require_stack;

// ── Registration ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn email_password_registration_succeeds() {
    require_stack!();

    let k      = KratosTestClient::new();
    let email  = random_email();
    let nyx_id = random_nyx_id();

    // Step 1: init registration flow
    let flow    = k.init_registration_flow().await;
    let flow_id = flow["id"].as_str().expect("no flow id").to_owned();
    assert!(!flow_id.is_empty(), "Flow ID should not be empty");

    // Step 2: submit credentials
    let (status, body) =
        k.submit_registration_password(&flow_id, &email, &nyx_id, TEST_PASSWORD).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Registration should return 200. Response: {body}"
    );

    // Verify session token is returned
    let session_token = body["session_token"].as_str();
    assert!(
        session_token.is_some(),
        "Response should contain session_token. Body: {body}"
    );

    // Verify identity traits are correct
    let traits = &body["identity"]["traits"];
    assert_eq!(
        traits["email"].as_str(),
        Some(email.as_str()),
        "Email trait should match"
    );
    assert_eq!(
        traits["nyx_id"].as_str(),
        Some(nyx_id.as_str()),
        "nyx_id trait should match"
    );
}

#[tokio::test]
async fn registration_creates_session_immediately() {
    require_stack!();

    let k      = KratosTestClient::new();
    let email  = random_email();
    let nyx_id = random_nyx_id();

    let (session_token, _) =
        register_user_password(&email, &nyx_id, TEST_PASSWORD).await;

    // Verify the session is valid
    let (status, body) = k.whoami(&session_token).await;

    assert_eq!(status, StatusCode::OK, "Session should be valid immediately after registration. Body: {body}");
    assert_eq!(
        body["active"].as_bool(),
        Some(true),
        "Session should be active"
    );
}

#[tokio::test]
async fn registration_rejects_weak_password() {
    require_stack!();

    let k      = KratosTestClient::new();
    let flow   = k.init_registration_flow().await;
    let fid    = flow["id"].as_str().unwrap().to_owned();

    let (status, _body) = k
        .submit_registration_password(&fid, &random_email(), &random_nyx_id(), "weak")
        .await;

    // 400 or 422 — Kratos rejects short/weak passwords
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Weak password should be rejected, got {status}"
    );
}

#[tokio::test]
async fn registration_rejects_invalid_nyx_id_format() {
    require_stack!();

    let k   = KratosTestClient::new();
    let flow = k.init_registration_flow().await;
    let fid  = flow["id"].as_str().unwrap().to_owned();

    // Starts with a dot — invalid per schema pattern
    let (status, _body) = k
        .submit_registration_password(&fid, &random_email(), ".invalid_start", TEST_PASSWORD)
        .await;

    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "nyx_id starting with dot should be rejected, got {status}"
    );
}

#[tokio::test]
async fn registration_requires_email_and_nyx_id() {
    require_stack!();

    let k = KratosTestClient::new();

    // Missing nyx_id
    {
        let flow = k.init_registration_flow().await;
        let fid  = flow["id"].as_str().unwrap().to_owned();

        let resp = k.http
            .post(format!("{}/self-service/registration?flow={fid}", super::helpers::kratos_public()))
            .json(&serde_json::json!({
                "method":   "password",
                "password": TEST_PASSWORD,
                "traits":   { "email": random_email() }
            }))
            .send()
            .await
            .unwrap();

        assert!(
            resp.status() == StatusCode::BAD_REQUEST
                || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "Registration without nyx_id should fail"
        );
    }

    // Missing email
    {
        let flow = k.init_registration_flow().await;
        let fid  = flow["id"].as_str().unwrap().to_owned();

        let resp = k.http
            .post(format!("{}/self-service/registration?flow={fid}", super::helpers::kratos_public()))
            .json(&serde_json::json!({
                "method":   "password",
                "password": TEST_PASSWORD,
                "traits":   { "nyx_id": random_nyx_id() }
            }))
            .send()
            .await
            .unwrap();

        assert!(
            resp.status() == StatusCode::BAD_REQUEST
                || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "Registration without email should fail"
        );
    }
}

// ── Login ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn login_with_email_and_password_succeeds() {
    require_stack!();

    let k      = KratosTestClient::new();
    let email  = random_email();
    let nyx_id = random_nyx_id();

    // Register first
    register_user_password(&email, &nyx_id, TEST_PASSWORD).await;

    // Then login
    let flow    = k.init_login_flow().await;
    let flow_id = flow["id"].as_str().unwrap().to_owned();

    let (status, body) =
        k.submit_login_password(&flow_id, &email, TEST_PASSWORD).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Login with email should succeed. Body: {body}"
    );
    assert!(
        body["session_token"].as_str().is_some(),
        "Login response should contain session_token"
    );
}

#[tokio::test]
async fn login_with_nyx_id_and_password_succeeds() {
    require_stack!();

    let k      = KratosTestClient::new();
    let email  = random_email();
    let nyx_id = random_nyx_id();

    register_user_password(&email, &nyx_id, TEST_PASSWORD).await;

    let flow    = k.init_login_flow().await;
    let flow_id = flow["id"].as_str().unwrap().to_owned();

    // Login using nyx_id instead of email
    let (status, body) =
        k.submit_login_password(&flow_id, &nyx_id, TEST_PASSWORD).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Login with nyx_id should succeed. Body: {body}"
    );
    assert!(body["session_token"].as_str().is_some());
}

#[tokio::test]
async fn login_with_wrong_password_fails() {
    require_stack!();

    let k      = KratosTestClient::new();
    let email  = random_email();
    let nyx_id = random_nyx_id();

    register_user_password(&email, &nyx_id, TEST_PASSWORD).await;

    let flow    = k.init_login_flow().await;
    let flow_id = flow["id"].as_str().unwrap().to_owned();

    let (status, _body) =
        k.submit_login_password(&flow_id, &email, "WrongPassword!999").await;

    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Wrong password should fail, got {status}"
    );
}

#[tokio::test]
async fn login_with_nonexistent_email_fails() {
    require_stack!();

    let k    = KratosTestClient::new();
    let flow = k.init_login_flow().await;
    let fid  = flow["id"].as_str().unwrap().to_owned();

    let (status, _body) =
        k.submit_login_password(&fid, "ghost@nyx.test", TEST_PASSWORD).await;

    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Login with unknown email should fail, got {status}"
    );
}
