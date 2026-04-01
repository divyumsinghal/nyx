//! Raw SQL queries for the `uzume-stories` service.
//!
//! All functions use `sqlx::query_as` with explicit type annotations. This
//! avoids requiring a live database connection at build time. Use
//! `SQLX_OFFLINE=true` with a populated `.sqlx/` cache in CI, or run
//! `cargo sqlx prepare` before building.

pub mod highlights;
pub mod stories;
pub mod viewers;
