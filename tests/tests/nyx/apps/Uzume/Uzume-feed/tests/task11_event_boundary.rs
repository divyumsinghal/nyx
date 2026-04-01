use std::collections::HashMap;

use async_trait::async_trait;
use heka::link_policy::LinkPolicyEngine;
use nun::{IdentityId, NyxApp};
use nyx_events::{subjects, InMemoryEventPublisher};
use uzume_feed::{Authenticator, CreatePostPayload, FeedService};

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
async fn create_post_publishes_uzume_post_created_event() {
    // #given feed service with in-memory events publisher
    let author = identity(1);
    let auth = TestAuthenticator::default().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let publisher = InMemoryEventPublisher::default();

    let mut service = FeedService::new_with_events(auth, policy, publisher.clone());
    service.register_alias(author, "author_alias");

    // #when a post is created
    let created = service
        .create_post(
            Some("author-token"),
            CreatePostPayload {
                caption: "hello world".to_string(),
            },
        )
        .await
        .unwrap();

    // #then a typed uzume post-created event is published without leaking global identity
    let events = publisher.snapshot();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].subject, subjects::UZUME_POST_CREATED);
    assert_eq!(events[0].app, NyxApp::Uzume);
    assert_eq!(events[0].payload["post_id"], created.id.to_string());
    assert_eq!(events[0].payload["author_alias"], "author_alias");
    assert!(events[0].payload.get("author_id").is_none());
    assert!(events[0].payload.get("identity_id").is_none());
}

#[tokio::test]
async fn feed_default_constructor_remains_provider_agnostic() {
    // #given step-1 default feed service constructor
    let author = identity(2);
    let auth = TestAuthenticator::default().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "author_alias");

    // #when creating a post using the default constructor (no provider wiring)
    let created = service
        .create_post(
            Some("author-token"),
            CreatePostPayload {
                caption: "works".to_string(),
            },
        )
        .await
        .unwrap();

    // #then service behavior still works with chronological runtime semantics
    let timeline = service
        .get_home_timeline(Some("author-token"), 10)
        .await
        .unwrap();

    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline[0].id, created.id);
    assert_eq!(timeline[0].author_alias, "author_alias");
}
