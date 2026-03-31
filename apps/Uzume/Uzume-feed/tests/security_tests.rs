use async_trait::async_trait;
use chrono::Utc;
use heka::link_policy::LinkPolicyEngine;
use nun::{IdentityId, NyxError, Result};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use uzume_feed::{Authenticator, FeedService, Post, PostResponse};

/// Test authenticator that tracks calls
#[derive(Clone)]
struct TestAuthenticator {
    valid_tokens: Arc<Mutex<std::collections::HashMap<String, IdentityId>>>,
}

impl TestAuthenticator {
    fn new() -> Self {
        Self {
            valid_tokens: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    fn with_session(self, token: &str, identity: IdentityId) -> Self {
        self.valid_tokens
            .lock()
            .unwrap()
            .insert(token.to_string(), identity);
        self
    }
}

#[async_trait]
impl Authenticator for TestAuthenticator {
    async fn validate_session(&self, session_token: &str) -> Result<IdentityId> {
        self.valid_tokens
            .lock()
            .unwrap()
            .get(session_token)
            .copied()
            .ok_or_else(|| NyxError::unauthorized("invalid_token", "Invalid session token"))
    }
}

fn identity(n: u64) -> IdentityId {
    let uuid = Uuid::parse_str(&format!("550e8400-e29b-41d4-a716-{:012}", n))
        .expect("Valid UUID format");
    IdentityId::from_uuid(uuid)
}

// ============================================================
// SECURITY TESTS: Authentication Enforcement
// ============================================================

#[tokio::test]
async fn get_home_timeline_denies_unauthenticated_access() {
    // #given a feed service with a post
    let author = identity(1);
    let auth = TestAuthenticator::new().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test_user");

    let post = Post::new(Uuid::now_v7(), author, "test_user", "Test post", Utc::now());
    service.seed_post_for_testing(post);

    // #when calling get_home_timeline WITHOUT authentication
    let result = service.get_home_timeline(None, 10).await;

    // #then it returns 401 Unauthorized
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "auth_session_token_missing");
}

#[tokio::test]
async fn get_home_timeline_requires_valid_token() {
    // #given a feed service
    let author = identity(1);
    let auth = TestAuthenticator::new().with_session("valid-token", author);
    let policy = LinkPolicyEngine::new();
    let service = FeedService::new(auth, policy);

    // #when calling get_home_timeline with invalid token
    let result = service.get_home_timeline(Some("invalid-token"), 10).await;

    // #then it returns 401 Unauthorized
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "invalid_token");  // From validate_session
}

#[tokio::test]
async fn get_post_denies_unauthenticated_access() {
    // #given a feed service with a post
    let author = identity(1);
    let auth = TestAuthenticator::new().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test_user");

    let post = Post::new(Uuid::now_v7(), author, "test_user", "Test post", Utc::now());
    let post_id = post.id;
    service.seed_post_for_testing(post);

    // #when calling get_post WITHOUT authentication
    let result = service.get_post(post_id, None).await;

    // #then it returns 401 Unauthorized
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "auth_session_token_missing");
}

#[tokio::test]
async fn get_post_requires_valid_token() {
    // #given a feed service with a post
    let author = identity(1);
    let auth = TestAuthenticator::new().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test_user");

    let post = Post::new(Uuid::now_v7(), author, "test_user", "Test post", Utc::now());
    let post_id = post.id;
    service.seed_post_for_testing(post);

    // #when calling get_post with invalid token
    let result = service.get_post(post_id, Some("invalid-token")).await;

    // #then it returns 401 Unauthorized
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "invalid_token");  // From validate_session
}

// ============================================================
// SECURITY TESTS: Identity Encapsulation
// ============================================================

#[test]
fn post_serialization_does_not_expose_global_identity() {
    // #given a post with a sensitive global identity
    let sensitive_identity = identity(999);
    let post = Post::new(
        Uuid::now_v7(),
        sensitive_identity,
        "public_alias",
        "Test caption",
        Utc::now(),
    );

    // #when serializing the post response (public output)
    let response = PostResponse::from(post);
    let json = serde_json::to_value(&response).unwrap();

    // #then the global IdentityId is NOT exposed
    assert!(
        json.get("author_id").is_none(),
        "Global author_id must not appear in serialized response"
    );
    assert!(
        json.get("identity_id").is_none(),
        "identity_id must not appear in serialized response"
    );
    // ONLY the alias should be present
    assert_eq!(json["author_alias"], "public_alias");
}

#[test]
fn post_response_only_exposes_alias_not_identity() {
    // #given a post
    let identity = identity(5);
    let post = Post::new(Uuid::now_v7(), identity, "user_alias", "Content", Utc::now());
    let response = PostResponse::from(post);

    // #then the response has only alias, never identity
    assert_eq!(response.author_alias, "user_alias");
    // Verify serialize doesn't include author_id
    let json = serde_json::to_string(&response).unwrap();
    assert!(!json.contains("550e8400"), "Global identity should not appear in JSON");
}

// ============================================================
// SECURITY TESTS: Privacy Policy Enforcement
// ============================================================

#[tokio::test]
async fn get_post_enforces_visibility_policy() {
    // #given two users
    let alice = identity(1);
    let bob = identity(2);
    let auth = TestAuthenticator::new()
        .with_session("alice-token", alice)
        .with_session("bob-token", bob);
    let policy = LinkPolicyEngine::new();

    // #and alice and bob are NOT linked
    // (policy remains empty, so no cross-user visibility)

    let mut service = FeedService::new(auth, policy);
    service.register_alias(alice, "alice");
    service.register_alias(bob, "bob");

    // #and alice posts
    let post = Post::new(Uuid::now_v7(), alice, "alice", "Private content", Utc::now());
    let post_id = post.id;
    service.seed_post_for_testing(post);

    // #when bob tries to view alice's post
    let result = service.get_post(post_id, Some("bob-token")).await;

    // #then bob is denied (policy doesn't allow visibility)
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "policy_denied");
}

#[tokio::test]
async fn get_post_allows_author_to_view_own_post() {
    // #given alice with a post
    let alice = identity(1);
    let auth = TestAuthenticator::new().with_session("alice-token", alice);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(alice, "alice");

    let post = Post::new(Uuid::now_v7(), alice, "alice", "My post", Utc::now());
    let post_id = post.id;
    service.seed_post_for_testing(post);

    // #when alice views her own post
    let result = service.get_post(post_id, Some("alice-token")).await;

    // #then author can always view (no policy check needed)
    assert!(result.is_ok());
    assert_eq!(result.unwrap().caption, "My post");
}

// ============================================================
// SECURITY TESTS: Empty Token Handling
// ============================================================

#[tokio::test]
async fn get_home_timeline_rejects_empty_token() {
    // #given a feed service
    let author = identity(1);
    let auth = TestAuthenticator::new().with_session("valid", author);
    let policy = LinkPolicyEngine::new();
    let service = FeedService::new(auth, policy);

    // #when calling with empty string token
    let result = service.get_home_timeline(Some(""), 10).await;

    // #then it's rejected as invalid
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "auth_session_token_missing");  // Empty tokens are treated as missing
}

#[tokio::test]
async fn get_post_rejects_empty_token() {
    // #given a feed service with a post
    let author = identity(1);
    let auth = TestAuthenticator::new().with_session("valid", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test");

    let post = Post::new(Uuid::now_v7(), author, "test", "Post", Utc::now());
    let post_id = post.id;
    service.seed_post_for_testing(post);

    // #when calling with empty string token
    let result = service.get_post(post_id, Some("")).await;

    // #then it's rejected as invalid
    assert!(result.is_err());
}

// ============================================================
// SECURITY TESTS: Authorization (Author-Only Operations)
// ============================================================

#[tokio::test]
async fn delete_post_denied_for_non_author() {
    // #given alice and bob
    let alice = identity(1);
    let bob = identity(2);
    let auth = TestAuthenticator::new()
        .with_session("alice-token", alice)
        .with_session("bob-token", bob);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(alice, "alice");
    service.register_alias(bob, "bob");

    // #and alice creates a post
    let post = Post::new(Uuid::now_v7(), alice, "alice", "Alice's post", Utc::now());
    let post_id = post.id;
    service.seed_post_for_testing(post);

    // #when bob tries to delete alice's post
    let result = service.delete_post(post_id, Some("bob-token")).await;

    // #then deletion is denied
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "not_post_author");
}

#[tokio::test]
async fn delete_post_allowed_for_author() {
    // #given alice with a post
    let alice = identity(1);
    let auth = TestAuthenticator::new().with_session("alice-token", alice);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(alice, "alice");

    let post = Post::new(Uuid::now_v7(), alice, "alice", "Alice's post", Utc::now());
    let post_id = post.id;
    service.seed_post_for_testing(post);

    // #when alice deletes her post
    let result = service.delete_post(post_id, Some("alice-token")).await;

    // #then deletion succeeds
    assert!(result.is_ok());
}
