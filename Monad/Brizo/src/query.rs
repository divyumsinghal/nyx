//! Typed search query and response types for Brizo.
//!
//! [`SearchRequest`] describes what to search for and how to filter/paginate
//! results. [`SearchResponse`] carries the typed hits, total estimate, and
//! Meilisearch processing metadata back to the caller.
//!
//! # Example
//!
//! ```rust,ignore
//! use brizo::{SearchClient, query::SearchRequest};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Post { id: String, caption: String }
//!
//! async fn search_posts(client: &SearchClient) -> nun::Result<()> {
//!     let req = SearchRequest {
//!         query: "sunset".to_string(),
//!         limit: 20,
//!         offset: 0,
//!         filter: Some("app = Uzume".to_string()),
//!     };
//!     let resp = client.search::<Post>(brizo::indexes::UZUME_POSTS, req).await?;
//!     println!("{} hits in {}ms", resp.total, resp.processing_time_ms);
//!     Ok(())
//! }
//! ```

use serde::de::DeserializeOwned;
use tracing::instrument;

use nun::Result;

use crate::client::SearchClient;

// ── Request ──────────────────────────────────────────────────────────────────

/// Parameters for a Meilisearch full-text search query.
///
/// All fields are intentionally plain types so callers have no compile-time
/// dependency on the underlying SDK.
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// The search query string. Empty string returns all documents.
    pub query: String,

    /// Maximum number of hits to return. Defaults to [`SearchRequest::DEFAULT_LIMIT`].
    pub limit: usize,

    /// Number of hits to skip (for offset-based pagination).
    pub offset: usize,

    /// Optional Meilisearch filter expression.
    ///
    /// Syntax: `"field = value"`, `"field IN [a, b]"`, etc.
    /// See <https://www.meilisearch.com/docs/learn/advanced/filtering>.
    pub filter: Option<String>,
}

impl SearchRequest {
    /// Default page size used when no explicit limit is given.
    pub const DEFAULT_LIMIT: usize = 20;

    /// Construct a plain text query with no filter and the default limit.
    #[must_use]
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: Self::DEFAULT_LIMIT,
            offset: 0,
            filter: None,
        }
    }

    /// Override the result limit (builder-style).
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Override the offset (builder-style).
    #[must_use]
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Attach a filter expression (builder-style).
    #[must_use]
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self::new("")
    }
}

// ── Response ─────────────────────────────────────────────────────────────────

/// A single document returned in a search result set.
///
/// `T` is the deserialized document type (e.g. a post or profile struct).
#[derive(Debug, Clone)]
pub struct SearchHit<T> {
    /// Meilisearch ranking score in the range `[0.0, 1.0]`. `None` when ranking
    /// score exposure is disabled on the index.
    pub score: Option<f64>,

    /// The fully deserialized document.
    pub document: T,
}

/// The complete typed result of a search query.
#[derive(Debug, Clone)]
pub struct SearchResponse<T> {
    /// Ordered list of matching documents.
    pub hits: Vec<SearchHit<T>>,

    /// Estimated total number of matching documents (not just the current page).
    ///
    /// Meilisearch returns this as `estimated_total_hits`; it may be slightly
    /// off for very large corpora but is accurate for small datasets.
    pub total: usize,

    /// Time taken by Meilisearch to process the query, in milliseconds.
    pub processing_time_ms: u64,
}

// ── SearchClient extension ────────────────────────────────────────────────────

impl SearchClient {
    /// Execute a typed search against the named index.
    ///
    /// # Errors
    ///
    /// Returns [`nun::NyxError::internal`] if the Meilisearch request fails or
    /// the response cannot be deserialized into `T`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let req = SearchRequest::new("hello");
    /// let resp = client.search::<MyDoc>(UZUME_POSTS, req).await?;
    /// ```
    #[instrument(skip(self), fields(index = index_name))]
    pub async fn search<T>(
        &self,
        index_name: &str,
        req: SearchRequest,
    ) -> Result<SearchResponse<T>>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        let index = self.index(index_name);

        let mut query = index.search();
        query.with_query(&req.query);
        query.with_limit(req.limit);
        query.with_offset(req.offset);

        // Stash the filter string so it lives long enough for the query.
        let filter_owned;
        if let Some(ref f) = req.filter {
            filter_owned = f.clone();
            query.with_filter(&filter_owned);
        }

        let raw = query
            .execute::<T>()
            .await
            .map_err(|e| nun::NyxError::internal(format!("meilisearch search failed: {e}")))?;

        let total = raw.estimated_total_hits.unwrap_or(raw.hits.len());

        let hits = raw
            .hits
            .into_iter()
            .map(|h| SearchHit {
                score: h.ranking_score,
                document: h.result,
            })
            .collect();

        Ok(SearchResponse {
            hits,
            total,
            processing_time_ms: raw.processing_time_ms as u64,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_request_new_defaults() {
        let req = SearchRequest::new("sunset");
        assert_eq!(req.query, "sunset");
        assert_eq!(req.limit, SearchRequest::DEFAULT_LIMIT);
        assert_eq!(req.offset, 0);
        assert!(req.filter.is_none());
    }

    #[test]
    fn search_request_default_is_empty_query() {
        let req = SearchRequest::default();
        assert_eq!(req.query, "");
        assert_eq!(req.limit, SearchRequest::DEFAULT_LIMIT);
    }

    #[test]
    fn search_request_builder_chain() {
        let req = SearchRequest::new("cats")
            .with_limit(5)
            .with_offset(10)
            .with_filter("app = Uzume");

        assert_eq!(req.limit, 5);
        assert_eq!(req.offset, 10);
        assert_eq!(req.filter.as_deref(), Some("app = Uzume"));
    }

    #[test]
    fn search_request_with_limit_immutable_original() {
        let original = SearchRequest::new("test");
        let modified = original.clone().with_limit(50);
        // Original is unaffected — we consumed the clone.
        assert_eq!(original.limit, SearchRequest::DEFAULT_LIMIT);
        assert_eq!(modified.limit, 50);
    }

    #[test]
    fn search_hit_carries_document_and_score() {
        let hit: SearchHit<String> = SearchHit {
            score: Some(0.95),
            document: "hello".to_string(),
        };
        assert_eq!(hit.document, "hello");
        assert!((hit.score.unwrap() - 0.95_f64).abs() < f64::EPSILON);
    }

    #[test]
    fn search_response_fields() {
        let resp: SearchResponse<u32> = SearchResponse {
            hits: vec![SearchHit {
                score: None,
                document: 42,
            }],
            total: 1,
            processing_time_ms: 3,
        };
        assert_eq!(resp.hits.len(), 1);
        assert_eq!(resp.total, 1);
        assert_eq!(resp.processing_time_ms, 3);
    }
}
