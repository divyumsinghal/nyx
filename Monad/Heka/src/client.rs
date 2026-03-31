//! Ory Kratos session validation client.
use reqwest::Client;

use Nun::config::AuthConfig;
use Nun::{NyxError, Result};

use crate::identity::Session;

/// HTTP client for the Kratos public API.
pub struct KratosClient {
    client: Client,
    base_url: String,
}

impl KratosClient {
    /// Build a new client from the auth configuration.
    pub fn new(config: &AuthConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.public_url.clone(),
        }
    }

    /// Validate a session token with Kratos (`/sessions/whoami`).
    pub async fn whoami(&self, session_token: &str) -> Result<Session> {
        let url = format!("{}/sessions/whoami", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("X-Session-Token", session_token)
            .send()
            .await
            .map_err(NyxError::internal)?;

        if resp.status().is_success() {
            resp.json::<Session>()
                .await
                .map_err(NyxError::internal)
        } else {
            Err(NyxError::unauthorized(
                "invalid_session",
                "Session is invalid or expired",
            ))
        }
    }
}
