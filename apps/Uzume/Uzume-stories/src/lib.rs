//! `uzume-stories` — Stories, highlights, and 24-hour TTL service.
//!
//! Service port: 3003
//!
//! ## Module layout
//!
//! - [`config`] — Load [`NyxConfig`] for this service.
//! - [`state`] — Shared [`AppState`] injected into every handler.
//! - [`models`] — Domain types: [`Story`], [`StoryViewer`], [`Highlight`].
//! - [`queries`] — Raw SQL via `sqlx`.
//! - [`services`] — Business logic (pure, no I/O where possible).
//! - [`handlers`] — Axum HTTP handlers.
//! - [`routes`] — Axum router wiring.
//! - [`workers`] — Background tasks (expiry, retention, media-ready).

pub mod config;
pub mod handlers;
pub mod models;
pub mod queries;
pub mod routes;
pub mod services;
pub mod state;
pub mod workers;
