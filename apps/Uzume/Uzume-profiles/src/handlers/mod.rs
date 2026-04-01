//! HTTP handler functions.
//!
//! Handlers are thin: they extract inputs, call the query + service layers,
//! and return `ApiResponse<T>`. Business logic lives in `services/`.

pub mod follow;
pub mod profile;
