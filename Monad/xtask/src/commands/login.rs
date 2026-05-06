//! Login to an existing Nyx account via interactive CLI.
//!
//! Flow:
//!   1. Collect identifier (email or nyx_id) + password
//!   2. Init Kratos password login flow
//!   3. Submit credentials — Kratos returns a session token on success
//!   4. Exchange session token for a Nyx JWT via Heimdall
#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use serde_json::json;

use crate::auth::KratosClient;
use crate::commands::input::{prompt_line, prompt_secret};

pub async fn run() -> Result<()> {
    let client = KratosClient::new()?;

    // Collect credentials.
    let identifier = prompt_line("Email or Nyx ID: ")?;
    if identifier.is_empty() {
        anyhow::bail!("Identifier cannot be empty.");
    }
    let password = prompt_secret("Password: ")?;
    if password.is_empty() {
        anyhow::bail!("Password cannot be empty.");
    }

    // Initialise a fresh login flow.
    let flow = client
        .init_login_flow()
        .await
        .context("could not start login flow — is the stack running? (just auth-up)")?;
    let flow_id = flow["id"].as_str().context("login flow missing id")?;

    // Submit once — no client-side retries.
    // Kratos tracks failures globally; retrying here adds noise to audit logs
    // and weakens the brute-force signal.
    let body = client
        .submit_login(
            flow_id,
            "password",
            &identifier,
            json!({ "password": password }),
        )
        .await
        .map_err(|err| {
            anyhow::anyhow!("Login failed: {err}\nCheck your credentials and try again.")
        })?;

    let session_token = body["session_token"]
        .as_str()
        .context("login response missing session_token")?;

    // Exchange the Kratos session token for a Nyx JWT.
    match client.exchange_session_for_jwt(session_token).await {
        Ok(_token) => {
            println!("Access token obtained (not printed for security).");
        }
        Err(err) => {
            eprintln!("Warning: token exchange failed: {err}");
        }
    }

    // Show only user-facing fields — the internal Kratos identity UUID is never
    // displayed (it's an implementation detail, not a user identifier).
    let nyx_id = body["identity"]["traits"]["nyx_id"]
        .as_str()
        .unwrap_or("unknown");
    let email = body["identity"]["traits"]["email"]
        .as_str()
        .unwrap_or("");

    println!("\nWelcome back, @{nyx_id}!");
    if !email.is_empty() {
        println!("  Email:    {email}");
    }
    println!("  Username: @{nyx_id}");

    Ok(())
}
