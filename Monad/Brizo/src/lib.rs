//! # Brizo — Meilisearch search client
//!
//! Index convention: `{app}_{entity}` (e.g. `Uzume_posts`, `Uzume_profiles`).
//!
//! ## Usage
//!
//! ```rust,ignore
//! let client = brizo::connect(&config.search);
//! let index = client.index(brizo::indexes::UZUME_POSTS);
//! index.search().with_query("hello").execute::<Post>().await?;
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod indexes;

pub use client::{connect, SearchClient};
