use async_trait::async_trait;
use chrono::{DateTime, Utc};
use heka::link_policy::LinkPolicyEngine;
use nun::{IdentityId, NyxApp, NyxError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Feed mode enum for future extensibility
/// Step-1 only exposes Chronological at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedMode {
    Chronological,
    // Future modes (step-2+):
    // Ranked,
    // Personalized,
}

/// Post represents a single post
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Post {
    pub id: Uuid,
    author_id: IdentityId,  // SECURITY: Private to prevent global identity leakage
    pub author_alias: String,
    pub caption: String,
    pub like_count: u64,
    pub comment_count: u64,
    pub created_at: DateTime<Utc>,
}

impl Post {
    /// Internal getter for author_id (service use only, never in responses)
    pub(crate) fn author_id(&self) -> IdentityId {
        self.author_id
    }
}

impl Post {
    pub fn new(
        id: Uuid,
        author_id: IdentityId,
        author_alias: impl Into<String>,
        caption: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            author_id,
            author_alias: author_alias.into(),
            caption: caption.into(),
            like_count: 0,
            comment_count: 0,
            created_at,
        }
    }
}

/// Public post response that hides internal implementation details
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub author_alias: String,
    pub caption: String,
    pub like_count: u64,
    pub comment_count: u64,
    pub created_at: DateTime<Utc>,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            author_alias: post.author_alias,
            caption: post.caption,
            like_count: post.like_count,
            comment_count: post.comment_count,
            created_at: post.created_at,
        }
    }
}

/// Post creation payload
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatePostPayload {
    pub caption: String,
}

/// Authenticator trait for session validation
#[async_trait]
pub trait Authenticator: Clone + Send + Sync + 'static {
    async fn validate_session(&self, session_token: &str) -> Result<IdentityId>;
}

/// FeedService implements chronological feed logic
/// Step-1: only chronological ordering, no ranking/personalization
pub struct FeedService<A>
where
    A: Authenticator,
{
    auth: A,
    policy: LinkPolicyEngine,  // SECURITY: Enforce cross-app privacy
    posts_by_id: HashMap<Uuid, Post>,
    posts_by_author: HashMap<IdentityId, Vec<Uuid>>, // newest-first
    global_timeline: Vec<Uuid>,                      // newest-first
    alias_to_identity: HashMap<String, IdentityId>,
}

impl<A> FeedService<A>
where
    A: Authenticator,
{
    pub fn new(auth: A, policy: LinkPolicyEngine) -> Self {
        Self {
            auth,
            policy,  // SECURITY: Store policy for enforcement
            posts_by_id: HashMap::new(),
            posts_by_author: HashMap::new(),
            global_timeline: Vec::new(),
            alias_to_identity: HashMap::new(),
        }
    }

    /// Register an alias for an identity (called by profiles service integration)
    pub fn register_alias(&mut self, identity: IdentityId, alias: impl Into<String>) {
        self.alias_to_identity.insert(alias.into(), identity);
    }

    /// Insert a post directly for deterministic test setup.
    pub fn seed_post_for_testing(&mut self, post: Post) {
        let post_id = post.id;
        let author_id = post.author_id;

        self.posts_by_id.insert(post_id, post);
        self.posts_by_author
            .entry(author_id)
            .or_insert_with(Vec::new)
            .insert(0, post_id);
        self.global_timeline.insert(0, post_id);
    }

    /// Create a new post (chronological): newest posts first
    pub async fn create_post(
        &mut self,
        session_token: Option<&str>,
        payload: CreatePostPayload,
    ) -> Result<PostResponse> {
        let author_id = self.require_identity(session_token).await?;

        let post = Post::new(
            Uuid::now_v7(),
            author_id,
            self.alias_for_identity(author_id)?,
            payload.caption,
            Utc::now(),
        );

        let post_id = post.id;

        // Insert into posts
        self.posts_by_id.insert(post_id, post.clone());

        // Insert into author timeline (newest first)
        self.posts_by_author
            .entry(author_id)
            .or_insert_with(Vec::new)
            .insert(0, post_id);

        // Insert into global timeline (newest first)
        self.global_timeline.insert(0, post_id);

        Ok(PostResponse::from(post))
    }

    /// Get a specific post by ID
    /// SECURITY: Requires authentication for privacy enforcement
    pub async fn get_post(
        &self,
        post_id: Uuid,
        session_token: Option<&str>,
    ) -> Result<PostResponse> {
        let viewer = self.require_identity(session_token).await?;  // SECURITY: Require auth
        
        let post = self
            .posts_by_id
            .get(&post_id)
            .ok_or_else(|| NyxError::not_found("post_not_found", "Post was not found"))?;

        // Check visibility: author can always see their own post
        let author_id = post.author_id();
        if viewer != author_id {
            // SECURITY: Enforce privacy policy
            if !self.policy.is_visible(author_id, viewer, NyxApp::Uzume, NyxApp::Uzume) {
                return Err(NyxError::forbidden(
                    "policy_denied",
                    "Access to post denied by privacy policy",
                ));
            }
        }

        Ok(PostResponse::from(post.clone()))
    }

    /// Delete a post (only author can delete)
    pub async fn delete_post(&mut self, post_id: Uuid, session_token: Option<&str>) -> Result<()> {
        let author = self.require_identity(session_token).await?;
        
        // Check if post exists and extract author_id
        let post_author_id = self
            .posts_by_id
            .get(&post_id)
            .map(|p| p.author_id())
            .ok_or_else(|| NyxError::not_found("post_not_found", "Post was not found"))?;

        if post_author_id != author {
            return Err(NyxError::forbidden(
                "not_post_author",
                "Only post author can delete",
            ));
        }

        // Remove from posts
        self.posts_by_id.remove(&post_id);

        // Remove from author timeline
        if let Some(author_posts) = self.posts_by_author.get_mut(&post_author_id) {
            author_posts.retain(|id| id != &post_id);
        }

        // Remove from global timeline
        self.global_timeline.retain(|id| id != &post_id);

        Ok(())
    }

    /// Get home timeline (chronological: newest first)
    /// Step-1: all public posts in chronological order (no ranking)
    /// SECURITY: Requires authentication to track viewer identity
    pub async fn get_home_timeline(
        &self,
        session_token: Option<&str>,
        limit: usize,
    ) -> Result<Vec<PostResponse>> {
        let _viewer = self.require_identity(session_token).await?;  // SECURITY: Require auth

        // Return chronological timeline (newest first)
        let mut posts: Vec<_> = self
            .global_timeline
            .iter()
            .filter_map(|id| self.posts_by_id.get(id).cloned())
            .collect();
        
        // Sort by created_at DESC (newest first), with id DESC as stable tie-breaker
        posts.sort_by(|a, b| {
            b.created_at.cmp(&a.created_at)
                .then_with(|| b.id.cmp(&a.id))
        });
        
        Ok(posts
            .into_iter()
            .take(limit)
            .map(|p| PostResponse::from(p))
            .collect())
    }

    /// Get user's own timeline
    pub async fn get_user_timeline(
        &self,
        alias: &str,
        _session_token: Option<&str>,
        limit: usize,
    ) -> Result<Vec<PostResponse>> {
        let user_identity = self
            .alias_to_identity
            .get(alias)
            .copied()
            .ok_or_else(|| NyxError::not_found("user_not_found", "User was not found"))?;

        // Get user's posts (should already be in newest-first order)
        let posts = self
            .posts_by_author
            .get(&user_identity)
            .map(|ids| {
                ids.iter()
                    .take(limit)
                    .filter_map(|id| self.posts_by_id.get(id))
                    .map(|p| PostResponse::from(p.clone()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(posts)
    }

    async fn require_identity(&self, session_token: Option<&str>) -> Result<IdentityId> {
        let token = session_token.ok_or_else(|| {
            NyxError::unauthorized("auth_session_token_missing", "Session token is required")
        })?;

        if token.trim().is_empty() {
            return Err(NyxError::unauthorized(
                "auth_session_token_missing",
                "Session token is required",
            ));
        }

        self.auth.validate_session(token).await
    }

    fn alias_for_identity(&self, identity: IdentityId) -> Result<String> {
        self.alias_to_identity
            .iter()
            .find_map(|(alias, id)| {
                if *id == identity {
                    Some(alias.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| NyxError::not_found("alias_not_found", "Alias not found for identity"))
    }
}

/// Endpoint handler for HTTP routing
pub struct FeedEndpoints<A>
where
    A: Authenticator,
{
    pub service: FeedService<A>,
}

impl<A> FeedEndpoints<A>
where
    A: Authenticator,
{
    pub fn new(service: FeedService<A>) -> Self {
        Self { service }
    }

    /// Register alias (integration point with profiles service)
    pub fn register_alias(&mut self, identity: IdentityId, alias: impl Into<String>) {
        self.service.register_alias(identity, alias);
    }

    /// Insert a post directly (for testing)
    pub fn insert_post(&mut self, post: Post) {
        self.service.seed_post_for_testing(post);
    }
}
