//! Gorush HTTP client for APNs (iOS) and FCM (Android) push notifications.
//!
//! Gorush is a Go-based push notification server. This module wraps its
//! `/api/push` endpoint with a typed Rust interface.
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::UshasError;

/// The device platform for a push notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Platform {
    /// Apple Push Notification service (APNs).
    Ios = 1,
    /// Firebase Cloud Messaging (FCM / Android).
    Android = 2,
}

impl Platform {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

/// A push notification to dispatch via Gorush.
#[derive(Debug, Clone)]
pub struct PushNotification {
    /// Device tokens to deliver to.
    pub tokens: Vec<String>,
    /// Target platform.
    pub platform: Platform,
    /// Notification title displayed on the device.
    pub title: String,
    /// Notification body text displayed on the device.
    pub body: String,
    /// Arbitrary JSON data payload delivered silently alongside the visible alert.
    pub data: serde_json::Value,
}

/// Gorush `/api/push` success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GorushResponse {
    /// Total number of notifications accepted for delivery.
    pub counts: u32,
    /// Per-device delivery logs (empty on full success).
    pub logs: Vec<GorushLog>,
}

/// A single delivery log entry returned by Gorush.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GorushLog {
    /// Log type, e.g. `"failed"`.
    #[serde(rename = "type")]
    pub type_: String,
    /// Platform string, e.g. `"ios"` or `"android"`.
    pub platform: String,
    /// Device token.
    pub token: String,
    /// Error message or delivery status.
    pub message: String,
}

// ── Wire types (Gorush JSON schema) ────────────────────────────────────────

#[derive(Serialize)]
struct GorushNotification<'a> {
    tokens: &'a [String],
    platform: u8,
    title: &'a str,
    body: &'a str,
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    data: &'a serde_json::Value,
}

#[derive(Serialize)]
struct GorushRequest<'a> {
    notifications: Vec<GorushNotification<'a>>,
}

#[derive(Deserialize)]
struct GorushRawResponse {
    counts: Option<u32>,
    logs: Option<Vec<GorushLog>>,
}

/// HTTP client for the Gorush push notification gateway.
///
/// # Example
///
/// ```rust,ignore
/// let client = GorushClient::new("http://gorush:8088");
/// client.send(PushNotification {
///     tokens: vec!["device_token".to_string()],
///     platform: Platform::Ios,
///     title: "New message".to_string(),
///     body: "Alice sent you a message".to_string(),
///     data: serde_json::json!({ "room_id": "!abc:nyx.app" }),
/// }).await?;
/// ```
pub struct GorushClient {
    base_url: String,
    client: Client,
}

impl GorushClient {
    /// Create a new `GorushClient` pointing at `base_url` (no trailing slash).
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client: Client::new(),
        }
    }

    /// Dispatch `notification` via the Gorush gateway.
    ///
    /// Returns the aggregated [`GorushResponse`] on success.
    pub async fn send(&self, notification: PushNotification) -> Result<GorushResponse, UshasError> {
        let url = format!("{}/api/push", self.base_url);

        let payload = GorushRequest {
            notifications: vec![GorushNotification {
                tokens: &notification.tokens,
                platform: notification.platform.as_u8(),
                title: &notification.title,
                body: &notification.body,
                data: &notification.data,
            }],
        };

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| UshasError::GorushError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(UshasError::GorushError(format!(
                "Gorush returned HTTP {}",
                resp.status().as_u16()
            )));
        }

        let raw: GorushRawResponse = resp
            .json()
            .await
            .map_err(|e| UshasError::GorushError(e.to_string()))?;

        Ok(GorushResponse {
            counts: raw.counts.unwrap_or(0),
            logs: raw.logs.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_values() {
        assert_eq!(Platform::Ios.as_u8(), 1);
        assert_eq!(Platform::Android.as_u8(), 2);
    }

    #[test]
    fn push_notification_fields() {
        let n = PushNotification {
            tokens: vec!["tok".to_string()],
            platform: Platform::Android,
            title: "Hi".to_string(),
            body: "World".to_string(),
            data: serde_json::json!({}),
        };
        assert_eq!(n.tokens.len(), 1);
        assert_eq!(n.platform, Platform::Android);
    }

    #[test]
    fn gorush_response_deserialization() {
        let json = r#"{"counts": 1, "logs": []}"#;
        let resp: GorushRawResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.counts, Some(1));
        assert!(resp.logs.unwrap().is_empty());
    }
}
