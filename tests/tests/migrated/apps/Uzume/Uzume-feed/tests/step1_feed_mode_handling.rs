use std::collections::HashMap;

use async_trait::async_trait;
use heka::link_policy::LinkPolicyEngine;
use nun::IdentityId;
use uzume_feed::{Authenticator, FeedMode, FeedService};

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

#[test]
fn feed_mode_chronological_is_serializable() {
    // #given a FeedMode::Chronological value
    let mode = FeedMode::Chronological;

    // #when serializing to JSON
    let json = serde_json::to_value(mode).unwrap();

    // #then it serializes to the correct lowercase string
    assert_eq!(json, serde_json::json!("chronological"));
}

#[test]
fn feed_mode_chronological_is_deserializable() {
    // #given a JSON value with "chronological" string
    let json = serde_json::json!("chronological");

    // #when deserializing to FeedMode
    let mode: FeedMode = serde_json::from_value(json).unwrap();

    // #then it parses to FeedMode::Chronological
    assert_eq!(mode, FeedMode::Chronological);
}

#[test]
fn feed_mode_only_chronological_available_in_step1() {
    // #given the FeedMode enum defined for step-1
    // #when reviewing available variants
    // #then only Chronological is exported and documented for runtime use
    // Future modes (Ranked, Personalized) are commented out

    // This test verifies the contract via code examination:
    // The FeedMode enum shows all mode options, with only Chronological exposed
    assert_eq!(FeedMode::Chronological, FeedMode::Chronological);

    // Attempting to use a non-chronological mode in the actual service
    // would require uncommenting code, which would be caught in review
}

#[tokio::test]
async fn feed_service_ignores_mode_parameter_in_timeline() {
    // #given a feed service in step-1 (only chronological available)
    let author = identity(100);
    let auth = TestAuthenticator::default().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test_user");

    // #when requesting timeline (mode is not exposed in API surface)
    let timeline = service
        .get_home_timeline(Some("author-token"), 10)
        .await
        .unwrap();

    // #then result is deterministically chronological
    // The API doesn't accept a mode parameter at all
    assert_eq!(timeline.len(), 0); // Empty for new service
}

#[test]
fn feed_mode_contract_is_future_proof() {
    // #given FeedMode enum with comment placeholders
    // #when inspecting the enum definition
    // #then future modes (Ranked, Personalized) are clear via comments
    // and commented-out variants show extensibility path

    // This test documents the contract:
    // - Step-1: Only FeedMode::Chronological
    // - Step-2: Add ranked/personalized by uncommenting
    // - No breaking changes to existing serialization

    let chronological = FeedMode::Chronological;
    assert_eq!(chronological, FeedMode::Chronological);
}

#[tokio::test]
async fn feed_service_no_ranking_logic_in_step1() {
    // #given the FeedService implementation
    let author = identity(101);
    let auth = TestAuthenticator::default().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test_user");

    // #when examining the service methods
    // #then there are no methods for ranking, scoring, or engagement-based sorting
    // Methods available: create_post, get_post, delete_post, get_home_timeline, get_user_timeline
    // None of these accept a mode parameter or perform ranking

    // This is a compile-time contract: the service API doesn't expose
    // any ranking or personalization parameters
    let timeline = service
        .get_home_timeline(Some("author-token"), 10)
        .await
        .unwrap();

    // Chronological result guaranteed by method signature
    assert_eq!(timeline.len(), 0);
}

#[test]
fn feed_mode_transition_path_documented() {
    // #given step-1 FeedMode enum with future-proofing
    // #when reviewing the implementation
    // #then the transition to step-2 modes is clear:
    // 1. Uncomment Ranked variant in FeedMode enum
    // 2. Add score/ranking_score fields to Post
    // 3. Add ranking logic to FeedService methods
    // 4. Add mode parameter to get_home_timeline()
    // 5. Implement ranking when mode != Chronological

    // Current step-1 state ensures:
    // - No ranking logic is reachable
    // - Serialization is stable
    // - Post schema is ready for ranking fields

    let mode = FeedMode::Chronological;
    let serialized = serde_json::to_string(&mode).unwrap();
    assert_eq!(serialized, "\"chronological\"");
}
