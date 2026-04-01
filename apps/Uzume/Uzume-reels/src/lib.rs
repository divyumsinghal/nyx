//! `uzume-reels` — Short-form video, algorithmic feed service.
//!
//! Service port: 3004
//!
//! ## Module layout
//!
//! - [`config`] — Load [`NyxConfig`] for this service.
//! - [`state`] — Shared [`AppState`] injected into every handler.
//! - [`models`] — Domain types: [`models::reel::ReelRow`], [`models::reel_audio::ReelAudioRow`].
//! - [`queries`] — Raw SQL via `sqlx`.
//! - [`services`] — Business logic (pure, no I/O where possible).
//! - [`handlers`] — Axum HTTP handlers.
//! - [`routes`] — Axum router wiring.
//! - [`workers`] — Background tasks (search sync, score updater).

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(non_snake_case)]
#![allow(clippy::cast_precision_loss)]

pub mod config;
pub mod handlers;
pub mod models;
pub mod queries;
pub mod routes;
pub mod services;
pub mod state;
pub mod workers;
