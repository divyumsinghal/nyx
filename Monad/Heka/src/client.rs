use std::sync::Arc;

use nun::{IdentityId, NyxError, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::types::NyxIdentity;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct KratosSession {
    identity: KratosIdentity,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct KratosIdentity {
    id: String,
    traits: KratosIdentityTraits,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct KratosIdentityTraits {
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KratosProviderError {
    Status(u16),
    Network,
    Decode,
}

#[async_trait::async_trait]
pub trait KratosProvider: Send + Sync {
    async fn fetch_session(
        &self,
        session_token: &str,
    ) -> std::result::Result<serde_json::Value, KratosProviderError>;

    async fn fetch_identity(
        &self,
        identity_id: &str,
    ) -> std::result::Result<serde_json::Value, KratosProviderError>;
}

#[derive(Clone)]
pub struct KratosClient {
    provider: Arc<dyn KratosProvider>,
}

impl KratosClient {
    pub fn new(public_url: impl Into<String>, admin_url: impl Into<String>) -> Self {
        Self::with_provider(ReqwestKratosProvider::new(public_url, admin_url))
    }

    pub fn with_provider<P>(provider: P) -> Self
    where
        P: KratosProvider + 'static,
    {
        Self {
            provider: Arc::new(provider),
        }
    }

    pub async fn validate_session(&self, session_token: &str) -> Result<NyxIdentity> {
        if session_token.trim().is_empty() {
            return Err(NyxError::bad_request(
                "auth_session_token_missing",
                "Session token is required",
            ));
        }

        let session = self
            .provider
            .fetch_session(session_token)
            .await
            .and_then(parse_session)
            .map_err(map_provider_error)?;
        map_kratos_identity(session.identity)
    }

    pub async fn get_identity(&self, identity_id: IdentityId) -> Result<NyxIdentity> {
        let identity = self
            .provider
            .fetch_identity(&identity_id.to_string())
            .await
            .and_then(parse_identity)
            .map_err(map_provider_error)?;
        map_kratos_identity(identity)
    }
}

fn parse_session(
    payload: serde_json::Value,
) -> std::result::Result<KratosSession, KratosProviderError> {
    serde_json::from_value(payload).map_err(|_| KratosProviderError::Decode)
}

fn parse_identity(
    payload: serde_json::Value,
) -> std::result::Result<KratosIdentity, KratosProviderError> {
    serde_json::from_value(payload).map_err(|_| KratosProviderError::Decode)
}

fn map_kratos_identity(identity: KratosIdentity) -> Result<NyxIdentity> {
    let _ = (&identity.traits.email, &identity.traits.phone);
    let id = identity.id.parse::<IdentityId>().map_err(|_| {
        NyxError::service_unavailable(
            "auth_provider_invalid_response",
            "Authentication provider returned malformed identity payload",
        )
    })?;

    Ok(NyxIdentity { id })
}

fn map_provider_error(error: KratosProviderError) -> NyxError {
    match error {
        KratosProviderError::Status(401) => {
            NyxError::unauthorized("auth_session_invalid", "Session is invalid or expired")
        }
        KratosProviderError::Status(403) => NyxError::forbidden(
            "auth_session_forbidden",
            "Session does not have required privileges",
        ),
        KratosProviderError::Status(404) => {
            NyxError::not_found("auth_identity_not_found", "Identity was not found")
        }
        KratosProviderError::Status(status) if status >= 500 => NyxError::service_unavailable(
            "auth_provider_unavailable",
            "Authentication provider is unavailable",
        ),
        KratosProviderError::Network => NyxError::service_unavailable(
            "auth_network_unreachable",
            "Authentication provider is unreachable",
        ),
        KratosProviderError::Decode => NyxError::service_unavailable(
            "auth_provider_invalid_response",
            "Authentication provider returned malformed data",
        ),
        KratosProviderError::Status(status) => NyxError::bad_request(
            "auth_provider_request_failed",
            format!("Authentication provider rejected request with status {status}"),
        ),
    }
}

#[derive(Clone)]
pub struct ReqwestKratosProvider {
    http: Client,
    public_url: String,
    admin_url: String,
}

impl ReqwestKratosProvider {
    pub fn new(public_url: impl Into<String>, admin_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            public_url: public_url.into().trim_end_matches('/').to_owned(),
            admin_url: admin_url.into().trim_end_matches('/').to_owned(),
        }
    }
}

#[async_trait::async_trait]
impl KratosProvider for ReqwestKratosProvider {
    async fn fetch_session(
        &self,
        session_token: &str,
    ) -> std::result::Result<serde_json::Value, KratosProviderError> {
        let response = self
            .http
            .get(format!("{}/sessions/whoami", self.public_url))
            .header("X-Session-Token", session_token)
            .send()
            .await
            .map_err(|_| KratosProviderError::Network)?;

        if !response.status().is_success() {
            return Err(KratosProviderError::Status(response.status().as_u16()));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|_| KratosProviderError::Decode)
    }

    async fn fetch_identity(
        &self,
        identity_id: &str,
    ) -> std::result::Result<serde_json::Value, KratosProviderError> {
        let response = self
            .http
            .get(format!("{}/admin/identities/{identity_id}", self.admin_url))
            .send()
            .await
            .map_err(|_| KratosProviderError::Network)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(KratosProviderError::Status(404));
        }

        if !response.status().is_success() {
            return Err(KratosProviderError::Status(response.status().as_u16()));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|_| KratosProviderError::Decode)
    }
}
