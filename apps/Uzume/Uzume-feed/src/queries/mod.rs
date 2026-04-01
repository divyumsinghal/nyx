//! Raw SQL query functions for the feed service.
//!
//! All functions use `sqlx::query_as` for type-safe row mapping.
//! We use `sqlx::query_as::<_, Row>(...)` with explicit type annotation instead
//! of the compile-time macro so the crate builds without a live database.

pub mod comments;
pub mod likes;
pub mod posts;
