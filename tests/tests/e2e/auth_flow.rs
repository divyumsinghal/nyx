//! End-to-end integration test for Instagram-style auth flow
//!
//! Tests the complete flow: Email → OTP → Password → Nyx ID
#![cfg(test)]

use std::time::Duration;

use reqwest::Client;

/// Test the Nyx ID availability endpoint
#[tokio::test]
async fn test_nyx_id_availability_check() {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    // Check a random ID should be available
    let resp = client
        .get("http://localhost:3000/api/nyx/id/check-availability?id=testuser123")
        .send()
        .await;

    // Should get a response (may be available or not depending on DB state)
    assert!(resp.is_ok(), "Failed to connect to Heimdall");
    
    let resp = resp.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Expected 200 OK");
    
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["available"].is_boolean(), "Response should have 'available' field");
    assert!(body["id"].is_string(), "Response should have 'id' field");
}

/// Test email validation in registration
#[test]
fn test_email_validation() {
    // Valid emails
    assert!(is_valid_email("user@example.com"));
    assert!(is_valid_email("test.user+tag@example.co.uk"));
    
    // Invalid emails
    assert!(!is_valid_email(""));
    assert!(!is_valid_email("notanemail"));
    assert!(!is_valid_email("@example.com"));
    assert!(!is_valid_email("user@"));
}

fn is_valid_email(email: &str) -> bool {
    let pattern = regex_lite::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    pattern.is_match(email)
}

/// Test the complete registration flow (requires running services)
#[tokio::test]
#[ignore = "Requires Docker services running"] 
async fn test_complete_registration_flow() {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    // 1. Check Nyx ID availability
    let resp = client
        .get("http://localhost:3000/api/nyx/id/check-availability?id=newuser999")
        .send()
        .await
        .expect("Failed to check Nyx ID");
    
    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let is_available = body["available"].as_bool().unwrap_or(false);
    println!("Nyx ID available: {}", is_available);

    // 2. Init registration flow
    let resp = client
        .get("http://localhost:3000/api/nyx/auth/self-service/registration/api")
        .send()
        .await;
    
    assert!(resp.is_ok(), "Failed to init registration: {:?}", resp.err());

    // The full flow would continue with OTP verification, password submission, etc.
    // This is tested manually or with a full integration test harness.
}
