//! Meilisearch client.
//!
//! Wraps `meilisearch_sdk::Client` with a minimal, Nyx-idiomatic interface.
//! Index management (settings, document upsert/delete) lives here so that
//! every caller shares the same error-mapping conventions.
use meilisearch_sdk::{client::Client, indexes::Index, settings::Settings};
use serde::Serialize;
use tracing::instrument;

use nun::{NyxError, Result};

use nun::config::SearchConfig;

/// Thin wrapper around a `meilisearch_sdk::Client`.
///
/// Cheap to clone — the inner `Client` is `Arc`-backed.
#[derive(Clone, Debug)]
pub struct SearchClient {
    inner: Client,
}

/// Build a [`SearchClient`] from the given search configuration.
///
/// # Panics
///
/// Panics if `config.url` is not a valid URL. This is a configuration error
/// that should be caught at service startup.
#[must_use]
pub fn connect(config: &SearchConfig) -> SearchClient {
    let api_key = config.api_key.expose().clone();
    SearchClient {
        inner: Client::new(config.url.clone(), Some(api_key)).expect("valid meilisearch URL"),
    }
}

impl SearchClient {
    /// Return the raw Meilisearch [`Index`] handle for `name`.
    ///
    /// Prefer the higher-level methods ([`search`](crate::query), [`add_documents`],
    /// [`delete_document`]) over calling methods on the raw index directly.
    #[must_use]
    pub fn index(&self, name: &str) -> Index {
        self.inner.index(name)
    }

    /// Upsert a batch of documents into `index_name`.
    ///
    /// Uses Meilisearch's "add or replace" semantics: existing documents with
    /// the same primary key are overwritten entirely.
    ///
    /// This method enqueues the task but does **not** wait for indexing to
    /// finish — indexing is eventually consistent.
    ///
    /// # Errors
    ///
    /// Returns [`NyxError::internal`] if the HTTP request to Meilisearch fails.
    #[instrument(skip(self, docs), fields(index = index_name, count = docs.len()))]
    pub async fn add_documents<T>(&self, index_name: &str, docs: &[T]) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        self.index(index_name)
            .add_or_replace(docs, None)
            .await
            .map(|_task| ())
            .map_err(|e| NyxError::internal(format!("meilisearch add_documents failed: {e}")))
    }

    /// Delete the document identified by `id` from `index_name`.
    ///
    /// No-op when no document with that ID exists. Enqueues the deletion task
    /// without waiting for it to complete.
    ///
    /// # Errors
    ///
    /// Returns [`NyxError::internal`] if the HTTP request to Meilisearch fails.
    #[instrument(skip(self), fields(index = index_name, id))]
    pub async fn delete_document(&self, index_name: &str, id: &str) -> Result<()> {
        self.index(index_name)
            .delete_document(id)
            .await
            .map(|_task| ())
            .map_err(|e| NyxError::internal(format!("meilisearch delete_document failed: {e}")))
    }

    /// Apply index settings to `index_name`.
    ///
    /// Used during service startup or migration to configure searchable fields,
    /// filterable attributes, etc.
    ///
    /// # Errors
    ///
    /// Returns [`NyxError::internal`] if the HTTP request fails.
    #[instrument(skip(self, settings), fields(index = index_name))]
    pub async fn set_settings(&self, index_name: &str, settings: &Settings) -> Result<()> {
        self.index(index_name)
            .set_settings(settings)
            .await
            .map(|_task| ())
            .map_err(|e| NyxError::internal(format!("meilisearch set_settings failed: {e}")))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nun::sensitive::Sensitive;

    fn make_config(url: &str) -> SearchConfig {
        SearchConfig {
            url: url.to_string(),
            api_key: Sensitive::new("test-key".to_string()),
        }
    }

    #[test]
    fn connect_returns_client() {
        // Should not panic for a syntactically valid URL.
        let _client = connect(&make_config("http://localhost:7700"));
    }

    #[test]
    fn client_is_clone() {
        let client = connect(&make_config("http://localhost:7700"));
        let _clone = client.clone();
    }

    #[test]
    fn index_returns_handle_with_correct_uid() {
        let client = connect(&make_config("http://localhost:7700"));
        let idx = client.index("Uzume_posts");
        assert_eq!(idx.uid, "Uzume_posts");
    }

    #[test]
    fn client_debug_does_not_panic() {
        let client = connect(&make_config("http://localhost:7700"));
        let _ = format!("{client:?}");
    }
}
