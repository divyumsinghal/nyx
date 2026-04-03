//! Create a new Nyx account via interactive CLI.
//! Instagram-style flow: Email → OTP → Password → Nyx ID
#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use validator::ValidateEmail;

use crate::auth::KratosClient;
use crate::commands::input::{prompt_line, prompt_secret};
use heka::validate_nyx_id;

fn normalize_email(email: &str) -> String {
    let email = email.trim().to_lowercase();
    let Some((local, domain)) = email.split_once('@') else {
        return email;
    };

    if domain == "gmail.com" || domain == "googlemail.com" {
        let normalized_local = local.split('+').next().unwrap_or(local).replace('.', "");
        return format!("{normalized_local}@gmail.com");
    }

    email
}

fn password_is_strong(password: &str, email: &str, nyx_id: &str) -> bool {
    zxcvbn::zxcvbn(password, &[email, nyx_id]).score() >= zxcvbn::Score::Three
}

pub async fn run() -> Result<()> {
    let client = KratosClient::new()?;

    // Step 1: Email with validation
    let email = loop {
        let input = prompt_line("Email: ")?;

        if input.is_empty() {
            eprintln!("Email cannot be empty");
            continue;
        }

        if !input.validate_email() {
            eprintln!("Invalid email format");
            continue;
        }

        break normalize_email(&input);
    };

    // Step 2: Init registration flow (triggers OTP email via Kratos)
    let flow = client.init_registration_flow().await?;
    let flow_id = flow["id"].as_str().context("no flow id")?;

    // Step 3: Password
    let password = loop {
        let value = prompt_secret("Password: ")?;
        if value.len() < 12 {
            eprintln!("Password must be at least 12 characters");
            continue;
        }
        match client.password_is_breached(&value).await {
            Ok(true) => {
                eprintln!("Password has appeared in breach data. Choose a different password.");
                continue;
            }
            Ok(false) => {}
            Err(err) => {
                eprintln!("Breached-password check unavailable: {err}");
                continue;
            }
        }
        break value;
    };

    // Step 4: Nyx ID with availability check
    let nyx_id = loop {
        let id = prompt_line("Nyx ID: ")?;

        if id.is_empty() {
            eprintln!("Nyx ID cannot be empty");
            continue;
        }

        if let Err(reason) = validate_nyx_id(&id) {
            eprintln!("Invalid Nyx ID: {reason}");
            continue;
        }

        if !password_is_strong(&password, &email, &id) {
            eprintln!("Password is too weak for this account. Use a longer and less predictable password.");
            continue;
        }

        match client.check_nyx_id_available(&id).await {
            Ok(true) => break id,
            Ok(false) => eprintln!("Nyx ID '{}' is already taken", id),
            Err(e) => eprintln!("Error checking availability: {}", e),
        }
    };

    // Step 5: initialize code-based registration (sends OTP email).
    client
        .submit_registration_code_init(flow_id, &email, &nyx_id)
        .await?;

    // Step 6: OTP Code with bounded retries.
    let mut result = None;
    for attempt in 1..=3 {
        let otp = loop {
            let value = prompt_secret("OTP Code: ")?;
            if value.is_empty() {
                eprintln!("OTP code cannot be empty");
                continue;
            }
            break value;
        };

        match client
            .submit_registration_code_verify(flow_id, &otp, &email, &nyx_id)
            .await
        {
            Ok(body) => {
                result = Some(Ok(body));
                break;
            }
            Err(err) if attempt < 3 => {
                eprintln!("OTP verification failed (attempt {attempt}/3): {err}");
            }
            Err(err) => {
                result = Some(Err(err));
            }
        }
    }

    let result = match result {
        Some(outcome) => outcome,
        None => anyhow::bail!("OTP verification did not complete"),
    };

    match result {
        Ok(body) => {
            println!("\nAccount created!");
            if body.get("session_token").is_some() {
                println!("Session established (token is intentionally not printed).");
            }
            if let Some(id) = body["identity"]["id"].as_str() {
                println!("Identity ID: {id}");
            }
        }
        Err(e) => {
            eprintln!("\n{e}");
            return Err(e);
        }
    }

    Ok(())
}
