//! Index synchronisation helpers for Brizo.
//!
//! Services that consume NATS events (e.g. `Uzume.post.created`) call these
//! helpers to keep Meilisearch in sync with the primary database.
//!
//! # Design
//!
//! - [`IndexSync`] is a thin, cloneable handle backed by a [`SearchClient`].
//! - [`IndexSync::add_document`] upserts a single serialisable document.
//! - [`IndexSync::delete_document`] removes a document by its string ID.
//!
//! Both methods fire-and-forget the Meilisearch task (they do not wait for
//! indexing to complete). This keeps event handler latency low; Meilisearch
//! will persist the task and retry on transient failures internally.
//!
//! # Example
//!
//! ```rust,ignore
//! use brizo::{connect, indexes, sync::IndexSync};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Post { id: String, caption: String, author_alias: String }
//!
//! async fn on_post_created(client: &brizo::SearchClient, post: Post) -> nun::Result<()> {
//!     let sync = IndexSync::new(client.clone());
//!     sync.add_document(indexes::UZUME_POSTS, &post).await
//! }
//!
//! async fn on_post_deleted(client: &brizo::SearchClient, id: &str) -> nun::Result<()> {
//!     let sync = IndexSync::new(client.clone());
//!     sync.delete_document(indexes::UZUME_POSTS, id).await
//! }
//! ```

use serde::Serialize;
use tracing::instrument;

use nun::Result;

use crate::client::SearchClient;

// ── IndexSync ─────────────────────────────────────────────────────────────────

/// Handle for syncing documents to a Meilisearch index.
///
/// Cheap to clone — internally holds only an `Arc`-backed [`SearchClient`].
#[derive(Clone, Debug)]
pub struct IndexSync {
    client: SearchClient,
}

impl IndexSync {
    /// Create a new [`IndexSync`] from an existing [`SearchClient`].
    #[must_use]
    pub fn new(client: SearchClient) -> Self {
        Self { client }
    }

    /// Upsert a document into `index_name`.
    ///
    /// The document is serialised to JSON and sent to Meilisearch. If a
    /// document with the same primary key already exists it is replaced
    /// (add-or-replace semantics).
    ///
    /// This method enqueues the task in Meilisearch but does **not** wait for
    /// it to complete — indexing is eventually consistent.
    ///
    /// # Errors
    ///
    /// Returns [`nun::NyxError::internal`] if the Meilisearch HTTP request fails
    /// or if `doc` cannot be serialised to JSON.
    #[instrument(skip(self, doc), fields(index = index_name))]
    pub async fn add_document<T>(&self, index_name: &str, doc: &T) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        self.client
            .add_documents(index_name, std::slice::from_ref(doc))
            .await
    }

    /// Delete the document with the given `id` from `index_name`.
    ///
    /// No-op if no document with that ID exists. Enqueues the deletion task
    /// without waiting for completion.
    ///
    /// # Errors
    ///
    /// Returns [`nun::NyxError::internal`] if the Meilisearch HTTP request fails.
    #[instrument(skip(self), fields(index = index_name, id))]
    pub async fn delete_document(&self, index_name: &str, id: &str) -> Result<()> {
        self.client.delete_document(index_name, id).await
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::connect;
    use nun::config::SearchConfig;
    use nun::sensitive::Sensitive;

    fn test_client() -> SearchClient {
        connect(&SearchConfig {
            url: "http://localhost:7700".to_string(),
            api_key: Sensitive::new("test-key".to_string()),
        })
    }

    #[test]
    fn index_sync_new_and_clone() {
        let client = test_client();
        let sync = IndexSync::new(client);
        // Clone should compile and produce an independent handle.
        let _sync2 = sync.clone();
    }

    #[test]
    fn index_sync_debug_does_not_panic() {
        let client = test_client();
        let sync = IndexSync::new(client);
        let _ = format!("{sync:?}");
    }
}
