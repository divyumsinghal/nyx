//! Create a new Nyx account via interactive CLI.
//! Instagram-style flow: Email → OTP → Password → Nyx ID
#![warn(clippy::pedantic)]

use std::io::{self, Write};

use anyhow::{Context, Result};
use serde_json::json;

use crate::auth::KratosClient;

/// Validate email format
fn is_valid_email(email: &str) -> bool {
    let pattern = regex_lite::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    pattern.is_match(email)
}

pub async fn run() -> Result<()> {
    let client = KratosClient::new();

    // Step 1: Email with validation
    let email = loop {
        print!("Email: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            eprintln!("Email cannot be empty");
            continue;
        }

        if !is_valid_email(input) {
            eprintln!("Invalid email format");
            continue;
        }

        break input.to_string();
    };

    // Step 2: Init registration flow (triggers OTP email via Kratos)
    let flow = client.init_registration_flow().await?;
    let flow_id = flow["id"].as_str().context("no flow id")?;

    // Step 3: OTP Code
    print!("OTP Code: ");
    io::stdout().flush()?;
    let mut otp = String::new();
    io::stdin().read_line(&mut otp)?;
    let otp = otp.trim();

    // Step 4: Password
    print!("Password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    // Step 5: Nyx ID with availability check
    let nyx_id = loop {
        print!("Nyx ID: ");
        io::stdout().flush()?;
        let mut id = String::new();
        io::stdin().read_line(&mut id)?;
        let id = id.trim();

        if id.is_empty() {
            eprintln!("Nyx ID cannot be empty");
            continue;
        }

        match client.check_nyx_id_available(id).await {
            Ok(true) => break id.to_string(),
            Ok(false) => eprintln!("Nyx ID '{}' is already taken", id),
            Err(e) => eprintln!("Error checking availability: {}", e),
        }
    };

    // Submit registration
    let result = client.submit_registration(
        flow_id,
        "password",
        json!({ "email": email }),
        json!({
            "password": password,
            "traits.nyx_id": nyx_id,
            "traits.otp_code": otp
        }),
    ).await;

    match result {
        Ok(body) => {
            println!("\nAccount created!");
            if let Some(token) = body["session_token"].as_str() {
                println!("Session token: {token}");
            }
            if let Some(id) = body["identity"]["id"].as_str() {
                println!("Identity ID: {id}");
            }
        }
        Err(e) => {
            eprintln!("\n{e}");
        }
    }

    Ok(())
}
