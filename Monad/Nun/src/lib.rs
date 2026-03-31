//! # Nun — Nyx Platform Foundation
//!
//! The foundational crate of the Nyx ecosystem. Every crate in the workspace depends on this.
//! Nun provides typed IDs, error handling, configuration, pagination, time utilities,
//! validation, and the core `NyxApp` enum.
//!
//! ## Design principles
//!
//! - **Zero dependencies on other `nyx-*` crates.** Nun is the root of the dependency tree.
//! - **Framework-agnostic.** No Axum, no HTTP types. Those live in `nyx-api`.
//! - **Compile-time safety.** Typed IDs prevent mixing entity types. Non-exhaustive enums
//!   make adding new apps and error kinds non-breaking.
//! - **Minimal features.** Optional `sqlx` and `validator` integration behind feature flags.
//!
//! ## Feature flags
//!
//! - `sqlx` — Enables `sqlx::Type`, `Encode`, `Decode` impls on [`Id<T>`] and
//!   `From<sqlx::Error>` on [`NyxError`].
//! - `validator` — Enables `From<validator::ValidationErrors>` on [`NyxError`] and
//!   validator-compatible function signatures in [`validation`].
//! - `test` — Enables test utilities in [`testing`].

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod app;
pub mod config;
pub mod error;
pub mod id;
pub mod pagination;
pub mod sensitive;
pub mod time;
pub mod validation;

#[cfg(any(test, feature = "test"))]
pub mod testing;

// ── Prelude-style re-exports ────────────────────────────────────────────────
// The most commonly used types, importable with `use nun::*` or individually.

pub use app::NyxApp;
pub use error::{ErrorKind, NyxError, Result};
pub use id::Id;
pub use pagination::{Cursor, PageRequest, PageResponse};
pub use sensitive::Sensitive;
pub use time::Timestamp;

// Platform entity markers and their ID type aliases.
pub use id::IdentityId;
pub use id::entity;
