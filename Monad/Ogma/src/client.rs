//! Matrix/Continuwuity HTTP client.
use reqwest::Client;
use serde_json::{json, Value};

use nun::config::MessagingConfig;
use nun::{NyxError, Result};

/// HTTP client for the Continuwuity (Matrix) homeserver.
pub struct MatrixClient {
    client: Client,
    base_url: String,
    access_token: String,
}

impl MatrixClient {
    /// Build a new client from messaging config and a server access token.
    pub fn new(config: &MessagingConfig, access_token: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: config.homeserver_url.clone(),
            access_token: access_token.into(),
        }
    }

    /// Create a direct message room and invite `invitee` (Matrix user ID).
    /// Returns the `room_id` of the newly created room.
    pub async fn create_dm_room(&self, invitee: &str) -> Result<String> {
        let url = format!("{}/_matrix/client/v3/createRoom", self.base_url);
        let body = json!({
            "preset": "private_chat",
            "invite": [invitee],
            "is_direct": true,
        });

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .map_err(NyxError::internal)?;

        if resp.status().is_success() {
            let data: Value = resp.json().await.map_err(NyxError::internal)?;
            data["room_id"]
                .as_str()
                .map(String::from)
                .ok_or_else(|| NyxError::internal("missing room_id in Matrix response"))
        } else {
            Err(NyxError::internal("Matrix room creation failed"))
        }
    }
}
