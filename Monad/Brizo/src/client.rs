//! Meilisearch client.
use meilisearch_sdk::{client::Client, indexes::Index};

use Nun::config::SearchConfig;

/// Thin wrapper around a `meilisearch_sdk::Client`.
#[derive(Clone)]
pub struct SearchClient {
    inner: Client,
}

/// Build a [`SearchClient`] from the given search configuration.
pub fn connect(config: &SearchConfig) -> SearchClient {
    let api_key = config.api_key.expose().clone();
    SearchClient {
        inner: Client::new(config.url.clone(), Some(api_key)),
    }
}

impl SearchClient {
    /// Return the Meilisearch index with the given name.
    pub fn index(&self, name: &str) -> Index {
        self.inner.index(name)
    }
}
