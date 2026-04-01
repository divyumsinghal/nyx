use chrono::{Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use async_trait::async_trait;
use heka::link_policy::LinkPolicyEngine;
use nun::IdentityId;
use uzume_feed::{Authenticator, CreatePostPayload, FeedService, Post};

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
async fn feed_chronological_newest_first_ordering() {
    // #given a feed service with multiple posts created at different times
    let author = identity(1);
    let auth = TestAuthenticator::default().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test_user");

    // Create posts with known timestamps (3 seconds apart)
    let now = Utc::now();
    let post1_time = now - Duration::seconds(6); // oldest
    let post2_time = now - Duration::seconds(3);
    let post3_time = now; // newest

    let post1 = Post::new(
        Uuid::now_v7(),
        author,
        "test_user",
        "First post",
        post1_time,
    );
    let post2 = Post::new(
        Uuid::now_v7(),
        author,
        "test_user",
        "Second post",
        post2_time,
    );
    let post3 = Post::new(
        Uuid::now_v7(),
        author,
        "test_user",
        "Third post",
        post3_time,
    );

    let mut endpoints = uzume_feed::FeedEndpoints::new(service);
    endpoints.insert_post(post1.clone());
    endpoints.insert_post(post2.clone());
    endpoints.insert_post(post3.clone());

    // #when fetching home timeline
    let timeline = endpoints
        .service
        .get_home_timeline(Some("author-token"), 100)
        .await
        .unwrap();

    // #then posts are ordered newest-first (post3, post2, post1)
    assert_eq!(timeline.len(), 3);
    assert_eq!(timeline[0].caption, "Third post");
    assert_eq!(timeline[1].caption, "Second post");
    assert_eq!(timeline[2].caption, "First post");
}

#[tokio::test]
async fn feed_chronological_stable_tiebreak_with_uuid() {
    // #given two posts created at the same moment (same created_at timestamp)
    let author = identity(2);
    let auth = TestAuthenticator::default().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test_user");

    let now = Utc::now();
    let post1_id = Uuid::now_v7();
    let post2_id = Uuid::now_v7();

    // Create two posts at identical timestamps
    let post1 = Post::new(post1_id, author, "test_user", "Post A", now);
    let post2 = Post::new(post2_id, author, "test_user", "Post B", now);

    let mut endpoints = uzume_feed::FeedEndpoints::new(service);
    endpoints.insert_post(post1.clone());
    endpoints.insert_post(post2.clone());

    // #when fetching timeline
    let timeline = endpoints
        .service
        .get_home_timeline(Some("author-token"), 100)
        .await
        .unwrap();

    // #then order is stable (insertion order acts as tiebreaker via global_timeline vec)
    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].caption, "Post B"); // inserted last = newest
    assert_eq!(timeline[1].caption, "Post A"); // inserted first = older
}

#[tokio::test]
async fn feed_create_post_inserts_newest_first() {
    // #given a feed service with one existing post
    let author = identity(3);
    let auth = TestAuthenticator::default().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth.clone(), policy);
    service.register_alias(author, "test_user");

    let old_now = Utc::now() - Duration::seconds(10);
    let old_post = Post::new(Uuid::now_v7(), author, "test_user", "Old post", old_now);

    let mut endpoints = uzume_feed::FeedEndpoints::new(service);
    endpoints.insert_post(old_post.clone());

    // #when creating a new post
    let new_response = endpoints
        .service
        .create_post(
            Some("author-token"),
            CreatePostPayload {
                caption: "New post".to_string(),
            },
        )
        .await
        .unwrap();

    // #then the new post appears first in timeline
    let timeline = endpoints
        .service
        .get_home_timeline(Some("author-token"), 100)
        .await
        .unwrap();

    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].caption, "New post");
    assert_eq!(timeline[0].id, new_response.id);
    assert_eq!(timeline[1].caption, "Old post");
}

#[tokio::test]
async fn feed_delete_post_maintains_chronological_order() {
    // #given a timeline with posts: A (old), B (mid), C (new)
    let author = identity(4);
    let auth = TestAuthenticator::default().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test_user");

    let now = Utc::now();
    let post_a = Post::new(
        Uuid::now_v7(),
        author,
        "test_user",
        "Post A",
        now - Duration::seconds(6),
    );
    let post_b = Post::new(
        Uuid::now_v7(),
        author,
        "test_user",
        "Post B",
        now - Duration::seconds(3),
    );
    let post_c = Post::new(Uuid::now_v7(), author, "test_user", "Post C", now);

    let post_b_id = post_b.id;

    let mut endpoints = uzume_feed::FeedEndpoints::new(service);
    endpoints.insert_post(post_a);
    endpoints.insert_post(post_b);
    endpoints.insert_post(post_c);

    // #when deleting post B (middle)
    endpoints
        .service
        .delete_post(post_b_id, Some("author-token"))
        .await
        .unwrap();

    // #then remaining posts are still in chronological order: C, A
    let timeline = endpoints
        .service
        .get_home_timeline(Some("author-token"), 100)
        .await
        .unwrap();

    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].caption, "Post C"); // newest
    assert_eq!(timeline[1].caption, "Post A"); // oldest
}

#[tokio::test]
async fn feed_limit_respects_chronological_ordering() {
    // #given a feed with 5 posts in chronological order
    let author = identity(5);
    let auth = TestAuthenticator::default().with_session("author-token", author);
    let policy = LinkPolicyEngine::new();
    let mut service = FeedService::new(auth, policy);
    service.register_alias(author, "test_user");

    let now = Utc::now();
    for i in 0..5 {
        let post = Post::new(
            Uuid::now_v7(),
            author,
            "test_user",
            format!("Post {}", i),
            now - Duration::seconds(i as i64 * 10),
        );
        let mut endpoints = uzume_feed::FeedEndpoints::new(service);
        endpoints.insert_post(post);
        service = endpoints.service;
    }

    // #when fetching with limit=3
    let timeline = service
        .get_home_timeline(Some("author-token"), 3)
        .await
        .unwrap();

    // #then the 3 newest posts are returned in chronological order
    assert_eq!(timeline.len(), 3);
    assert_eq!(timeline[0].caption, "Post 0"); // newest
    assert_eq!(timeline[1].caption, "Post 1");
    assert_eq!(timeline[2].caption, "Post 2"); // oldest in result
}
