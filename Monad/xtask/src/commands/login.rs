//! Login to an existing Nyx account via interactive CLI.
//! Thin wrapper - all logic is in the backend (Kratos).
#![warn(clippy::pedantic)]

use std::io::{self, Write};

use anyhow::{Context, Result};
use serde_json::json;

use crate::auth::KratosClient;

pub async fn run() -> Result<()> {
    let client = KratosClient::new();

    // Prompt for credentials
    print!("Email or Nyx ID: ");
    io::stdout().flush()?;
    let mut identifier = String::new();
    io::stdin().read_line(&mut identifier)?;
    let identifier = identifier.trim();

    print!("Password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    // Init login flow
    let flow = client.init_login_flow().await?;
    let flow_id = flow["id"].as_str().context("no flow id")?;

    // Submit login
    let result = client.submit_login(
        flow_id,
        "password",
        identifier,
        json!({ "password": password }),
    ).await;

    match result {
        Ok(body) => {
            println!("\nLogin successful!");
            if let Some(token) = body["session_token"].as_str() {
                println!("Session token: {token}");
            }
            if let Some(id) = body["identity"]["id"].as_str() {
                println!("Identity ID: {id}");
            }
        }
        Err(e) => {
            eprintln!("\nError: {e}");
        }
    }

    Ok(())
}
