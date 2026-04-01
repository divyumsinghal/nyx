use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use heka::link_policy::LinkPolicyEngine;
use nun::{IdentityId, NyxApp};
use nyx_events::{subjects, EventEnvelope, EventPublisher};
use uzume_profiles::{Authenticator, Profile, ProfilePatch, ProfilesService};

// ── Local in-memory publisher for tests ──────────────────────────────────────

#[derive(Clone, Default)]
struct InMemoryEventPublisher {
    events: Arc<Mutex<Vec<EventEnvelope>>>,
}

impl InMemoryEventPublisher {
    fn snapshot(&self) -> Vec<EventEnvelope> {
        self.events.lock().unwrap().clone()
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventPublisher {
    async fn publish(&self, event: EventEnvelope) -> nun::Result<()> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

// ── Test authenticator ────────────────────────────────────────────────────────

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
async fn patch_me_publishes_uzume_profile_updated_event() {
    // #given profiles service with in-memory event publisher
    let owner = identity(1);
    let auth = TestAuthenticator::default().with_session("owner-token", owner);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");
    let publisher = InMemoryEventPublisher::default();

    let mut service = ProfilesService::new_with_events(auth, policy, publisher.clone());
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner"));

    // #when owner patches their profile
    service
        .patch_me(
            Some("owner-token"),
            ProfilePatch {
                display_name: Some("Owner Updated".to_string()),
                bio: Some("hello".to_string()),
                avatar_url: None,
                is_private: Some(true),
            },
        )
        .await
        .unwrap();

    // #then profile-updated event is recorded and payload remains alias-scoped
    let events = publisher.snapshot();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].subject, subjects::UZUME_PROFILE_UPDATED);
    assert_eq!(events[0].app, NyxApp::Uzume);
    assert_eq!(events[0].payload["alias"], "owner_alias");
    assert_eq!(events[0].payload["display_name"], "Owner Updated");
    assert_eq!(events[0].payload["is_private"], true);
    assert!(events[0].payload.get("identity_id").is_none());
}

#[tokio::test]
async fn profiles_default_constructor_remains_provider_agnostic() {
    // #given step-1 default profiles constructor
    let owner = identity(2);
    let auth = TestAuthenticator::default().with_session("owner-token", owner);
    let mut policy = LinkPolicyEngine::new();
    policy.upsert_alias(owner, NyxApp::Uzume, "owner_alias");

    let mut service = ProfilesService::new(auth, policy);
    service.insert_profile(Profile::new(owner, "owner_alias", "Owner"));

    // #when GET /me equivalent service call is executed
    let profile = service.get_me(Some("owner-token")).await.unwrap();

    // #then behavior remains unchanged and identity internals stay hidden
    assert_eq!(profile.alias, "owner_alias");
    let json = serde_json::to_value(profile).unwrap();
    assert!(json.get("identity_id").is_none());
}
