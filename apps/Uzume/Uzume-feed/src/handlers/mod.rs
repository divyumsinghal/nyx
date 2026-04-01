//! Axum handler modules for the feed service.

pub mod interactions;
pub mod posts;

// Re-export the shared error type so sub-handlers can use `super::ApiError`.
pub use posts::ApiError;
