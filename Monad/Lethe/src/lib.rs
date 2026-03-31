//! # Lethe — DragonflyDB/Redis cache client
//!
//! Named for the river of forgetfulness: a cache forgets (TTL expiry).
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod rate_limit;
pub mod session;

pub use client::{connect, CacheClient};
