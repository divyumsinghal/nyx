//! Database model types — sqlx `FromRow` structs and insert/update payloads.
//!
//! These types are the direct representation of what the database stores.
//! They are only used inside the `queries` layer and are never exposed
//! through the HTTP layer directly.

pub mod follow;
pub mod profile;

pub use follow::{FollowProfileRow, FollowRow, FollowStatus};
pub use profile::{ProfileInsert, ProfileRow, ProfileUpdate};
