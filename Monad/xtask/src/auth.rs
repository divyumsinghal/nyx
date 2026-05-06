//! Thin Kratos/Heimdall HTTP client for xtask commands.
#![warn(clippy::pedantic)]

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode, Url};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct KratosClient {
    http: Client,
    url: String,
}

impl KratosClient {
    pub fn new() -> Result<Self> {
        let url = std::env::var("HEIMDALL_URL")
            .context("HEIMDALL_URL environment variable is required")?
            .trim_end_matches('/')
            .to_owned();

        Url::parse(&url).context("HEIMDALL_URL must be a valid URL")?;

        let mut client_builder = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .pool_idle_timeout(Duration::from_secs(60));

        // If CADDY_CA_CERT points to a PEM file (Caddy's local CA root), add it as a
        // trusted root so rustls validates the self-signed cert without any bypass.
        if let Ok(cert_path) = std::env::var("CADDY_CA_CERT") {
            let pem = std::fs::read(&cert_path)
                .with_context(|| format!("failed to read CADDY_CA_CERT at {cert_path}"))?;
            let cert = reqwest::Certificate::from_pem(&pem)
                .context("failed to parse CADDY_CA_CERT as PEM")?;
            client_builder = client_builder.add_root_certificate(cert);
        }

        let http = client_builder
            .build()
            .context("failed to build reqwest::Client")?;

        Ok(Self { http, url })
    }

    // ── Nyx ID ───────────────────────────────────────────────────────────────

    pub async fn check_nyx_id_available(&self, nyx_id: &str) -> Result<bool> {
        let resp = self
            .http
            .post(format!("{}/api/nyx/id/check-availability", self.url))
            .json(&json!({ "id": nyx_id }))
            .send()
            .await;

        match resp {
            Ok(r) if r.status() == StatusCode::OK => {
                let body: Value = r.json().await?;
                body["available"]
                    .as_bool()
                    .context("check-availability response missing boolean `available`")
            }
            Ok(r) => anyhow::bail!("check nyx_id failed: {}", r.status()),
            Err(e) => Err(e.into()),
        }
    }

    // ── Registration ─────────────────────────────────────────────────────────

    /// Initialise a new Kratos registration flow.
    pub async fn init_registration_flow(&self) -> Result<Value> {
        let resp = self
            .http
            .get(format!(
                "{}/api/nyx/auth/self-service/registration/api",
                self.url
            ))
            .send()
            .await
            .context("init registration flow")?;

        if resp.status() != StatusCode::OK {
            anyhow::bail!("init registration failed: {}", resp.status());
        }
        resp.json().await.context("parse registration flow")
    }

    /// Send email + nyx_id via the code method to trigger the OTP email.
    ///
    /// Kratos returns 422 (UNPROCESSABLE_ENTITY) here — that is expected and
    /// means the OTP has been dispatched and we should collect the code.
    pub async fn submit_registration_code_init(
        &self,
        flow_id: &str,
        email: &str,
        nyx_id: &str,
    ) -> Result<Value> {
        let body = json!({
            "method": "code",
            "traits": {
                "email": email,
                "nyx_id": nyx_id,
            }
        });

        let resp = self
            .http
            .post(format!(
                "{}/api/nyx/auth/self-service/registration?flow={flow_id}",
                self.url
            ))
            .json(&body)
            .send()
            .await
            .context("submit registration code init")?;

        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);

        // 400 or 422 means OTP dispatched — Kratos is waiting for the code.
        // Kratos v1.3.1 returns 400 (not 422) when transitioning to sent_email state.
        if status.is_success()
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::UNPROCESSABLE_ENTITY
        {
            Ok(body)
        } else {
            anyhow::bail!(parse_kratos_errors(&body))
        }
    }

    /// Submit the OTP code to complete registration. Returns the response body
    /// which contains `session_token` and `identity` on success.
    pub async fn submit_registration_code_verify(
        &self,
        flow_id: &str,
        code: &str,
        email: &str,
        nyx_id: &str,
    ) -> Result<Value> {
        let body = json!({
            "method": "code",
            "code": code,
            "traits": {
                "email": email,
                "nyx_id": nyx_id,
            }
        });

        let resp = self
            .http
            .post(format!(
                "{}/api/nyx/auth/self-service/registration?flow={flow_id}",
                self.url
            ))
            .json(&body)
            .send()
            .await
            .context("submit registration code verify")?;

        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);

        if status.is_success() {
            Ok(body)
        } else {
            anyhow::bail!(parse_kratos_errors(&body))
        }
    }

    // ── Settings (post-registration password setup) ───────────────────────────

    /// Initialise a Kratos settings flow using an existing session token.
    ///
    /// The fresh session returned by OTP registration is in privileged mode, so
    /// the password can be set immediately without re-authentication.
    pub async fn init_settings_flow(&self, session_token: &str) -> Result<Value> {
        let resp = self
            .http
            .get(format!(
                "{}/api/nyx/auth/self-service/settings/api",
                self.url
            ))
            .header("X-Session-Token", session_token)
            .send()
            .await
            .context("init settings flow")?;

        if !resp.status().is_success() {
            anyhow::bail!("init settings flow failed: {}", resp.status());
        }
        resp.json().await.context("parse settings flow")
    }

    /// Set a password via the Kratos settings flow.
    pub async fn submit_settings_password(
        &self,
        flow_id: &str,
        session_token: &str,
        password: &str,
    ) -> Result<()> {
        let body = json!({ "method": "password", "password": password });

        let resp = self
            .http
            .post(format!(
                "{}/api/nyx/auth/self-service/settings?flow={flow_id}",
                self.url
            ))
            .header("X-Session-Token", session_token)
            .json(&body)
            .send()
            .await
            .context("submit settings password")?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body: Value = resp.json().await.unwrap_or(Value::Null);
            anyhow::bail!("set password failed: {}", parse_kratos_errors(&body))
        }
    }

    // ── Login ─────────────────────────────────────────────────────────────────

    /// Initialise a Kratos login flow.
    pub async fn init_login_flow(&self) -> Result<Value> {
        let resp = self
            .http
            .get(format!(
                "{}/api/nyx/auth/self-service/login/api",
                self.url
            ))
            .send()
            .await
            .context("init login flow")?;

        if resp.status() != StatusCode::OK {
            anyhow::bail!("init login failed: {}", resp.status());
        }
        resp.json().await.context("parse login flow")
    }

    /// Submit login credentials. Returns the full Kratos response body which
    /// includes `session_token` and `identity` on success.
    pub async fn submit_login(
        &self,
        flow_id: &str,
        method: &str,
        identifier: &str,
        extra: Value,
    ) -> Result<Value> {
        let mut body = serde_json::Map::new();
        body.insert("method".to_string(), Value::String(method.to_string()));
        body.insert(
            "identifier".to_string(),
            Value::String(identifier.to_string()),
        );

        if let Some(obj) = extra.as_object() {
            for (k, v) in obj {
                body.insert(k.clone(), v.clone());
            }
        }

        let resp = self
            .http
            .post(format!(
                "{}/api/nyx/auth/self-service/login?flow={flow_id}",
                self.url
            ))
            .json(&body)
            .send()
            .await
            .context("submit login")?;

        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);

        if status == StatusCode::OK {
            Ok(body)
        } else {
            anyhow::bail!("{}", parse_kratos_errors(&body))
        }
    }

    // ── Token exchange ─────────────────────────────────────────────────────────

    /// Exchange a Kratos session token for a Nyx JWT.
    ///
    /// Calls `POST /api/nyx/auth/token` on Heimdall, which validates the
    /// Kratos session and issues a signed JWT for use with protected endpoints.
    /// Returns the raw access token string.
    pub async fn exchange_session_for_jwt(&self, session_token: &str) -> Result<String> {
        let resp = self
            .http
            .post(format!("{}/api/nyx/auth/token", self.url))
            .json(&json!({ "session_token": session_token }))
            .send()
            .await
            .context("exchange session for JWT")?;

        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);

        if status.is_success() {
            body["access_token"]
                .as_str()
                .map(str::to_owned)
                .context("token exchange response missing `access_token`")
        } else {
            anyhow::bail!(
                "token exchange failed ({}): {}",
                status,
                body["error"].as_str().unwrap_or("unknown error")
            )
        }
    }

    // ── Password breach check ─────────────────────────────────────────────────

    /// Check whether a password has appeared in known data breaches via the
    /// HaveIBeenPwned k-anonymity API. Returns `Ok(true)` if breached.
    ///
    /// Network errors return `Err` — callers should treat this as advisory and
    /// fail open (warn, then proceed).
    pub async fn password_is_breached(&self, password: &str) -> Result<bool> {
        let hash = Sha1::digest(password.as_bytes());
        let hash_hex = format!("{hash:x}").to_uppercase();
        let (prefix, suffix) = hash_hex.split_at(5);

        let response = self
            .http
            .get(format!("https://api.pwnedpasswords.com/range/{prefix}"))
            .header("Add-Padding", "true")
            .send()
            .await
            .context("query breached password database")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "breached-password check returned {}",
                response.status()
            );
        }

        let body = response
            .text()
            .await
            .context("read breached-password response")?;
        Ok(body.lines().any(|line| {
            line.split_once(':')
                .is_some_and(|(candidate, _)| candidate.trim() == suffix)
        }))
    }
}

impl Default for KratosClient {
    fn default() -> Self {
        Self::new().expect("Failed to create KratosClient from environment")
    }
}

/// Parse a Kratos error response body into a human-readable message.
pub fn parse_kratos_errors(body: &Value) -> String {
    let mut messages = Vec::new();

    // Field-level validation errors (UI nodes)
    if let Some(nodes) = body["ui"]["nodes"].as_array() {
        for node in nodes {
            if let Some(name) = node["attributes"]["name"].as_str() {
                if let Some(node_msgs) = node["messages"].as_array() {
                    for msg in node_msgs {
                        if let Some(text) = msg["text"].as_str() {
                            messages.push(format!("{name}: {text}"));
                        }
                    }
                }
            }
        }
    }

    // Flow-level messages
    if let Some(flow_msgs) = body["ui"]["messages"].as_array() {
        for msg in flow_msgs {
            if let Some(text) = msg["text"].as_str() {
                messages.push(text.to_string());
            }
        }
    }

    // Top-level error
    if let Some(err) = body["error"]["message"].as_str() {
        messages.push(err.to_string());
    }

    if messages.is_empty() {
        "Unexpected Kratos error response".to_string()
    } else {
        messages.join("\n")
    }
}
