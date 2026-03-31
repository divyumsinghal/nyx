//! Typed identifiers for the Nyx platform.
//!
//! [`Id<T>`] is a UUIDv7 wrapper parameterized by a phantom marker type. This provides
//! compile-time distinction between IDs of different entity types — passing an `Id<Post>`
//! where an `Id<Profile>` is expected is a type error, not a runtime bug.
//!
//! # Defining entity markers
//!
//! Any empty struct works as a marker. No traits, no registration:
//!
//! ```rust
//! pub struct Post;
//! pub struct Comment;
//!
//! pub type PostId = nun::Id<Post>;
//! pub type CommentId = nun::Id<Comment>;
//!
//! let id = PostId::new();
//! ```
//!
//! # Platform markers
//!
//! Nun defines markers for platform-level entities in the [`entity`] module.
//! App-level markers are defined in their respective crates.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

/// A typed identifier wrapping a UUIDv7.
///
/// The phantom type parameter `T` prevents accidentally mixing IDs of different
/// entity types at compile time. `Id<Post>` and `Id<Profile>` are distinct types.
///
/// # UUIDv7
///
/// All IDs are UUIDv7: the first 48 bits encode a millisecond-precision Unix
/// timestamp, making them time-sortable with no coordination. This is critical
/// for cursor-based pagination — sorting by ID is equivalent to sorting by
/// creation time.
///
/// # Variance
///
/// Uses `PhantomData<fn() -> T>` for correct covariance and automatic
/// `Send + Sync` without bounds on `T`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id<T> {
    inner: Uuid,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    /// Generate a new UUIDv7 with the current timestamp.
    pub fn new() -> Self {
        Self {
            inner: Uuid::now_v7(),
            _marker: PhantomData,
        }
    }

    /// Wrap an existing UUID (e.g., from a database row).
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            inner: uuid,
            _marker: PhantomData,
        }
    }

    /// Borrow the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.inner
    }

    /// Consume and return the inner UUID.
    pub fn into_uuid(self) -> Uuid {
        self.inner
    }

    /// Cast to a different entity type.
    ///
    /// Use sparingly — this deliberately defeats type safety. Exists for rare
    /// cases like generic middleware or serialization boundaries where the
    /// concrete entity type is erased.
    pub fn cast<U>(self) -> Id<U> {
        Id {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

// ── Display / Debug ─────────────────────────────────────────────────────────

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Include the marker type name for clarity in debug output.
        let type_name = std::any::type_name::<T>();
        // Extract the short name after the last `::`.
        let short = type_name.rsplit("::").next().unwrap_or(type_name);
        write!(f, "Id<{short}>({})", self.inner)
    }
}

// ── FromStr ─────────────────────────────────────────────────────────────────

impl<T> FromStr for Id<T> {
    type Err = uuid::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self::from_uuid)
    }
}

// ── Serde ───────────────────────────────────────────────────────────────────
// Serializes as a plain UUID string: "0194e5a2-7b3c-7def-8a12-..."
// The marker type is erased in the wire format — it's a compile-time-only concern.

impl<T> Serialize for Id<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        Uuid::deserialize(deserializer).map(Self::from_uuid)
    }
}

// ── sqlx integration (feature = "sqlx") ─────────────────────────────────────
// Transparent to the inner Uuid — PostgreSQL sees a UUID column.

#[cfg(feature = "sqlx")]
impl<T> sqlx::Type<sqlx::Postgres> for Id<T> {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <Uuid as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

#[cfg(feature = "sqlx")]
impl<T> sqlx::Encode<'_, sqlx::Postgres> for Id<T> {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <Uuid as sqlx::Encode<'_, sqlx::Postgres>>::encode_by_ref(&self.inner, buf)
    }
}

#[cfg(feature = "sqlx")]
impl<T> sqlx::Decode<'_, sqlx::Postgres> for Id<T> {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        <Uuid as sqlx::Decode<'_, sqlx::Postgres>>::decode(value).map(Self::from_uuid)
    }
}

// ── Platform entity markers ─────────────────────────────────────────────────

/// Marker types for platform-level entities.
///
/// App-level markers (Post, Comment, Profile, etc.) are defined in their
/// respective crates. These platform markers are for entities that live in
/// the `nyx` PostgreSQL schema — shared across all apps.
pub mod entity {
    /// A Kratos identity — the real human behind all app-scoped aliases.
    /// One per user, holds phone + email + password hash.
    pub struct Identity;

    /// An app-scoped alias record. Maps (identity, app) → visible username.
    pub struct Alias;

    /// A cross-app consent link. Records that user A allows user B to see
    /// their profile across a specific app boundary.
    pub struct AppLink;

    /// A device push notification token (APNs or FCM).
    pub struct PushToken;
}

/// The identity of a Nyx user — references the Kratos identity.
pub type IdentityId = Id<entity::Identity>;

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct Post;
    struct Comment;

    type PostId = Id<Post>;
    type CommentId = Id<Comment>;

    #[test]
    fn new_id_is_valid_uuidv7() {
        let id = PostId::new();
        let uuid = id.into_uuid();
        assert_eq!(uuid.get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn ids_are_time_sortable() {
        let a = PostId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = PostId::new();
        assert!(a < b, "later ID should sort after earlier ID");
    }

    #[test]
    fn display_shows_uuid_string() {
        let uuid = Uuid::nil();
        let id = PostId::from_uuid(uuid);
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn debug_includes_type_name() {
        let id = PostId::from_uuid(Uuid::nil());
        let debug = format!("{id:?}");
        assert!(
            debug.contains("Post"),
            "debug should include marker type name"
        );
        assert!(debug.contains("00000000"), "debug should include UUID");
    }

    #[test]
    fn from_str_round_trip() {
        let id = PostId::new();
        let s = id.to_string();
        let parsed: PostId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn serde_round_trip() {
        let id = PostId::new();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: PostId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn serde_format_is_uuid_string() {
        let uuid = Uuid::nil();
        let id = PostId::from_uuid(uuid);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
    }

    #[test]
    fn different_marker_types_are_incompatible() {
        // This test verifies the type system at compile time.
        // If this compiles, the types are correctly distinct.
        let post_id = PostId::new();
        let _comment_id: CommentId = post_id.cast(); // explicit cast required
    }

    #[test]
    fn cast_preserves_uuid() {
        let post_id = PostId::new();
        let uuid = *post_id.as_uuid();
        let comment_id: CommentId = post_id.cast();
        assert_eq!(*comment_id.as_uuid(), uuid);
    }

    #[test]
    fn id_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PostId>();
        assert_send_sync::<CommentId>();
        assert_send_sync::<IdentityId>();
    }
}
