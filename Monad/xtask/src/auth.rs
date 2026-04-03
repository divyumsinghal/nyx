//! Thin Kratos HTTP client for xtask commands.
#![warn(clippy::pedantic)]

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};

const DEFAULT_HEIMDALL_URL: &str = "http://localhost:3000";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct KratosClient {
    http: Client,
    url: String,
}

impl KratosClient {
    pub fn new() -> Self {
        let url = std::env::var("HEIMDALL_URL")
            .unwrap_or_else(|_| DEFAULT_HEIMDALL_URL.to_string())
            .trim_end_matches('/')
            .to_owned();

        // Security: HTTP client with timeouts to prevent hanging
        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .pool_idle_timeout(Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest::Client");

        Self { http, url }
    }

    pub async fn check_nyx_id_available(&self, nyx_id: &str) -> Result<bool> {
        let resp = self.http
            .get(format!("{}/api/nyx/id/check-availability?id={}", self.url, nyx_id))
            .send()
            .await;

        match resp {
            Ok(r) if r.status() == StatusCode::OK => {
                let body: serde_json::Value = r.json().await?;
                Ok(body["available"].as_bool().unwrap_or(false))
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

    pub async fn submit_registration(&self, flow_id: &str, method: &str, traits: Value, extra: Value) -> Result<Value> {
        let mut body = json!({
            "method": method,
            "traits": traits,
        }).as_object().unwrap().clone();

        if let Some(obj) = extra.as_object() {
            for (k, v) in obj { body.insert(k.clone(), v.clone()); }
        }

        let resp = self.http.post(format!("{}/api/nyx/auth/self-service/registration?flow={flow_id}", self.url))
            .json(&body)
            .send().await.context("submit registration")?;

        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);

        if status == StatusCode::OK {
            Ok(body)
        } else {
            let error_msg = parse_kratos_errors(&body);
            anyhow::bail!("{}", error_msg)
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
        let mut body = json!({
            "method": method,
            "identifier": identifier,
        }).as_object().unwrap().clone();

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
}

impl Default for KratosClient {
    fn default() -> Self {
        Self::new()
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
        // Fallback to raw JSON
        serde_json::to_string_pretty(body).unwrap_or_default()
    } else {
        messages.join("\n")
    }
}
