//! Login to an existing Nyx account via interactive CLI.
//! Thin wrapper - all logic is in the backend (Kratos).
#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use serde_json::json;
use tokio::time::{sleep, Duration};

use crate::auth::KratosClient;
use crate::commands::input::{prompt_line, prompt_secret};

pub async fn run() -> Result<()> {
    let client = KratosClient::new()?;

    // Prompt for credentials
    let identifier = prompt_line("Email or Nyx ID: ")?;
    let password = prompt_secret("Password: ")?;

    // Init login flow
    let flow = client.init_login_flow().await?;
    let flow_id = flow["id"].as_str().context("no flow id")?;

    // Submit login with bounded retries and a small backoff.
    let mut result = None;
    for attempt in 1..=5 {
        match client
            .submit_login(
                flow_id,
                "password",
                &identifier,
                json!({ "password": password }),
            )
            .await
        {
            Ok(body) => {
                result = Some(Ok(body));
                break;
            }
            Err(err) if attempt < 5 => {
                eprintln!("Login failed (attempt {attempt}/5): {err}");
                sleep(Duration::from_millis(250 * attempt as u64)).await;
            }
            Err(err) => {
                result = Some(Err(err));
            }
        }
    }

    let result = match result {
        Some(outcome) => outcome,
        None => anyhow::bail!("login did not complete"),
    };

    match result {
        Ok(body) => {
            println!("\nLogin successful!");
            if body.get("session_token").is_some() {
                println!("Session established (token is intentionally not printed).");
            }
            if let Some(id) = body["identity"]["id"].as_str() {
                println!("Identity ID: {id}");
            }
        }
        Err(e) => {
            eprintln!("\nError: {e}");
            return Err(e);
        }
    }

    Ok(())
}
