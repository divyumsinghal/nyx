//! Cursor-based pagination for the Nyx platform.
//!
//! All list endpoints use cursor-based pagination (never offset-based). Cursors
//! are opaque strings that clients pass back to get the next page. Internally,
//! they are MessagePack-encoded + base64url-encoded byte arrays.
//!
//! # The "fetch one extra" pattern
//!
//! Every list query should fetch `limit + 1` rows. If the query returns more
//! than `limit` rows, there are more pages. The extra row is discarded and the
//! last kept row is used to build the next cursor.
//!
//! [`PageResponse::from_overflowed`] encapsulates this pattern.
//!
//! # Cursor types
//!
//! Different endpoints sort by different keys, so the cursor format must be
//! flexible:
//!
//! - **Timestamp + ID** — most common: feeds, comments, notifications
//! - **Score + ID** — algorithmic feeds: reels, discover
//! - **Distance + ID** — geo-sorted: Themis listings, Aengus profiles
//! - **Arbitrary** — escape hatch via [`CursorValue`]

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{NyxError, Result};

// ── Cursor ──────────────────────────────────────────────────────────────────

/// An opaque pagination cursor.
///
/// Clients receive cursors as base64url strings and pass them back verbatim.
/// The internal format is MessagePack-encoded bytes — clients cannot (and
/// should not) parse or construct cursors.
#[derive(Debug, Clone)]
pub struct Cursor(Vec<u8>);

impl Cursor {
    /// Encode the cursor to a URL-safe base64 string for API responses.
    pub fn encode(&self) -> String {
        URL_SAFE_NO_PAD.encode(&self.0)
    }

    /// Decode a cursor from the base64url string passed by the client.
    pub fn decode(s: &str) -> Result<Self> {
        let bytes = URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|_| NyxError::bad_request("invalid_cursor", "Invalid pagination cursor"))?;
        Ok(Self(bytes))
    }

    // ── Timestamp + ID cursor (most common) ─────────────────────────────

    /// Create a cursor from a timestamp and entity ID.
    ///
    /// Used by: home feed, comments, notifications, followers — any endpoint
    /// sorted by `created_at DESC`.
    pub fn timestamp_id(ts: DateTime<Utc>, id: Uuid) -> Self {
        let value = (ts.timestamp_millis(), id.as_bytes().as_slice());
        let bytes = rmp_serde::to_vec(&value).expect("cursor serialization is infallible");
        Self(bytes)
    }

    /// Extract a timestamp + ID from a cursor.
    pub fn as_timestamp_id(&self) -> Result<(DateTime<Utc>, Uuid)> {
        let (millis, id_bytes): (i64, Vec<u8>) = rmp_serde::from_slice(&self.0)
            .map_err(|_| NyxError::bad_request("invalid_cursor", "Malformed cursor data"))?;

        let ts = DateTime::from_timestamp_millis(millis).ok_or_else(|| {
            NyxError::bad_request("invalid_cursor", "Invalid timestamp in cursor")
        })?;

        let id = Uuid::from_slice(&id_bytes)
            .map_err(|_| NyxError::bad_request("invalid_cursor", "Invalid ID in cursor"))?;

        Ok((ts, id))
    }

    // ── Score + ID cursor (algorithmic feeds) ───────────────────────────

    /// Create a cursor from a score and entity ID.
    ///
    /// Used by: reels feed, discover page — any endpoint sorted by an
    /// engagement/relevance score.
    pub fn score_id(score: f64, id: Uuid) -> Self {
        let value = (score.to_bits(), id.as_bytes().as_slice());
        let bytes = rmp_serde::to_vec(&value).expect("cursor serialization is infallible");
        Self(bytes)
    }

    /// Extract a score + ID from a cursor.
    pub fn as_score_id(&self) -> Result<(f64, Uuid)> {
        let (score_bits, id_bytes): (u64, Vec<u8>) = rmp_serde::from_slice(&self.0)
            .map_err(|_| NyxError::bad_request("invalid_cursor", "Malformed cursor data"))?;

        let score = f64::from_bits(score_bits);

        let id = Uuid::from_slice(&id_bytes)
            .map_err(|_| NyxError::bad_request("invalid_cursor", "Invalid ID in cursor"))?;

        Ok((score, id))
    }

    // ── Distance + ID cursor (geo-sorted) ───────────────────────────────

    /// Create a cursor from a distance (meters) and entity ID.
    ///
    /// Used by: Themis listing search, Aengus profile discovery — any endpoint
    /// sorted by geographic distance.
    pub fn distance_id(distance_meters: f64, id: Uuid) -> Self {
        // Reuse the score encoding — both are f64 + UUID.
        Self::score_id(distance_meters, id)
    }

    /// Extract a distance + ID from a cursor.
    pub fn as_distance_id(&self) -> Result<(f64, Uuid)> {
        self.as_score_id()
    }

    // ── Generic cursor (escape hatch) ───────────────────────────────────

    /// Create a cursor from arbitrary key-value pairs.
    ///
    /// Use this when the standard cursor types don't fit. The values are
    /// serialized to MessagePack and can be extracted with [`as_values`](Self::as_values).
    pub fn from_values(values: &[CursorValue]) -> Self {
        let bytes = rmp_serde::to_vec(values).expect("cursor serialization is infallible");
        Self(bytes)
    }

    /// Extract the arbitrary values from a generic cursor.
    pub fn as_values(&self) -> Result<Vec<CursorValue>> {
        rmp_serde::from_slice(&self.0)
            .map_err(|_| NyxError::bad_request("invalid_cursor", "Malformed cursor data"))
    }
}

/// A value that can be stored in a generic cursor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CursorValue {
    /// A UUID (typically an entity ID).
    Uuid(Uuid),
    /// A millisecond-precision UTC timestamp.
    TimestampMillis(i64),
    /// A floating-point value (score, distance, etc.).
    Float(f64),
    /// An integer value.
    Int(i64),
    /// A string value.
    String(String),
}

// ── PageRequest ─────────────────────────────────────────────────────────────

/// Incoming pagination parameters from query string.
///
/// ```ignore
/// GET /api/uzume/feed/posts?cursor=abc123&limit=20
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct PageRequest {
    /// The opaque cursor string from a previous response. `None` for the first page.
    pub cursor: Option<String>,

    /// Requested page size. Clamped to 1..=100, defaults to 20.
    #[serde(default = "default_limit")]
    pub limit: u16,
}

fn default_limit() -> u16 {
    20
}

/// Minimum page size.
const MIN_LIMIT: u16 = 1;
/// Maximum page size.
const MAX_LIMIT: u16 = 100;

impl PageRequest {
    /// Decode the cursor string, if present.
    pub fn decode_cursor(&self) -> Result<Option<Cursor>> {
        self.cursor.as_deref().map(Cursor::decode).transpose()
    }

    /// The effective page size, clamped to the allowed range.
    pub fn effective_limit(&self) -> u16 {
        self.limit.clamp(MIN_LIMIT, MAX_LIMIT)
    }

    /// The number of rows to fetch from the database: `effective_limit + 1`.
    ///
    /// The extra row is used to determine whether more pages exist.
    /// Pass this to your SQL `LIMIT` clause.
    pub fn query_limit(&self) -> i64 {
        i64::from(self.effective_limit()) + 1
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            cursor: None,
            limit: default_limit(),
        }
    }
}

// ── PageResponse ────────────────────────────────────────────────────────────

/// Outgoing paginated response.
///
/// The `items` field contains the page data. `next_cursor` is `Some` if there
/// are more pages; clients pass it back as the `cursor` query parameter.
#[derive(Debug, Clone, Serialize)]
pub struct PageResponse<T: Serialize> {
    /// The items in this page.
    pub items: Vec<T>,

    /// Cursor for the next page. `None` if this is the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,

    /// Whether more pages exist after this one.
    pub has_more: bool,
}

impl<T: Serialize> PageResponse<T> {
    /// Create a page response directly.
    pub fn new(items: Vec<T>, next_cursor: Option<Cursor>, has_more: bool) -> Self {
        Self {
            items,
            next_cursor: next_cursor.map(|c| c.encode()),
            has_more,
        }
    }

    /// Create an empty response (no items, no more pages).
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_cursor: None,
            has_more: false,
        }
    }

    /// Build a page response from the "fetch one extra" pattern.
    ///
    /// # How it works
    ///
    /// 1. Your query fetches `limit + 1` rows (use [`PageRequest::query_limit`]).
    /// 2. Pass all returned rows to this method along with the `limit`.
    /// 3. If there are more rows than `limit`, the extra is removed and
    ///    `has_more` is `true`.
    /// 4. The `cursor_fn` builds a cursor from the last kept item.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let rows = sqlx::query_as!(Post, "SELECT * FROM posts ORDER BY created_at DESC LIMIT $1", page.query_limit())
    ///     .fetch_all(&pool)
    ///     .await?;
    ///
    /// Ok(PageResponse::from_overflowed(rows, page.effective_limit(), |post| {
    ///     Cursor::timestamp_id(post.created_at, post.id.into_uuid())
    /// }))
    /// ```
    pub fn from_overflowed(
        mut items: Vec<T>,
        limit: u16,
        cursor_fn: impl Fn(&T) -> Cursor,
    ) -> Self {
        let limit = limit as usize;
        let has_more = items.len() > limit;

        if has_more {
            items.truncate(limit);
        }

        let next_cursor = if has_more {
            items.last().map(|item| cursor_fn(item).encode())
        } else {
            None
        };

        Self {
            items,
            next_cursor,
            has_more,
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::now;

    #[test]
    fn timestamp_id_cursor_round_trip() {
        let ts = now();
        let id = Uuid::now_v7();
        let cursor = Cursor::timestamp_id(ts, id);

        let encoded = cursor.encode();
        let decoded = Cursor::decode(&encoded).unwrap();
        let (ts2, id2) = decoded.as_timestamp_id().unwrap();

        assert_eq!(ts.timestamp_millis(), ts2.timestamp_millis());
        assert_eq!(id, id2);
    }

    #[test]
    fn score_id_cursor_round_trip() {
        let score = 42.5;
        let id = Uuid::now_v7();
        let cursor = Cursor::score_id(score, id);

        let encoded = cursor.encode();
        let decoded = Cursor::decode(&encoded).unwrap();
        let (score2, id2) = decoded.as_score_id().unwrap();

        assert!((score - score2).abs() < f64::EPSILON);
        assert_eq!(id, id2);
    }

    #[test]
    fn generic_cursor_round_trip() {
        let values = vec![
            CursorValue::Int(42),
            CursorValue::Float(std::f64::consts::PI),
            CursorValue::String("hello".to_string()),
        ];
        let cursor = Cursor::from_values(&values);

        let encoded = cursor.encode();
        let decoded = Cursor::decode(&encoded).unwrap();
        let values2 = decoded.as_values().unwrap();

        assert_eq!(values2.len(), 3);
        match &values2[0] {
            CursorValue::Int(v) => assert_eq!(*v, 42),
            _ => panic!("expected Int"),
        }
    }

    #[test]
    fn invalid_cursor_returns_bad_request() {
        let err = Cursor::decode("not-valid-base64!!!").unwrap_err();
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.code(), "invalid_cursor");
    }

    #[test]
    fn page_request_clamps_limit() {
        let req = PageRequest {
            cursor: None,
            limit: 0,
        };
        assert_eq!(req.effective_limit(), 1);

        let req = PageRequest {
            cursor: None,
            limit: 500,
        };
        assert_eq!(req.effective_limit(), 100);

        let req = PageRequest {
            cursor: None,
            limit: 50,
        };
        assert_eq!(req.effective_limit(), 50);
    }

    #[test]
    fn query_limit_is_effective_plus_one() {
        let req = PageRequest {
            cursor: None,
            limit: 20,
        };
        assert_eq!(req.query_limit(), 21);
    }

    #[test]
    fn from_overflowed_with_more_data() {
        let items: Vec<i32> = (0..21).collect(); // 21 items, limit is 20
        let page = PageResponse::from_overflowed(items, 20, |item| {
            Cursor::score_id(f64::from(*item), Uuid::nil())
        });

        assert_eq!(page.items.len(), 20);
        assert!(page.has_more);
        assert!(page.next_cursor.is_some());
    }

    #[test]
    fn from_overflowed_at_end() {
        let items: Vec<i32> = (0..15).collect(); // 15 items, limit is 20
        let page = PageResponse::from_overflowed(items, 20, |item| {
            Cursor::score_id(f64::from(*item), Uuid::nil())
        });

        assert_eq!(page.items.len(), 15);
        assert!(!page.has_more);
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn from_overflowed_exactly_at_limit() {
        let items: Vec<i32> = (0..20).collect(); // exactly 20 items, limit 20
        let page = PageResponse::from_overflowed(items, 20, |item| {
            Cursor::score_id(f64::from(*item), Uuid::nil())
        });

        assert_eq!(page.items.len(), 20);
        assert!(!page.has_more); // exactly at limit means no overflow
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn empty_response() {
        let page: PageResponse<()> = PageResponse::empty();
        assert!(page.items.is_empty());
        assert!(!page.has_more);
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn page_response_serializes_correctly() {
        let page = PageResponse {
            items: vec![1, 2, 3],
            next_cursor: Some("abc123".to_string()),
            has_more: true,
        };
        let json = serde_json::to_value(&page).unwrap();
        assert_eq!(json["items"], serde_json::json!([1, 2, 3]));
        assert_eq!(json["next_cursor"], "abc123");
        assert_eq!(json["has_more"], true);
    }

    #[test]
    fn page_response_omits_null_cursor() {
        let page = PageResponse {
            items: vec![1],
            next_cursor: None,
            has_more: false,
        };
        let json = serde_json::to_value(&page).unwrap();
        assert!(json.get("next_cursor").is_none());
    }
}
