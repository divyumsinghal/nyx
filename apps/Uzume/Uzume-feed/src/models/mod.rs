//! Database-mapped types for the feed service.
//!
//! These types mirror the `uzume.posts`, `uzume.post_likes`, and
//! `uzume.comments` PostgreSQL tables and are used exclusively by the queries
//! layer.

pub mod comment;
pub mod post;
