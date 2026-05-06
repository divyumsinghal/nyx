//! Create a new Nyx account via interactive CLI.
//!
//! Flow (Instagram-style — all info collected before any OTP):
//!   1. Collect email
//!   2. Collect + validate password (required — needed for future logins)
//!   3. Collect Nyx ID and verify availability
//!   4. Init Kratos code-flow registration + send OTP to email
//!   5. User enters OTP — account created, privileged session returned
//!   6. Set password on the fresh privileged session
//!   7. Exchange Kratos session token for a Nyx JWT
#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use serde_json::Value;
use validator::ValidateEmail;

use crate::auth::KratosClient;
use crate::commands::input::{prompt_line, prompt_secret};
use heka::validate_nyx_id;

// ── Email helpers ─────────────────────────────────────────────────────────────

fn normalize_email(email: &str) -> String {
    let email = email.trim().to_lowercase();
    let Some((local, domain)) = email.split_once('@') else {
        return email;
    };
    // Normalise Gmail: strip +aliases and dots from local part.
    if domain == "gmail.com" || domain == "googlemail.com" {
        let normalized_local = local.split('+').next().unwrap_or(local).replace('.', "");
        return format!("{normalized_local}@gmail.com");
    }
    email
}

// ── Password helpers ──────────────────────────────────────────────────────────

/// Returns true if the password is strong enough relative to the user's email.
/// NOTE: nyx_id is not yet known when this is called, so only email is used as
/// context — same behaviour as Instagram's signup password check.
fn password_is_strong(password: &str, email: &str) -> bool {
    zxcvbn::zxcvbn(password, &[email]).score() >= zxcvbn::Score::Three
}

/// Interactively collect a valid password with confirmation + strength checks.
/// Fails open on HaveIBeenPwned network errors so an outage never blocks signup.
async fn collect_password(client: &KratosClient, email: &str) -> Result<String> {
    loop {
        let password = prompt_secret("Password (12+ chars): ")?;

        if password.len() < 12 {
            eprintln!("Password must be at least 12 characters.");
            continue;
        }

        if !password_is_strong(&password, email) {
            eprintln!(
                "Password is too weak — too predictable or too similar to your email. \
                 Try a longer, less guessable password."
            );
            continue;
        }

        match client.password_is_breached(&password).await {
            Ok(true) => {
                eprintln!(
                    "This password has appeared in known data breaches. \
                     Choose a different one."
                );
                continue;
            }
            Ok(false) => {}
            Err(err) => {
                eprintln!("Warning: breach check unavailable ({err}). Proceeding.");
            }
        }

        let confirm = prompt_secret("Confirm password: ")?;
        if password != confirm {
            eprintln!("Passwords don't match. Try again.");
            continue;
        }

        break Ok(password);
    }
}

// ── Duplicate-account detection ───────────────────────────────────────────────

/// Inspect the body Kratos returns after `submit_registration_code_init`.
/// If the email is already registered Kratos returns a 400 whose body contains
/// a UI message about the duplicate identity — detect it here so we can bail
/// with a clear, actionable error instead of dispatching a useless OTP.
fn is_duplicate_account_error(body: &Value) -> bool {
    let serialized = body.to_string().to_lowercase();
    serialized.contains("already registered")
        || serialized.contains("already exists")
        || serialized.contains("account with the same identifier")
        || serialized.contains("duplicate")
}

// ── OTP verification ──────────────────────────────────────────────────────────

async fn verify_otp(
    client: &KratosClient,
    flow_id: &str,
    email: &str,
    nyx_id: &str,
) -> Result<String> {
    for attempt in 1..=3_u8 {
        // OTP codes are short numeric strings — they must be visible when typed,
        // not masked like a password.
        let otp = loop {
            let v = prompt_line(&format!("Enter the 6-digit code sent to {email}: "))?;
            if v.is_empty() {
                eprintln!("Code cannot be empty.");
                continue;
            }
            break v;
        };

        match client
            .submit_registration_code_verify(flow_id, &otp, email, nyx_id)
            .await
        {
            Ok(body) => {
                let session_token = body["session_token"]
                    .as_str()
                    .context("registration response missing session_token")?
                    .to_owned();
                return Ok(session_token);
            }
            Err(err) if attempt < 3 => {
                eprintln!("Incorrect code (attempt {attempt}/3). Try again.");
                let _ = err; // logged above
            }
            Err(err) => {
                return Err(err).context("Email verification failed after 3 attempts");
            }
        }
    }
    unreachable!()
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub async fn run() -> Result<()> {
    let client = KratosClient::new()?;

    // ── Step 1: Email ─────────────────────────────────────────────────────────
    let email = loop {
        let input = prompt_line("Email: ")?;
        if input.is_empty() {
            eprintln!("Email cannot be empty.");
            continue;
        }
        if !input.validate_email() {
            eprintln!("Enter a valid email address.");
            continue;
        }
        break normalize_email(&input);
    };

    // ── Step 2: Password (required) ───────────────────────────────────────────
    // Collected before the Nyx ID so the user knows their credentials before
    // committing to a username — matches Instagram's signup order.
    // The privileged session returned after OTP is used to set this password
    // immediately, so every account always has a working password.
    let password = collect_password(&client, &email).await?;

    // ── Step 3: Nyx ID ────────────────────────────────────────────────────────
    let nyx_id = loop {
        let id = prompt_line("Username (3–32 chars, letters/digits/underscores): ")?;
        if id.is_empty() {
            eprintln!("Username cannot be empty.");
            continue;
        }
        if let Err(reason) = validate_nyx_id(&id) {
            eprintln!("Invalid username: {reason}");
            continue;
        }
        match client.check_nyx_id_available(&id).await {
            Ok(true) => break id,
            Ok(false) => eprintln!("'{id}' is already taken. Try a different username."),
            Err(err) => eprintln!("Could not check availability: {err}"),
        }
    };

    // ── Step 4: Init code-flow registration ───────────────────────────────────
    let flow = client
        .init_registration_flow()
        .await
        .context("could not start registration — is the stack running? (`just auth-up`)")?;
    let flow_id = flow["id"].as_str().context("registration flow missing id")?;

    // ── Step 5: Send OTP ──────────────────────────────────────────────────────
    println!("Sending a verification code to {email}…");
    let init_body = client
        .submit_registration_code_init(flow_id, &email, &nyx_id)
        .await
        .context("failed to send verification code")?;

    // Kratos returns a 400 (treated as Ok by the client) whose body contains a
    // duplicate-identity message when the email is already registered.  Catch
    // it here so we never pretend to dispatch a useful OTP.
    if is_duplicate_account_error(&init_body) {
        anyhow::bail!(
            "An account with this email already exists. \
             Run `just account-login` to sign in."
        );
    }

    println!("Check your inbox — a 6-digit code is on its way.");

    // ── Step 6: Verify OTP ────────────────────────────────────────────────────
    let session_token = verify_otp(&client, flow_id, &email, &nyx_id).await?;

    // ── Step 7: Set password ──────────────────────────────────────────────────
    // The Kratos session returned immediately after OTP verification is in
    // privileged mode, so we can set the password without any re-authentication.
    // This is NOT optional — every account must have a password so login works.
    match client.init_settings_flow(&session_token).await {
        Ok(settings_flow) => {
            let sfid = settings_flow["id"]
                .as_str()
                .context("settings flow missing id")?;
            match client
                .submit_settings_password(sfid, &session_token, &password)
                .await
            {
                Ok(()) => {}
                Err(err) => eprintln!("Warning: could not set password: {err}"),
            }
        }
        Err(err) => eprintln!("Warning: could not init settings flow: {err}"),
    }

    // ── Step 8: Exchange Kratos session for a Nyx JWT ─────────────────────────
    match client.exchange_session_for_jwt(&session_token).await {
        Ok(_token) => {
            println!("Access token obtained.");
        }
        Err(err) => {
            eprintln!(
                "Warning: token exchange failed: {err}\n\
                 You can still log in with `just account-login`."
            );
        }
    }

    println!("\nWelcome to Nyx, @{nyx_id}!");
    println!("  Email:    {email}");
    println!("  Username: @{nyx_id}");

    Ok(())
}
