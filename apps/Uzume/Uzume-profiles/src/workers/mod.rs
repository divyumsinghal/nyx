//! Background NATS subscriber workers.
//!
//! Each worker module exposes a `run(state: AppState)` async function.
//! `main.rs` spawns each on its own Tokio task after the HTTP server starts.

pub mod profile_stub;
pub mod search_sync;
