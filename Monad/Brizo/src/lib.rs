//! # Brizo — Meilisearch search client
//!
//! Index convention: `{app}_{entity}` (e.g. `Uzume_posts`, `Uzume_profiles`).
//!
//! ## Usage
//!
//! ### Searching
//!
//! ```rust,ignore
//! use brizo::{connect, indexes, query::SearchRequest};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Post { id: String, caption: String }
//!
//! let client = brizo::connect(&config.search);
//! let req = SearchRequest::new("sunset").with_limit(10);
//! let resp = client.search::<Post>(indexes::UZUME_POSTS, req).await?;
//! println!("{} hits in {}ms", resp.total, resp.processing_time_ms);
//! ```
//!
//! ### Syncing documents from NATS event handlers
//!
//! ```rust,ignore
//! use brizo::{connect, indexes, sync::IndexSync};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Post { id: String, caption: String }
//!
//! let sync = IndexSync::new(client.clone());
//! sync.add_document(indexes::UZUME_POSTS, &post).await?;
//! sync.delete_document(indexes::UZUME_POSTS, &post_id).await?;
//! ```
//!
//! ### Low-level index access
//!
//! ```rust,ignore
//! // Raw meilisearch-sdk index handle for one-off operations.
//! let index = client.index(brizo::indexes::UZUME_POSTS);
//! index.search().with_query("hello").execute::<Post>().await?;
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod indexes;
pub mod query;
pub mod sync;

pub use client::{connect, SearchClient};
pub use query::{SearchHit, SearchRequest, SearchResponse};
pub use sync::IndexSync;
