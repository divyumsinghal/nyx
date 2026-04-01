use std::collections::HashMap;

use async_trait::async_trait;
use heka::link_policy::LinkPolicyEngine;
use nun::{IdentityId, NyxApp, NyxError};
use serde_json::json;
use uzume_profiles::{
    Authenticator, EndpointMethod, EndpointRequest, Profile, ProfilesEndpoints, ProfilesService,
};

#[derive(Clone, Default)]
struct TestAuthenticator {
    sessions: HashMap<String, IdentityId>,
}

impl TestAuthenticator {
    fn with_session(mut self, token: &str, identity: IdentityId) -> Self {
        self.sessions.insert(token.to_owned(), identity);
        self
    }
}

#[async_trait]
impl Authenticator for TestAuthenticator {
    async fn validate_session(&self, session_token: &str) -> nun::Result<IdentityId> {
        self.sessions.get(session_token).copied().ok_or_else(|| {
            NyxError::unauthorized("auth_session_invalid", "Session is invalid or expired")
        })
    }
}

fn identity(seed: u128) -> IdentityId {
    format!("00000000-0000-0000-0000-{:012x}", seed)
        .parse()
        .unwrap()
}

#[tokio::test]
async fn endpoint_me_lifecycle_authorized() {
    // #given endpoint wiring with authenticated owner profile
    let owner = identity(101);
    let auth = TestAuthenticator::default().with_session("owner-token", owner);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");
    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner"));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when calling GET /me then PATCH /me through endpoint surface
    let get_before = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/me".to_string(),
            session_token: Some("owner-token".to_string()),
            body: None,
        })
        .await;
    let patch = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Patch,
            path: "/me".to_string(),
            session_token: Some("owner-token".to_string()),
            body: Some(json!({
                "display_name": "Owner Updated",
                "bio": "hello bio",
                "avatar_url": "https://cdn.nyx/avatar.png",
                "is_private": false
            })),
        })
        .await;
    let get_after = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/me".to_string(),
            session_token: Some("owner-token".to_string()),
            body: None,
        })
        .await;

    // #then endpoint responses are successful and identity internals are not exposed
    assert_eq!(get_before.status, 200);
    assert_eq!(patch.status, 200);
    assert_eq!(get_after.status, 200);
    assert_eq!(patch.body["display_name"], "Owner Updated");
    assert!(get_after.body.get("identity_id").is_none());
}

#[tokio::test]
async fn endpoint_me_requires_auth() {
    // #given endpoint wiring without any authenticated session
    let owner = identity(102);
    let auth = TestAuthenticator::default();
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");
    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner"));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when GET /me is called without auth token
    let response = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/me".to_string(),
            session_token: None,
            body: None,
        })
        .await;

    // #then standardized unauthorized error is returned
    assert_eq!(response.status, 401);
    assert_eq!(response.body["code"], "auth_session_token_missing");
}

#[tokio::test]
async fn endpoint_private_profile_forbidden_without_visibility() {
    // #given private profile and authenticated viewer without policy visibility
    let owner = identity(103);
    let viewer = identity(104);
    let auth = TestAuthenticator::default().with_session("viewer-token", viewer);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");
    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner").with_private(true));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when GET /{alias} is called for private profile
    let response = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: Some("viewer-token".to_string()),
            body: None,
        })
        .await;

    // #then policy-denied forbidden is returned
    assert_eq!(response.status, 403);
    assert_eq!(response.body["code"], "policy_denied");
}
