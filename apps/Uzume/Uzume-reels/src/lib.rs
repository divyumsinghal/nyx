//! `uzume-reels` — Short-form video, algorithmic feed service.
//!
//! Service port: 3004
//!
//! ## Status
//!
//! This service is currently a stub. Full implementation pending.
//!
//! ## Planned modules
//!
//! - `config` — Load [`NyxConfig`] for this service.
//! - `state` — Shared [`AppState`] injected into every handler.
//! - `models` — Domain types: [`Reel`], [`ReelView`], [`ReelLike`].
//! - `queries` — Raw SQL via `sqlx`.
//! - `services` — Business logic (pure, no I/O where possible).
//! - `handlers` — Axum HTTP handlers.
//! - `routes` — Axum router wiring.
//! - `workers` — Background tasks (video transcoding, feed algorithm).

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(non_snake_case)]

// Stub exports for future implementation
