//! Pre-built test fixtures and factories.

use chrono::{DateTime, Utc};
use nun::id::Id;
use nun::NyxApp;
use uuid::Uuid;

/// Marker types for test entities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestUser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestPost;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestStory;

/// Fixed test UUIDs for deterministic tests.
pub mod fixed_ids {
    use super::*;

    /// Fixed UUID for test user Alice
    pub fn alice_uuid() -> Uuid {
        Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap()
    }

    /// Fixed UUID for test user Bob
    pub fn bob_uuid() -> Uuid {
        Uuid::parse_str("fedcba98-7654-3210-fedc-ba9876543210").unwrap()
    }

    /// Fixed UUID for test user Charlie
    pub fn charlie_uuid() -> Uuid {
        Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap()
    }

    pub fn alice_id<T>() -> Id<T> {
        Id::from_uuid(alice_uuid())
    }

    pub fn bob_id<T>() -> Id<T> {
        Id::from_uuid(bob_uuid())
    }

    pub fn charlie_id<T>() -> Id<T> {
        Id::from_uuid(charlie_uuid())
    }
}

/// Test user data builder.
pub struct TestUserBuilder {
    pub id: Id<TestUser>,
    pub email: String,
    pub phone: Option<String>,
    pub display_name: String,
    pub app: NyxApp,
}

impl TestUserBuilder {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            email: crate::common::random_email(),
            phone: Some(crate::common::random_phone()),
            display_name: "Test User".to_string(),
            app: NyxApp::Uzume,
        }
    }

    pub fn alice() -> Self {
        Self {
            id: fixed_ids::alice_id(),
            email: "alice@example.com".to_string(),
            phone: Some("+15551234567".to_string()),
            display_name: "Alice".to_string(),
            app: NyxApp::Uzume,
        }
    }

    pub fn bob() -> Self {
        Self {
            id: fixed_ids::bob_id(),
            email: "bob@example.com".to_string(),
            phone: Some("+15559876543".to_string()),
            display_name: "Bob".to_string(),
            app: NyxApp::Uzume,
        }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }

    pub fn with_app(mut self, app: NyxApp) -> Self {
        self.app = app;
        self
    }

    pub fn build(self) -> TestUserData {
        TestUserData {
            id: self.id,
            email: self.email,
            phone: self.phone,
            display_name: self.display_name,
            app: self.app,
        }
    }
}

impl Default for TestUserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Test user data.
#[derive(Debug, Clone)]
pub struct TestUserData {
    pub id: Id<TestUser>,
    pub email: String,
    pub phone: Option<String>,
    pub display_name: String,
    pub app: NyxApp,
}

/// Test post data builder.
pub struct TestPostBuilder {
    pub id: Id<TestPost>,
    pub author_id: Id<TestUser>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl TestPostBuilder {
    pub fn new(author_id: Id<TestUser>) -> Self {
        Self {
            id: Id::new(),
            author_id,
            content: "Test post content".to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn build(self) -> TestPostData {
        TestPostData {
            id: self.id,
            author_id: self.author_id,
            content: self.content,
            created_at: self.created_at,
        }
    }
}

/// Test post data.
#[derive(Debug, Clone)]
pub struct TestPostData {
    pub id: Id<TestPost>,
    pub author_id: Id<TestUser>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_builder_creates_valid_user() {
        let user = TestUserBuilder::new().build();
        assert!(!user.email.is_empty());
        assert_eq!(user.app, NyxApp::Uzume);
    }

    #[test]
    fn alice_builder_creates_alice() {
        let alice = TestUserBuilder::alice().build();
        assert_eq!(alice.email, "alice@example.com");
        assert_eq!(alice.display_name, "Alice");
        assert_eq!(alice.id, fixed_ids::alice_id());
    }

    #[test]
    fn test_post_builder_works() {
        let author_id = Id::new();
        let post = TestPostBuilder::new(author_id)
            .with_content("Hello world")
            .build();
        assert_eq!(post.content, "Hello world");
        assert_eq!(post.author_id, author_id);
    }
}
