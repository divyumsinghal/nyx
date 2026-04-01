//! Business logic modules.
//!
//! Functions here are pure domain logic with no I/O. They accept already-
//! fetched database rows (or plain Rust values) and return domain results.
//! This makes them straightforward to unit-test without any test fixtures.

pub mod follow;
pub mod profile;
