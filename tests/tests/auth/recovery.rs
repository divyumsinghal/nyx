//! Tests for the password recovery flow (email OTP).

use reqwest::StatusCode;

use super::helpers::{
    KratosInboxClient, KratosTestClient, TEST_PASSWORD, random_email, random_nyx_id,
    register_user_password,
};
use crate::require_stack;

#[tokio::test]
async fn password_recovery_full_flow() {
    require_stack!();

    let k       = KratosTestClient::new();
    let inbox   = KratosInboxClient::new();
    let email   = random_email();
    let nyx_id  = random_nyx_id();

    // Register user
    register_user_password(&email, &nyx_id, TEST_PASSWORD).await;

    // Record time before triggering the recovery flow so we only pick up
    // messages dispatched during this test (not leftovers from prior runs).
    let before = chrono::Utc::now();

    // Step 1: init recovery flow
    let flow    = k.init_recovery_flow().await;
    let flow_id = flow["id"].as_str().expect("no flow id").to_owned();

    // Step 2: submit email for recovery
    let (status, _body) = k.submit_recovery_email(&flow_id, &email).await;

    // 422 = code sent (needs to be entered), 200 = immediate (unlikely)
    assert!(
        status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::OK,
        "Recovery email submission should return 422 or 200, got {status}"
    );

    if status == StatusCode::OK {
        return; // Handled immediately
    }

    // Step 3: wait for recovery email via Kratos courier admin API
    let email_body = inbox
        .wait_for_email_after(&email, 15, Some(before))
        .await
        .expect("Recovery email should arrive via Kratos courier within 15s");

    let code = KratosInboxClient::extract_code(&email_body)
        .expect("Could not extract 6-digit code from recovery email");

    // Step 4: submit code
    let (status, body) = k.submit_recovery_code(&flow_id, &code).await;

    // Recovery with valid code returns 422 with a settings flow to set new password,
    // OR 200 with a session directly. Both are valid Kratos behaviours.
    assert!(
        status == StatusCode::OK || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Recovery code submission should succeed. Status: {status}, Body: {body}"
    );
}

#[tokio::test]
async fn recovery_for_unknown_email_does_not_leak_existence() {
    require_stack!();

    let k    = KratosTestClient::new();
    let flow = k.init_recovery_flow().await;
    let fid  = flow["id"].as_str().unwrap().to_owned();

    // Submit recovery for an email that was never registered.
    // Kratos should NOT reveal whether the email exists (notify_unknown_recipients: false).
    let (status, _body) = k.submit_recovery_email(&fid, "ghost@nyx.test").await;

    // Returns 422 (code sent — but no email will actually arrive) to prevent enumeration
    assert!(
        status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::OK,
        "Recovery for unknown email should not error differently than known email, got {status}"
    );
}
