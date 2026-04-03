//! Thin Kratos HTTP client for xtask commands.
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

        let parsed_url = Url::parse(&url).context("HEIMDALL_URL must be a valid URL")?;

        let allow_insecure_local_tls = std::env::var("NYX_INSECURE_TLS")
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false);

        let host = parsed_url.host_str().unwrap_or_default();
        let is_local_host = matches!(host, "localhost" | "127.0.0.1" | "::1");
        let env = std::env::var("NYX_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let may_accept_invalid_certs = allow_insecure_local_tls && is_local_host && env == "development";

        // Security: HTTP client with timeouts to prevent hanging
        let mut client_builder = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .pool_idle_timeout(Duration::from_secs(60));

        if may_accept_invalid_certs {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let http = client_builder
            .build()
            .context("failed to build reqwest::Client")?;

        Ok(Self { http, url })
    }

    pub async fn check_nyx_id_available(&self, nyx_id: &str) -> Result<bool> {
        let resp = self
            .http
            .post(format!("{}/api/nyx/id/check-availability", self.url))
            .json(&json!({ "id": nyx_id }))
            .send()
            .await;

        match resp {
            Ok(r) if r.status() == StatusCode::OK => {
                let body: serde_json::Value = r.json().await?;
                body["available"]
                    .as_bool()
                    .context("check-availability response missing boolean `available`")
            }
            Ok(r) => anyhow::bail!("check nyx_id failed: {}", r.status()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn init_registration_flow(&self) -> Result<Value> {
        let resp = self.http.get(format!("{}/api/nyx/auth/self-service/registration/api", self.url))
            .send().await.context("init registration flow")?;

        if resp.status() != StatusCode::OK {
            anyhow::bail!("init registration failed: {}", resp.status());
        }
        resp.json().await.context("parse registration flow")
    }

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
            .post(format!("{}/api/nyx/auth/self-service/registration?flow={flow_id}", self.url))
            .json(&body)
            .send()
            .await
            .context("submit registration code init")?;

        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);

        if status.is_success() || status == StatusCode::UNPROCESSABLE_ENTITY {
            Ok(body)
        } else {
            anyhow::bail!(parse_kratos_errors(&body))
        }
    }

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
            .post(format!("{}/api/nyx/auth/self-service/registration?flow={flow_id}", self.url))
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

    pub async fn init_login_flow(&self) -> Result<Value> {
        let resp = self.http.get(format!("{}/api/nyx/auth/self-service/login/api", self.url))
            .send().await.context("init login flow")?;

        if resp.status() != StatusCode::OK {
            anyhow::bail!("init login failed: {}", resp.status());
        }
        resp.json().await.context("parse login flow")
    }

    pub async fn submit_login(&self, flow_id: &str, method: &str, identifier: &str, extra: Value) -> Result<Value> {
        let mut body = serde_json::Map::new();
        body.insert("method".to_string(), Value::String(method.to_string()));
        body.insert("identifier".to_string(), Value::String(identifier.to_string()));

        if let Some(obj) = extra.as_object() {
            for (k, v) in obj { body.insert(k.clone(), v.clone()); }
        }

        let resp = self.http.post(format!("{}/api/nyx/auth/self-service/login?flow={flow_id}", self.url))
            .json(&body)
            .send().await.context("submit login")?;

        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);

        if status == StatusCode::OK {
            Ok(body)
        } else {
            let error_msg = parse_kratos_errors(&body);
            anyhow::bail!("{}", error_msg)
        }
    }

    pub async fn password_is_breached(&self, password: &str) -> Result<bool> {
        let hash = Sha1::digest(password.as_bytes());
        let hash_hex = format!("{:x}", hash).to_uppercase();
        let (prefix, suffix) = hash_hex.split_at(5);

        let response = self
            .http
            .get(format!("https://api.pwnedpasswords.com/range/{prefix}"))
            .header("Add-Padding", "true")
            .send()
            .await
            .context("query breached password database")?;

        if !response.status().is_success() {
            anyhow::bail!("breached-password check failed: {}", response.status());
        }

        let body = response.text().await.context("read breached-password response")?;
        Ok(body.lines().any(|line| line.split_once(':').is_some_and(|(candidate, _)| candidate.trim() == suffix)))
    }
}

impl Default for KratosClient {
    fn default() -> Self {
        Self::new().expect("Failed to create KratosClient from environment")
    }
}

/// Parse Kratos error response into human-readable message.
pub fn parse_kratos_errors(body: &Value) -> String {
    let mut messages = Vec::new();

    // UI node errors (field-level validation)
    if let Some(nodes) = body["ui"]["nodes"].as_array() {
        for node in nodes {
            if let Some(name) = node["attributes"]["name"].as_str() {
                if let Some(node_msgs) = node["messages"].as_array() {
                    for msg in node_msgs {
                        if let Some(text) = msg["text"].as_str() {
                            messages.push(format!("{}: {}", name, text));
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

    // Generic error
    if let Some(err) = body["error"]["message"].as_str() {
        messages.push(err.to_string());
    }

    if messages.is_empty() {
        "Unexpected Kratos error response".to_string()
    } else {
        messages.join("\n")
    }
}
