use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use heka::client::{KratosClient, KratosProvider, KratosProviderError};
use nun::IdentityId;

#[derive(Clone)]
struct ProviderTraits {
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Clone)]
struct ProviderIdentity {
    id: String,
    traits: ProviderTraits,
}

#[derive(Clone)]
struct ProviderSession {
    identity: ProviderIdentity,
}

impl From<ProviderTraits> for serde_json::Value {
    fn from(value: ProviderTraits) -> Self {
        serde_json::json!({
            "email": value.email,
            "phone": value.phone,
        })
    }
}

impl From<ProviderIdentity> for serde_json::Value {
    fn from(value: ProviderIdentity) -> Self {
        serde_json::json!({
            "id": value.id,
            "traits": serde_json::Value::from(value.traits),
        })
    }
}

impl From<ProviderSession> for serde_json::Value {
    fn from(value: ProviderSession) -> Self {
        serde_json::json!({
            "identity": serde_json::Value::from(value.identity),
        })
    }
}

fn session_from_value(value: serde_json::Value) -> Result<serde_json::Value, KratosProviderError> {
    if value.get("identity").is_none() {
        return Err(KratosProviderError::Decode);
    }
    Ok(value)
}

fn identity_from_value(value: serde_json::Value) -> Result<serde_json::Value, KratosProviderError> {
    if value.get("id").is_none() || value.get("traits").is_none() {
        return Err(KratosProviderError::Decode);
    }
    Ok(value)
}

#[derive(Clone)]
struct MockProvider {
    session_results: Arc<Mutex<VecDeque<Result<serde_json::Value, KratosProviderError>>>>,
    identity_results: Arc<Mutex<VecDeque<Result<serde_json::Value, KratosProviderError>>>>,
}

impl MockProvider {
    fn with_session_result(result: Result<serde_json::Value, KratosProviderError>) -> Self {
        Self {
            session_results: Arc::new(Mutex::new(VecDeque::from([result]))),
            identity_results: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn with_identity_result(result: Result<serde_json::Value, KratosProviderError>) -> Self {
        Self {
            session_results: Arc::new(Mutex::new(VecDeque::new())),
            identity_results: Arc::new(Mutex::new(VecDeque::from([result]))),
        }
    }
}

#[async_trait::async_trait]
impl KratosProvider for MockProvider {
    async fn fetch_session(
        &self,
        _session_token: &str,
    ) -> Result<serde_json::Value, KratosProviderError> {
        let value = self
            .session_results
            .lock()
            .unwrap()
            .pop_front()
            .expect("missing mock session result")?;
        session_from_value(value)
    }

    async fn fetch_identity(
        &self,
        _identity_id: &str,
    ) -> Result<serde_json::Value, KratosProviderError> {
        let value = self
            .identity_results
            .lock()
            .unwrap()
            .pop_front()
            .expect("missing mock identity result")?;
        identity_from_value(value)
    }
}

#[tokio::test]
async fn validate_session_returns_identity_for_valid_session() {
    // #given a valid Kratos session payload with typed identity id
    let id: IdentityId = "0195d0eb-8857-7d8e-8a10-ec8fdc357e7e".parse().unwrap();
    let provider =
        MockProvider::with_session_result(Ok(serde_json::Value::from(ProviderSession {
            identity: ProviderIdentity {
                id: id.to_string(),
                traits: ProviderTraits {
                    email: Some("private@example.com".to_string()),
                    phone: Some("+15551234567".to_string()),
                },
            },
        })));
    let client = KratosClient::with_provider(provider);

    // #when validating session
    let result = client.validate_session("session-token").await;

    // #then a typed Nyx identity is returned without provider internals
    let identity = result.unwrap();
    assert_eq!(identity.id, id);
}

#[tokio::test]
async fn validate_session_rejects_empty_token() {
    // #given an empty caller-provided session token
    let client = KratosClient::with_provider(MockProvider::with_session_result(Err(
        KratosProviderError::Network,
    )));

    // #when validating session
    let result = client.validate_session("  ").await;

    // #then deterministic bad request classification is returned
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), 400);
    assert_eq!(err.code(), "auth_session_token_missing");
}

#[tokio::test]
async fn validate_session_maps_401_to_unauthorized() {
    // #given Kratos reports invalid or expired session
    let client = KratosClient::with_provider(MockProvider::with_session_result(Err(
        KratosProviderError::Status(401),
    )));

    // #when validating session
    let result = client.validate_session("expired-token").await;

    // #then error maps to Nun unauthorized deterministically
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), 401);
    assert_eq!(err.code(), "auth_session_invalid");
}

#[tokio::test]
async fn validate_session_maps_403_to_forbidden() {
    // #given Kratos reports forbidden session scope
    let client = KratosClient::with_provider(MockProvider::with_session_result(Err(
        KratosProviderError::Status(403),
    )));

    // #when validating session
    let result = client.validate_session("forbidden-token").await;

    // #then error maps to Nun forbidden deterministically
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), 403);
    assert_eq!(err.code(), "auth_session_forbidden");
}

#[tokio::test]
async fn validate_session_maps_5xx_to_service_unavailable() {
    // #given Kratos upstream failure
    let client = KratosClient::with_provider(MockProvider::with_session_result(Err(
        KratosProviderError::Status(502),
    )));

    // #when validating session
    let result = client.validate_session("valid-token").await;

    // #then error maps to deterministic service unavailable category
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), 503);
    assert_eq!(err.code(), "auth_provider_unavailable");
}

#[tokio::test]
async fn validate_session_maps_network_failure_to_service_unavailable() {
    // #given Kratos network connectivity failure
    let client = KratosClient::with_provider(MockProvider::with_session_result(Err(
        KratosProviderError::Network,
    )));

    // #when validating session
    let result = client.validate_session("valid-token").await;

    // #then error maps to deterministic network-unreachable classification
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), 503);
    assert_eq!(err.code(), "auth_network_unreachable");
}

#[tokio::test]
async fn validate_session_maps_malformed_payload_to_invalid_response() {
    // #given Kratos returns an unreadable session payload
    let client = KratosClient::with_provider(MockProvider::with_session_result(Err(
        KratosProviderError::Decode,
    )));

    // #when validating session
    let result = client.validate_session("valid-token").await;

    // #then malformed payload maps to deterministic invalid-response category
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), 503);
    assert_eq!(err.code(), "auth_provider_invalid_response");
}

#[tokio::test]
async fn get_identity_maps_malformed_provider_id_to_invalid_response() {
    // #given Kratos identity payload contains malformed UUID value
    let provider =
        MockProvider::with_identity_result(Ok(serde_json::Value::from(ProviderIdentity {
            id: "not-a-uuid".to_string(),
            traits: ProviderTraits {
                email: Some("private@example.com".to_string()),
                phone: Some("+15551234567".to_string()),
            },
        })));
    let client = KratosClient::with_provider(provider);

    // #when looking up identity
    let result = client
        .get_identity("0195d0eb-8857-7d8e-8a10-ec8fdc357e7e".parse().unwrap())
        .await;

    // #then malformed provider payload maps to deterministic unavailable error
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), 503);
    assert_eq!(err.code(), "auth_provider_invalid_response");
}

#[tokio::test]
async fn get_identity_maps_404_to_not_found() {
    // #given Kratos admin API cannot find identity
    let client = KratosClient::with_provider(MockProvider::with_identity_result(Err(
        KratosProviderError::Status(404),
    )));

    // #when looking up identity by typed id
    let result = client
        .get_identity("0195d0eb-8857-7d8e-8a10-ec8fdc357e7e".parse().unwrap())
        .await;

    // #then deterministic not found classification is returned
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), 404);
    assert_eq!(err.code(), "auth_identity_not_found");
}

#[tokio::test]
async fn validate_session_rejects_identity_with_unknown_fields() {
    // #given a provider session payload containing unexpected fields
    let id: IdentityId = "0195d0eb-8857-7d8e-8a10-ec8fdc357e7e".parse().unwrap();
    let provider = MockProvider::with_session_result(Ok(serde_json::json!({
        "identity": {
            "id": id.to_string(),
            "traits": {
                "email": "private@example.com",
                "phone": "+15551234567",
                "leak": "should_fail"
            }
        }
    })));
    let client = KratosClient::with_provider(provider);

    // #when validating session
    let result = client.validate_session("session-token").await;

    // #then strict provider boundary mapping rejects payload as invalid response
    let err = result.unwrap_err();
    assert_eq!(err.status_code(), 503);
    assert_eq!(err.code(), "auth_provider_invalid_response");
}
