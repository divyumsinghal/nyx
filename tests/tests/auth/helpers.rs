//! Shared test helpers for auth integration tests.
//!
//! Provides thin clients for Kratos and Mailpit APIs, plus test data generators.
//! All HTTP calls use the real APIs — no mocks anywhere.

use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

// ── Service URLs ─────────────────────────────────────────────────────────────

/// Returns the Kratos public API base URL.
/// Defaults to http://localhost:4433 if KRATOS_PUBLIC_URL is not set.
pub fn kratos_public() -> String {
    std::env::var("KRATOS_PUBLIC_URL")
        .unwrap_or_else(|_| "http://localhost:4433".to_string())
        .trim_end_matches('/')
        .to_owned()
}

/// Returns the Kratos admin API base URL.
/// Defaults to http://localhost:4434 if KRATOS_ADMIN_URL is not set.
pub fn kratos_admin() -> String {
    std::env::var("KRATOS_ADMIN_URL")
        .unwrap_or_else(|_| "http://localhost:4434".to_string())
        .trim_end_matches('/')
        .to_owned()
}

/// Returns the Mailpit REST API base URL.
/// Defaults to http://localhost:8025 if MAILPIT_API_URL is not set.
pub fn mailpit_api() -> String {
    std::env::var("MAILPIT_API_URL")
        .unwrap_or_else(|_| "http://localhost:8025".to_string())
        .trim_end_matches('/')
        .to_owned()
}

// ── Stack availability check ─────────────────────────────────────────────────

/// Check whether the auth stack is reachable.
/// Returns `false` if Kratos is not up — tests should skip gracefully.
pub async fn is_stack_available() -> bool {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();

    client
        .get(format!("{}/health/ready", kratos_public()))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Macro: skip a test with an informative message if the auth stack is not up.
///
/// Usage: `require_stack!()`  at the top of any async test function.
#[macro_export]
macro_rules! require_stack {
    () => {
        if !$crate::helpers::is_stack_available().await {
            eprintln!(
                "⚠  Auth stack not running — skipping test.\n   \
                 Start it with: docker compose -f Prithvi/compose/auth-test.yml up -d --wait"
            );
            return;
        }
    };
}

// ── Test data generators ─────────────────────────────────────────────────────

/// Generate a unique test email address (will be routed to Mailpit).
pub fn random_email() -> String {
    format!("test.{}@nyx.test", Uuid::new_v4().simple())
}

/// Generate a unique, valid Nyx ID.
pub fn random_nyx_id() -> String {
    // Prefix with "u" so it starts with a letter; append 8 hex chars for uniqueness.
    let hex = &Uuid::new_v4().simple().to_string()[..8];
    format!("u_{hex}")
}

/// A strong test password that passes HIBP + length checks.
pub const TEST_PASSWORD: &str = "Nyx!TestP@ssw0rd_2024";

// ── Kratos API client ────────────────────────────────────────────────────────

/// Thin wrapper around the Kratos self-service and admin APIs.
pub struct KratosTestClient {
    pub http:   Client,
    public_url: String,
    admin_url:  String,
}

impl KratosTestClient {
    pub fn new() -> Self {
        Self {
            http:       Client::new(),
            public_url: kratos_public(),
            admin_url:  kratos_admin(),
        }
    }

    // ── Registration ────────────────────────────────────────────────────────

    /// Initialise a new registration flow (API mode).
    pub async fn init_registration_flow(&self) -> Value {
        let resp = self
            .http
            .get(format!("{}/self-service/registration/api", self.public_url))
            .send()
            .await
            .expect("init_registration_flow: network error");

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "init_registration_flow: unexpected status"
        );

        resp.json::<Value>().await.expect("init_registration_flow: parse error")
    }

    /// Submit a password registration (single step).
    ///
    /// Returns the full response body (session token on 200, flow on 422).
    pub async fn submit_registration_password(
        &self,
        flow_id: &str,
        email:   &str,
        nyx_id:  &str,
        password: &str,
    ) -> (StatusCode, Value) {
        let resp = self
            .http
            .post(format!(
                "{}/self-service/registration?flow={flow_id}",
                self.public_url
            ))
            .json(&json!({
                "method":   "password",
                "password": password,
                "traits": {
                    "email":  email,
                    "nyx_id": nyx_id,
                }
            }))
            .send()
            .await
            .expect("submit_registration_password: network error");

        let status = resp.status();
        let body   = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    /// Step 1 of OTP registration: submit email + nyx_id to trigger the code email.
    ///
    /// Returns 422 with the updated flow (which now has a `code` input field).
    pub async fn submit_registration_code_init(
        &self,
        flow_id: &str,
        email:   &str,
        nyx_id:  &str,
    ) -> (StatusCode, Value) {
        let resp = self
            .http
            .post(format!(
                "{}/self-service/registration?flow={flow_id}",
                self.public_url
            ))
            .json(&json!({
                "method": "code",
                "traits": {
                    "email":  email,
                    "nyx_id": nyx_id,
                }
            }))
            .send()
            .await
            .expect("submit_registration_code_init: network error");

        let status = resp.status();
        let body   = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    /// Step 2 of OTP registration: submit the 6-digit code from email.
    ///
    /// Kratos v1.3.1 requires traits to be re-submitted with the code — they
    /// are the hidden form fields the browser would normally include automatically.
    ///
    /// Returns 200 with session token on success.
    pub async fn submit_registration_code_verify(
        &self,
        flow_id: &str,
        code:    &str,
        email:   &str,
        nyx_id:  &str,
    ) -> (StatusCode, Value) {
        let resp = self
            .http
            .post(format!(
                "{}/self-service/registration?flow={flow_id}",
                self.public_url
            ))
            .json(&json!({
                "method": "code",
                "code":   code,
                "traits": {
                    "email":  email,
                    "nyx_id": nyx_id,
                }
            }))
            .send()
            .await
            .expect("submit_registration_code_verify: network error");

        let status = resp.status();
        let body   = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    // ── Login ────────────────────────────────────────────────────────────────

    /// Initialise a new login flow (API mode).
    pub async fn init_login_flow(&self) -> Value {
        let resp = self
            .http
            .get(format!("{}/self-service/login/api", self.public_url))
            .send()
            .await
            .expect("init_login_flow: network error");

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "init_login_flow: unexpected status"
        );

        resp.json::<Value>().await.expect("init_login_flow: parse error")
    }

    /// Submit password login. `identifier` may be email or nyx_id.
    pub async fn submit_login_password(
        &self,
        flow_id:    &str,
        identifier: &str,
        password:   &str,
    ) -> (StatusCode, Value) {
        let resp = self
            .http
            .post(format!(
                "{}/self-service/login?flow={flow_id}",
                self.public_url
            ))
            .json(&json!({
                "method":     "password",
                "identifier": identifier,
                "password":   password,
            }))
            .send()
            .await
            .expect("submit_login_password: network error");

        let status = resp.status();
        let body   = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    // ── Session ──────────────────────────────────────────────────────────────

    /// Validate a session token. Returns the full session body on 200.
    pub async fn whoami(&self, session_token: &str) -> (StatusCode, Value) {
        let resp = self
            .http
            .get(format!("{}/sessions/whoami", self.public_url))
            .header("X-Session-Token", session_token)
            .send()
            .await
            .expect("whoami: network error");

        let status = resp.status();
        let body   = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    // ── Recovery ─────────────────────────────────────────────────────────────

    /// Initialise a password recovery flow (API mode).
    pub async fn init_recovery_flow(&self) -> Value {
        let resp = self
            .http
            .get(format!("{}/self-service/recovery/api", self.public_url))
            .send()
            .await
            .expect("init_recovery_flow: network error");

        assert_eq!(resp.status(), StatusCode::OK);

        resp.json::<Value>().await.expect("init_recovery_flow: parse error")
    }

    /// Step 1: submit email to receive recovery code.
    pub async fn submit_recovery_email(
        &self,
        flow_id: &str,
        email:   &str,
    ) -> (StatusCode, Value) {
        let resp = self
            .http
            .post(format!(
                "{}/self-service/recovery?flow={flow_id}",
                self.public_url
            ))
            .json(&json!({
                "method": "code",
                "email":  email,
            }))
            .send()
            .await
            .expect("submit_recovery_email: network error");

        let status = resp.status();
        let body   = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    /// Step 2: submit recovery code.
    pub async fn submit_recovery_code(
        &self,
        flow_id: &str,
        code:    &str,
    ) -> (StatusCode, Value) {
        let resp = self
            .http
            .post(format!(
                "{}/self-service/recovery?flow={flow_id}",
                self.public_url
            ))
            .json(&json!({
                "method": "code",
                "code":   code,
            }))
            .send()
            .await
            .expect("submit_recovery_code: network error");

        let status = resp.status();
        let body   = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    // ── Admin ────────────────────────────────────────────────────────────────

    /// Fetch an identity by ID via the admin API.
    pub async fn admin_get_identity(&self, identity_id: &str) -> (StatusCode, Value) {
        let resp = self
            .http
            .get(format!("{}/admin/identities/{identity_id}", self.admin_url))
            .send()
            .await
            .expect("admin_get_identity: network error");

        let status = resp.status();
        let body   = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    /// List all identities (admin API, paginated — returns first page).
    #[allow(dead_code)]
    pub async fn admin_list_identities(&self) -> Value {
        self.http
            .get(format!("{}/admin/identities", self.admin_url))
            .send()
            .await
            .expect("admin_list_identities: network error")
            .json::<Value>()
            .await
            .expect("admin_list_identities: parse error")
    }
}

impl Default for KratosTestClient {
    fn default() -> Self {
        Self::new()
    }
}

// ── Mailpit API client ────────────────────────────────────────────────────────

/// Thin wrapper around Mailpit's REST API for fetching test emails.
pub struct MailpitClient {
    http:    Client,
    api_url: String,
}

impl MailpitClient {
    pub fn new() -> Self {
        Self {
            http:    Client::new(),
            api_url: mailpit_api(),
        }
    }

    /// Delete all messages (clean slate before a test).
    pub async fn clear_all(&self) {
        let _ = self
            .http
            .delete(format!("{}/api/v1/messages", self.api_url))
            .send()
            .await;
    }

    /// Wait up to `timeout_secs` for an email addressed to `to_address` that
    /// arrived at or after `after` (UTC).
    ///
    /// Pass `after = None` to match any message regardless of arrival time.
    /// Using a timestamp eliminates races between parallel tests that each call
    /// `clear_all()` and accidentally delete each other's in-flight messages.
    ///
    /// Polls the Mailpit API every 500ms.
    pub async fn wait_for_email_after(
        &self,
        to_address:   &str,
        timeout_secs: u64,
        after:        Option<chrono::DateTime<chrono::Utc>>,
    ) -> Option<String> {
        let deadline = std::time::Instant::now()
            + std::time::Duration::from_secs(timeout_secs);

        while std::time::Instant::now() < deadline {
            if let Some(body) = self.find_email_body_after(to_address, after).await {
                return Some(body);
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        None
    }

    /// Convenience wrapper: no time filter.
    pub async fn wait_for_email(
        &self,
        to_address:   &str,
        timeout_secs: u64,
    ) -> Option<String> {
        self.wait_for_email_after(to_address, timeout_secs, None).await
    }

    /// Extract a 6-digit OTP code from an email body using a regex.
    ///
    /// Kratos sends OTPs as plain 6-digit numbers. This extracts the first
    /// occurrence.
    pub fn extract_code(body: &str) -> Option<String> {
        // Match exactly 6 consecutive digits surrounded by word boundaries or whitespace
        let re = regex_lite::Regex::new(r"(?:^|[\s\-:])(\d{6})(?:$|[\s\.\,])").unwrap();
        re.captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_owned())
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    #[allow(dead_code)]
    async fn find_email_body(&self, to_address: &str) -> Option<String> {
        self.find_email_body_after(to_address, None).await
    }

    /// Find the most-recent email to `to_address` that was created after `after`.
    /// If `after` is None, returns the first matching message regardless of time.
    async fn find_email_body_after(
        &self,
        to_address: &str,
        after: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Option<String> {
        let messages: Value = self
            .http
            .get(format!("{}/api/v1/messages", self.api_url))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        let messages = messages["messages"].as_array()?;

        for msg in messages {
            let to_list = msg["To"].as_array()?;
            let to_matches = to_list.iter().any(|t| {
                t["Address"]
                    .as_str()
                    .map(|a| a.eq_ignore_ascii_case(to_address))
                    .unwrap_or(false)
            });
            if !to_matches {
                continue;
            }

            // Time-filter: skip messages that arrived before `after`
            if let Some(after_ts) = after {
                let created_str = msg["Created"].as_str().unwrap_or("");
                if let Ok(created) = created_str.parse::<chrono::DateTime<chrono::Utc>>() {
                    if created < after_ts {
                        continue;
                    }
                }
            }

            let id = msg["ID"].as_str()?;
            return self.fetch_message_body(id).await;
        }

        None
    }

    async fn fetch_message_body(&self, message_id: &str) -> Option<String> {
        let msg: Value = self
            .http
            .get(format!("{}/api/v1/message/{message_id}", self.api_url))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        // Prefer plain text; fall back to HTML
        msg["Text"]
            .as_str()
            .or_else(|| msg["HTML"].as_str())
            .map(|s| s.to_owned())
    }
}

impl Default for MailpitClient {
    fn default() -> Self {
        Self::new()
    }
}

// ── Registration convenience helper ──────────────────────────────────────────

/// Register a user with email+password in one call.
///
/// Returns `(session_token, identity_id)`.
/// Panics if registration fails — use only when you need a pre-registered user
/// as a test prerequisite.
pub async fn register_user_password(email: &str, nyx_id: &str, password: &str) -> (String, String) {
    let k = KratosTestClient::new();

    let flow   = k.init_registration_flow().await;
    let flow_id = flow["id"].as_str().expect("no flow id").to_owned();

    let (status, body) =
        k.submit_registration_password(&flow_id, email, nyx_id, password).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "register_user_password failed: {body}"
    );

    let session_token = body["session_token"]
        .as_str()
        .expect("no session_token in response")
        .to_owned();

    let identity_id = body["identity"]["id"]
        .as_str()
        .expect("no identity.id in response")
        .to_owned();

    (session_token, identity_id)
}
