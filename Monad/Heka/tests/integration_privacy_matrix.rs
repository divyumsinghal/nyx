use std::collections::HashMap;

use async_trait::async_trait;
use heka::link_policy::{LinkPolicyEngine, LinkRule};
use nun::{IdentityId, LinkPolicy, NyxApp, NyxError};
use serde_json::json;
use uzume_profiles::{
    Authenticator as ProfileAuthenticator, EndpointMethod, EndpointRequest, Profile,
    ProfilesEndpoints, ProfilesService,
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
impl ProfileAuthenticator for TestAuthenticator {
    async fn validate_session(&self, session_token: &str) -> nun::Result<IdentityId> {
        self.sessions
            .get(session_token)
            .copied()
            .ok_or_else(|| NyxError::unauthorized("auth_session_invalid", "Session is invalid or expired"))
    }
}

fn identity(seed: u128) -> IdentityId {
    format!("00000000-0000-0000-0000-{:012x}", seed)
        .parse()
        .unwrap()
}

#[tokio::test]
async fn cross_app_default_deny_with_no_links() {
    // #given owner in Uzume and viewer with no link policy
    let owner = identity(1);
    let viewer = identity(2);
    let auth = TestAuthenticator::default().with_session("viewer-token", viewer);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner").with_private(true));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when viewer (different identity) tries to access owner's private profile
    let response = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: Some("viewer-token".to_string()),
            body: None,
        })
        .await;

    // #then access is denied by default-deny policy
    assert_eq!(response.status, 403);
    assert_eq!(response.body["code"], "policy_denied");
}

#[tokio::test]
async fn explicit_two_way_link_allows_access() {
    // #given owner and viewer with two-way link established
    let owner = identity(3);
    let viewer = identity(4);
    let auth = TestAuthenticator::default().with_session("viewer-token", viewer);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");
    policy.upsert(LinkRule {
        owner,
        viewer,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Uzume,
        policy: LinkPolicy::TwoWay,
    });

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner").with_private(true));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when viewer requests private profile with valid link
    let response = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: Some("viewer-token".to_string()),
            body: None,
        })
        .await;

    // #then access is granted and profile is visible
    assert_eq!(response.status, 200);
    assert_eq!(response.body["alias"], "owner_alias");
    assert_eq!(response.body["display_name"], "Owner");
    assert!(response.body.get("identity_id").is_none()); // Global identity not exposed
}

#[tokio::test]
async fn one_way_link_owner_to_viewer_grants_access() {
    // #given owner (A) creates one-way link revealing to viewer (B)
    let owner = identity(5);
    let viewer = identity(6);
    let auth = TestAuthenticator::default().with_session("viewer-token", viewer);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");
    policy.upsert(LinkRule {
        owner,
        viewer,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Uzume,
        policy: LinkPolicy::OneWay,
    });

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner").with_private(true));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when viewer attempts to access owner's private profile
    let response = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: Some("viewer-token".to_string()),
            body: None,
        })
        .await;

    // #then access is granted (one-way is sufficient for viewer to see owner)
    assert_eq!(response.status, 200);
    assert_eq!(response.body["alias"], "owner_alias");
}

#[tokio::test]
async fn revoked_link_immediately_denies_access() {
    // #given owner and viewer with established two-way link
    let owner = identity(7);
    let viewer = identity(8);
    let auth = TestAuthenticator::default().with_session("viewer-token", viewer);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");
    policy.upsert(LinkRule {
        owner,
        viewer,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Uzume,
        policy: LinkPolicy::TwoWay,
    });

    let mut service = ProfilesService::new(auth.clone(), policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner").with_private(true));
    let mut endpoints = ProfilesEndpoints::new(service);

    // Verify initial access is granted
    let response_before = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: Some("viewer-token".to_string()),
            body: None,
        })
        .await;
    assert_eq!(response_before.status, 200);

    // #when revoking the link
    // Recreate service with revoked link (link rule removed)
    let mut policy_revoked = LinkPolicyEngine::new();
    policy_revoked.upsert_alias(owner, NyxApp::Uzume, "owner_alias");
    // Link is NOT added, effectively revoking it

    let service_revoked = ProfilesService::new(auth, policy_revoked);
    let mut endpoints_revoked = ProfilesEndpoints::new(service_revoked);
    let mut profile = Profile::new(owner, "owner_alias", "Owner").with_private(true);
    endpoints_revoked.insert_profile(profile);

    // #then access reverts to default-deny
    let response_after = endpoints_revoked
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: Some("viewer-token".to_string()),
            body: None,
        })
        .await;
    assert_eq!(response_after.status, 403);
    assert_eq!(response_after.body["code"], "policy_denied");
}

#[tokio::test]
async fn forged_app_context_isolation_enforcement() {
    // #given owner with private profile in Uzume
    let owner = identity(9);
    let viewer = identity(10);
    let auth = TestAuthenticator::default().with_session("viewer-token", viewer);

    // Set up link only for cross-app access (Uzume to Anteros)
    // But our service is Uzume-profiles checking Uzume-to-Uzume
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");
    policy.upsert(LinkRule {
        owner,
        viewer,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Anteros, // Link is for different app pair
        policy: LinkPolicy::TwoWay,
    });

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner").with_private(true));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when viewer attempts to access with link for wrong app pair
    let response = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: Some("viewer-token".to_string()),
            body: None,
        })
        .await;

    // #then access is denied (wrong app context)
    assert_eq!(response.status, 403);
    assert_eq!(response.body["code"], "policy_denied");
}

#[tokio::test]
async fn public_profile_accessible_without_authentication() {
    // #given an owner with a public profile (is_private=false)
    let owner = identity(11);
    let auth = TestAuthenticator::default();
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Public Owner").with_private(false));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when accessing public profile without authentication
    let response = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: None,
            body: None,
        })
        .await;

    // #then public profile is accessible
    assert_eq!(response.status, 200);
    assert_eq!(response.body["alias"], "owner_alias");
    assert_eq!(response.body["is_private"], false);
}

#[tokio::test]
async fn private_profile_blocks_unauthenticated_access() {
    // #given an owner with a private profile
    let owner = identity(12);
    let auth = TestAuthenticator::default();
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Private Owner").with_private(true));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when accessing private profile without any token
    let response = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: None,
            body: None,
        })
        .await;

    // #then access is denied with policy_denied
    assert_eq!(response.status, 403);
    assert_eq!(response.body["code"], "policy_denied");
}

#[tokio::test]
async fn owner_can_always_see_own_profile_regardless_of_privacy() {
    // #given owner with private profile
    let owner = identity(13);
    let auth = TestAuthenticator::default().with_session("owner-token", owner);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Private Owner").with_private(true));
    let mut endpoints = ProfilesEndpoints::new(service);

    // #when owner tries to view own profile
    let response = endpoints
        .handle(EndpointRequest {
            method: EndpointMethod::Get,
            path: "/owner_alias".to_string(),
            session_token: Some("owner-token".to_string()),
            body: None,
        })
        .await;

    // #then access is granted (owner can always see self)
    assert_eq!(response.status, 200);
    assert_eq!(response.body["alias"], "owner_alias");
}
