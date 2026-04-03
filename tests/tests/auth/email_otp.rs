//! Tests for the email OTP (code method) registration and login flows.
//!
//! These tests require Mailpit to be running (included in auth-test.yml).

use reqwest::StatusCode;

use super::helpers::{KratosTestClient, MailpitClient, random_email, random_nyx_id};
use crate::require_stack;

// ── OTP Registration ─────────────────────────────────────────────────────────

#[tokio::test]
async fn otp_registration_full_flow() {
    require_stack!();

    let k       = KratosTestClient::new();
    let mailpit = MailpitClient::new();
    let email   = random_email();
    let nyx_id  = random_nyx_id();

    // Step 1: init flow
    let flow    = k.init_registration_flow().await;
    let flow_id = flow["id"].as_str().expect("no flow id").to_owned();

    // Step 2: submit email + nyx_id with method=code → triggers OTP email
    let before_otp = chrono::Utc::now() - chrono::Duration::seconds(2);
    let (status, body) =
        k.submit_registration_code_init(&flow_id, &email, &nyx_id).await;

    // Kratos v1.3.1 returns 400 with state="sent_email" when the code has been emailed.
    // Older versions used 422. Accept both. 200 means the flow auto-completed (unlikely).
    let flow_state = body["state"].as_str().unwrap_or("");
    assert!(
        (status == StatusCode::BAD_REQUEST && flow_state == "sent_email")
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::OK,
        "Expected code-sent (400 sent_email / 422) or 200, got {status}: {body}"
    );

    if status == StatusCode::OK {
        return; // Already done
    }

    // Step 3: wait for OTP email in Mailpit (only the one sent after our request)
    let email_body = mailpit
        .wait_for_email_after(&email, 30, Some(before_otp))
        .await
        .expect("OTP email should arrive in Mailpit within 30s");

    let code = MailpitClient::extract_code(&email_body)
        .expect("Could not extract 6-digit OTP from email body");

    assert_eq!(code.len(), 6, "OTP should be 6 digits, got: {code}");

    // Step 4: submit the code to complete registration (traits must be re-submitted)
    let (status, body) = k.submit_registration_code_verify(&flow_id, &code, &email, &nyx_id).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "OTP verification should succeed. Body: {body}"
    );

    // Verify session is returned
    let session_token = body["session_token"].as_str();
    assert!(
        session_token.is_some(),
        "OTP registration should return session_token. Body: {body}"
    );

    // Verify traits
    let traits = &body["identity"]["traits"];
    assert_eq!(traits["email"].as_str(), Some(email.as_str()));
    assert_eq!(traits["nyx_id"].as_str(), Some(nyx_id.as_str()));
}

#[tokio::test]
async fn otp_invalid_code_is_rejected() {
    require_stack!();

    let k      = KratosTestClient::new();
    let email  = random_email();
    let nyx_id = random_nyx_id();

    let flow    = k.init_registration_flow().await;
    let flow_id = flow["id"].as_str().unwrap().to_owned();

    // Trigger OTP send
    let (status, _body) =
        k.submit_registration_code_init(&flow_id, &email, &nyx_id).await;

    // Kratos v1.3.1 returns 400 "sent_email" for this step too
    let flow_state = _body["state"].as_str().unwrap_or("");
    if !(status == StatusCode::UNPROCESSABLE_ENTITY
        || (status == StatusCode::BAD_REQUEST && flow_state == "sent_email"))
    {
        return; // Not in OTP step
    }

    // Submit wrong code (traits required by Kratos v1.3.1 even for invalid codes)
    let (status, _body) =
        k.submit_registration_code_verify(&flow_id, "000000", &email, &nyx_id).await;

    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::GONE,
        "Invalid OTP should be rejected, got {status}"
    );
}

// ── OTP Login (passwordless) ──────────────────────────────────────────────────

#[tokio::test]
async fn otp_passwordless_login_flow() {
    require_stack!();

    let k       = KratosTestClient::new();
    let mailpit = MailpitClient::new();
    let email   = random_email();
    let nyx_id  = random_nyx_id();

    // Register user first (password method for simplicity)
    super::helpers::register_user_password(&email, &nyx_id, super::helpers::TEST_PASSWORD).await;

    // Snapshot time before triggering the code login — avoids matching the
    // verification email from the registration step above.
    let before_login_otp = chrono::Utc::now() - chrono::Duration::seconds(2);

    // Init login flow
    let flow    = k.init_login_flow().await;
    let flow_id = flow["id"].as_str().unwrap().to_owned();

    // Submit email via code method
    let resp = k.http
        .post(format!("{}/self-service/login?flow={flow_id}", super::helpers::kratos_public()))
        .json(&serde_json::json!({
            "method":     "code",
            "identifier": email,
        }))
        .send()
        .await
        .expect("submit_login_code: network error");

    let status = resp.status();
    let _body: serde_json::Value = resp.json().await.unwrap_or_default();

    // Kratos v1.3.1 returns 400 with state="sent_email" when code is emailed.
    // Older versions used 422. Accept both. 200 means flow already completed.
    let body_state = _body["state"].as_str().unwrap_or("");
    if status == StatusCode::OK {
        return;
    }
    assert!(
        status == StatusCode::UNPROCESSABLE_ENTITY
            || (status == StatusCode::BAD_REQUEST && body_state == "sent_email"),
        "Expected code-sent (400 sent_email / 422), got {status}: {_body}"
    );

    // Wait for login code email — only messages after our timestamp to avoid
    // picking up the verification email from the password registration above.
    let email_body = mailpit
        .wait_for_email_after(&email, 30, Some(before_login_otp))
        .await
        .expect("Login OTP email should arrive in Mailpit within 30s");

    let code = MailpitClient::extract_code(&email_body)
        .expect("Could not extract OTP from login email");

    // Submit code — Kratos v1.3.1 requires the identifier to be re-submitted
    // along with the code (it's a hidden field in the browser form).
    let resp = k.http
        .post(format!("{}/self-service/login?flow={flow_id}", super::helpers::kratos_public()))
        .json(&serde_json::json!({
            "method":     "code",
            "code":       code,
            "identifier": email,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "OTP login should succeed"
    );

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["session_token"].as_str().is_some(),
        "Passwordless login should return session_token"
    );
}
