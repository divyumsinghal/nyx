use std::collections::HashMap;

use async_trait::async_trait;
use heka::link_policy::{LinkPolicyEngine, LinkRule};
use nun::{IdentityId, LinkPolicy, NyxApp};
use uzume_profiles::{
    Authenticator, Profile, ProfilePatch, ProfilesService, PublicProfileResponse,
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
            nun::NyxError::unauthorized("auth_session_invalid", "Session is invalid or expired")
        })
    }
}

fn identity(seed: u128) -> IdentityId {
    format!("00000000-0000-0000-0000-{:012x}", seed)
        .parse()
        .unwrap()
}

#[tokio::test]
async fn me_profile_lifecycle_and_public_read() {
    // #given an authenticated user with an app-scoped alias and profile
    let owner = identity(1);
    let auth = TestAuthenticator::default().with_session("owner-token", owner);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner"));

    // #when fetching /me, patching /me, and reading public /{alias}
    let me_before = service.get_me(Some("owner-token")).await.unwrap();
    let me_after = service
        .patch_me(
            Some("owner-token"),
            ProfilePatch {
                display_name: Some("Owner Updated".to_string()),
                bio: Some("hello bio".to_string()),
                avatar_url: Some("https://cdn.nyx/avatar.png".to_string()),
                is_private: Some(false),
            },
        )
        .await
        .unwrap();
    let public = service
        .get_public_profile("owner_alias", None)
        .await
        .unwrap();

    // #then lifecycle state is consistent and response does not leak global identity internals
    assert_eq!(me_before.alias, "owner_alias");
    assert_eq!(me_after.display_name, "Owner Updated");
    assert_eq!(public, PublicProfileResponse::from(me_after.clone()));

    let value = serde_json::to_value(public).unwrap();
    assert!(value.get("identity_id").is_none());
}

#[tokio::test]
async fn me_endpoints_require_authentication() {
    // #given a service with one profile but no caller token
    let owner = identity(2);
    let auth = TestAuthenticator::default();
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner"));

    // #when calling protected /me endpoints without token
    let get_err = service.get_me(None).await.unwrap_err();
    let patch_err = service
        .patch_me(None, ProfilePatch::default())
        .await
        .unwrap_err();

    // #then both paths are unauthorized with standardized Nun errors
    assert_eq!(get_err.status_code(), 401);
    assert_eq!(get_err.code(), "auth_session_token_missing");
    assert_eq!(patch_err.status_code(), 401);
    assert_eq!(patch_err.code(), "auth_session_token_missing");
}

#[tokio::test]
async fn private_profile_access_denied_without_policy_visibility() {
    // #given owner and viewer identities with a private owner profile and no link policy
    let owner = identity(3);
    let viewer = identity(4);
    let auth = TestAuthenticator::default().with_session("viewer-token", viewer);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner").with_private(true));

    // #when viewer requests public /{alias} with authenticated context
    let err = service
        .get_public_profile("owner_alias", Some("viewer-token"))
        .await
        .unwrap_err();

    // #then access is forbidden due to default-deny policy
    assert_eq!(err.status_code(), 403);
    assert_eq!(err.code(), "policy_denied");
}

#[tokio::test]
async fn private_profile_access_allowed_when_policy_allows() {
    // #given owner and viewer with explicit visibility policy
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
        policy: LinkPolicy::TwoWay,
    });

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner").with_private(true));

    // #when viewer requests private profile and policy permits visibility
    let profile = service
        .get_public_profile("owner_alias", Some("viewer-token"))
        .await
        .unwrap();

    // #then read succeeds with only public profile fields
    assert_eq!(profile.alias, "owner_alias");
    assert!(profile.is_private);
}

#[tokio::test]
async fn private_profile_without_auth_token_is_forbidden() {
    // #given a private profile and no viewer auth context
    let owner = identity(7);
    let auth = TestAuthenticator::default();
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner").with_private(true));

    // #when public profile read is attempted without token
    let err = service
        .get_public_profile("owner_alias", None)
        .await
        .unwrap_err();

    // #then default-deny returns forbidden
    assert_eq!(err.status_code(), 403);
    assert_eq!(err.code(), "policy_denied");
}
