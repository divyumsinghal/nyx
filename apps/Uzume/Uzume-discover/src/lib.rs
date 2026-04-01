//! `uzume-discover` — Explore page, trending, and search service.
//!
//! Service port: 3005
//!
//! ## Status
//!
//! This service is currently a stub. Full implementation pending.
//!
//! ## Planned modules
//!
//! - `config` — Load [`NyxConfig`] for this service.
//! - `state` — Shared [`AppState`] injected into every handler.
//! - `models` — Domain types: [`TrendingPost`], [`SearchResult`].
//! - `queries` — Meilisearch search queries.
//! - `services` — Business logic (trending algorithm, search ranking).
//! - `handlers` — Axum HTTP handlers.
//! - `routes` — Axum router wiring.
//! - `workers` — Background tasks (trending calculation, search index sync).

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(non_snake_case)]

pub mod config;
pub mod handlers;
pub mod models;
pub mod queries;
pub mod routes;
pub mod services;
pub mod state;
pub mod workers;
