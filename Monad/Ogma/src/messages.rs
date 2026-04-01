//! Matrix messaging operations — send, fetch, and redact room messages.
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nun::{NyxError, Result};

/// Type of Matrix message content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgType {
    /// Plain-text message (`m.text`).
    Text,
    /// Image message (`m.image`).
    Image,
    /// Video message (`m.video`).
    Video,
    /// Generic file message (`m.file`).
    File,
}

impl MsgType {
    /// Returns the `m.` event content `msgtype` string used by the Matrix spec.
    fn as_matrix_str(self) -> &'static str {
        match self {
            Self::Text => "m.text",
            Self::Image => "m.image",
            Self::Video => "m.video",
            Self::File => "m.file",
        }
    }
}

/// Request to send a message to a Matrix room.
#[derive(Debug, Clone)]
pub struct SendMessageRequest {
    /// The Matrix room ID (e.g., `!abc:homeserver`).
    pub room_id: String,
    /// Human-readable body of the message.
    pub body: String,
    /// Content type of the message.
    pub msg_type: MsgType,
}

/// A single Matrix message event as returned by the homeserver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    /// The event ID assigned by the homeserver.
    pub event_id: String,
    /// Matrix user ID of the sender.
    pub sender: String,
    /// Plain-text body of the message.
    pub body: String,
    /// Unix timestamp in milliseconds.
    pub timestamp: i64,
}

/// HTTP client for Matrix room messaging operations.
///
/// Wraps a [`reqwest::Client`] and issues requests directly to the
/// `/_matrix/client/v3/` API endpoints on the Continuwuity homeserver.
///
/// # Example
///
/// ```rust,ignore
/// let client = MessageClient::new(
///     "https://matrix.example.com",
///     "syt_access_token",
/// );
/// let event_id = client
///     .send_message(SendMessageRequest {
///         room_id: "!room:example.com".to_string(),
///         body: "Hello!".to_string(),
///         msg_type: MsgType::Text,
///     })
///     .await?;
/// ```
pub struct MessageClient {
    client: Client,
    base_url: String,
    access_token: String,
}

impl MessageClient {
    /// Create a new [`MessageClient`].
    ///
    /// * `base_url` — base URL of the Continuwuity homeserver (no trailing slash).
    /// * `access_token` — server access token used in the `Authorization` header.
    pub fn new(base_url: impl Into<String>, access_token: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            access_token: access_token.into(),
        }
    }

    /// Send a message to `room_id`.
    ///
    /// Uses a random UUIDv4 transaction ID to ensure idempotency.
    /// Returns the `event_id` assigned by the homeserver on success.
    pub async fn send_message(&self, req: SendMessageRequest) -> Result<String> {
        let txn_id = Uuid::new_v4();
        let url = format!(
            "{}/_matrix/client/v3/rooms/{}/send/m.room.message/{}",
            self.base_url, req.room_id, txn_id
        );

        let body = serde_json::json!({
            "msgtype": req.msg_type.as_matrix_str(),
            "body": req.body,
        });

        let resp = self
            .client
            .put(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .map_err(NyxError::internal)?;

        if resp.status().is_success() {
            let data: serde_json::Value = resp.json().await.map_err(NyxError::internal)?;
            data["event_id"]
                .as_str()
                .map(String::from)
                .ok_or_else(|| NyxError::internal("missing event_id in Matrix send response"))
        } else {
            let status = resp.status().as_u16();
            Err(NyxError::internal(format!(
                "Matrix send_message failed with status {status}"
            )))
        }
    }

    /// Retrieve messages from `room_id`.
    ///
    /// * `from` — an optional pagination token (the `end` token from a prior response).
    /// * `limit` — maximum number of events to return (capped server-side by the homeserver).
    pub async fn get_messages(
        &self,
        room_id: &str,
        from: Option<&str>,
        limit: u32,
    ) -> Result<Vec<MessageEvent>> {
        let mut url = format!(
            "{}/_matrix/client/v3/rooms/{}/messages?dir=b&limit={}",
            self.base_url, room_id, limit
        );
        if let Some(token) = from {
            url.push_str("&from=");
            url.push_str(token);
        }

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(NyxError::internal)?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            return Err(NyxError::internal(format!(
                "Matrix get_messages failed with status {status}"
            )));
        }

        let data: serde_json::Value = resp.json().await.map_err(NyxError::internal)?;
        let chunk = data["chunk"].as_array().ok_or_else(|| {
            NyxError::internal("Matrix get_messages response missing 'chunk' field")
        })?;

        let events = chunk
            .iter()
            .filter_map(|ev| {
                let event_id = ev["event_id"].as_str()?.to_owned();
                let sender = ev["sender"].as_str()?.to_owned();
                let body = ev["content"]["body"].as_str().unwrap_or("").to_owned();
                let timestamp = ev["origin_server_ts"].as_i64().unwrap_or(0);
                Some(MessageEvent {
                    event_id,
                    sender,
                    body,
                    timestamp,
                })
            })
            .collect();

        Ok(events)
    }

    /// Redact (soft-delete) a message event.
    ///
    /// * `room_id` — the room that contains the event.
    /// * `event_id` — the event to redact.
    /// * `reason` — optional human-readable reason stored in the redaction event.
    pub async fn redact_message(
        &self,
        room_id: &str,
        event_id: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let txn_id = Uuid::new_v4();
        let url = format!(
            "{}/_matrix/client/v3/rooms/{}/redact/{}/{}",
            self.base_url, room_id, event_id, txn_id
        );

        let mut body = serde_json::json!({});
        if let Some(r) = reason {
            body["reason"] = serde_json::Value::String(r.to_owned());
        }

        let resp = self
            .client
            .put(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .map_err(NyxError::internal)?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status().as_u16();
            Err(NyxError::internal(format!(
                "Matrix redact_message failed with status {status}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msg_type_matrix_strings() {
        assert_eq!(MsgType::Text.as_matrix_str(), "m.text");
        assert_eq!(MsgType::Image.as_matrix_str(), "m.image");
        assert_eq!(MsgType::Video.as_matrix_str(), "m.video");
        assert_eq!(MsgType::File.as_matrix_str(), "m.file");
    }

    #[test]
    fn send_message_request_fields() {
        let req = SendMessageRequest {
            room_id: "!abc:example.com".to_string(),
            body: "hello".to_string(),
            msg_type: MsgType::Text,
        };
        assert_eq!(req.room_id, "!abc:example.com");
        assert_eq!(req.msg_type, MsgType::Text);
    }

    #[test]
    fn message_event_roundtrip_serde() {
        let ev = MessageEvent {
            event_id: "$abc123".to_string(),
            sender: "@alice:example.com".to_string(),
            body: "hi".to_string(),
            timestamp: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let parsed: MessageEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_id, "$abc123");
        assert_eq!(parsed.sender, "@alice:example.com");
        assert_eq!(parsed.timestamp, 1_700_000_000_000);
    }
}
