//! Shared test helpers for auth integration tests.
//!
//! Provides thin clients for Kratos APIs plus test data generators.
//! All HTTP calls use the real APIs — no mocks anywhere.
//!
//! OTP retrieval uses the Kratos admin courier API
//! (`GET /admin/courier/messages`) which stores every dispatched message
//! regardless of which SMTP provider is configured.  This means CI reads OTPs
//! without needing IMAP access or a fake inbox.

use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
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

/// Returns the dedicated test email address used by OTP integration tests.
///
/// This must be a real address that receives email — set `E2E_TEST_EMAIL` in
/// `.env` or CI secrets.  OTP tests are marked `#[serial]` so at most one
/// test is waiting for a message at any given time.
pub fn e2e_test_email() -> String {
    std::env::var("E2E_TEST_EMAIL")
        .expect("E2E_TEST_EMAIL must be set to run OTP integration tests")
        .trim()
        .to_lowercase()
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

/// Generate a unique test email address for test runs.
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
    pub http: Client,
    public_url: String,
    admin_url: String,
}

impl KratosTestClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            public_url: kratos_public(),
            admin_url: kratos_admin(),
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

        resp.json::<Value>()
            .await
            .expect("init_registration_flow: parse error")
    }

    /// Submit a password registration (single step).
    ///
    /// Returns the full response body (session token on 200, flow on 422).
    pub async fn submit_registration_password(
        &self,
        flow_id: &str,
        email: &str,
        nyx_id: &str,
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
        let body = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    /// Step 1 of OTP registration: submit email + nyx_id to trigger the code email.
    ///
    /// Returns 422 with the updated flow (which now has a `code` input field).
    pub async fn submit_registration_code_init(
        &self,
        flow_id: &str,
        email: &str,
        nyx_id: &str,
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
        let body = resp.json::<Value>().await.unwrap_or(Value::Null);
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
        code: &str,
        email: &str,
        nyx_id: &str,
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
        let body = resp.json::<Value>().await.unwrap_or(Value::Null);
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

        resp.json::<Value>()
            .await
            .expect("init_login_flow: parse error")
    }

    /// Submit password login. `identifier` may be email or nyx_id.
    pub async fn submit_login_password(
        &self,
        flow_id: &str,
        identifier: &str,
        password: &str,
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
        let body = resp.json::<Value>().await.unwrap_or(Value::Null);
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
        let body = resp.json::<Value>().await.unwrap_or(Value::Null);
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

        resp.json::<Value>()
            .await
            .expect("init_recovery_flow: parse error")
    }

    /// Step 1: submit email to receive recovery code.
    pub async fn submit_recovery_email(&self, flow_id: &str, email: &str) -> (StatusCode, Value) {
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
        let body = resp.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    /// Step 2: submit recovery code.
    pub async fn submit_recovery_code(&self, flow_id: &str, code: &str) -> (StatusCode, Value) {
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
        let body = resp.json::<Value>().await.unwrap_or(Value::Null);
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
        let body = resp.json::<Value>().await.unwrap_or(Value::Null);
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

// ── Kratos courier inbox client ───────────────────────────────────────────────

/// Reads OTP codes from the Kratos admin courier API.
///
/// Kratos stores every dispatched email message in its own DB
/// (`courier_messages` table), accessible via `GET /admin/courier/messages`.
/// This works regardless of which SMTP provider is configured — we don't need
/// a fake SMTP inbox or real IMAP access to retrieve OTPs in tests.
pub struct KratosInboxClient {
    http: Client,
    admin_url: String,
}

impl KratosInboxClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            admin_url: kratos_admin(),
        }
    }

    /// Wait up to `timeout_secs` for an OTP email to `to_address` that was
    /// dispatched at or after `after`.
    ///
    /// The timestamp filter prevents a test from picking up an OTP that was
    /// sent during a previous test or run.  Polls every 500 ms.
    pub async fn wait_for_email_after(
        &self,
        to_address: &str,
        timeout_secs: u64,
        after: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Option<String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);

        while std::time::Instant::now() < deadline {
            if let Some(body) = self.find_message_body(to_address, after).await {
                return Some(body);
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        None
    }

    /// Convenience wrapper: no time filter.
    pub async fn wait_for_email(&self, to_address: &str, timeout_secs: u64) -> Option<String> {
        self.wait_for_email_after(to_address, timeout_secs, None)
            .await
    }

    /// Extract a 6-digit OTP code from a Kratos email body.
    ///
    /// Kratos sends OTPs as plain 6-digit numbers.  The regex matches the
    /// first occurrence surrounded by non-digit context.
    pub fn extract_code(body: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"(?:^|[\s\-:])(\d{6})(?:$|[\s\.\,])").unwrap();
        re.captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_owned())
    }

    // ── Private ──────────────────────────────────────────────────────────────

    /// Query `GET /admin/courier/messages`, filter by recipient + time.
    async fn find_message_body(
        &self,
        to_address: &str,
        after: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Option<String> {
        // Fetch the most recent 50 messages — more than enough for any test run.
        let messages: Value = self
            .http
            .get(format!(
                "{}/admin/courier/messages?page_size=50",
                self.admin_url
            ))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        // Kratos returns an array at the root, newest-first.
        let messages = messages.as_array()?;

        for msg in messages {
            // Filter by recipient.
            let recipient = msg["recipient"].as_str().unwrap_or("");
            if !recipient.eq_ignore_ascii_case(to_address) {
                continue;
            }

            // Time filter: skip messages dispatched before `after`.
            if let Some(after_ts) = after {
                let created_str = msg["created_at"].as_str().unwrap_or("");
                if let Ok(created) = created_str.parse::<chrono::DateTime<chrono::Utc>>() {
                    if created < after_ts {
                        continue;
                    }
                }
            }

            // Return the message body (contains the OTP code).
            if let Some(body) = msg["body"].as_str() {
                return Some(body.to_owned());
            }
        }

        None
    }
}

impl Default for KratosInboxClient {
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

    let flow = k.init_registration_flow().await;
    let flow_id = flow["id"].as_str().expect("no flow id").to_owned();

    let (status, body) = k
        .submit_registration_password(&flow_id, email, nyx_id, password)
        .await;

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
