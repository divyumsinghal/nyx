//! Unified test harness for Nyx platform and applications.
//!
//! This crate provides:
//! - Common test fixtures and factories
//! - Mock implementations for external services
//! - Property-based testing generators
//! - E2E sandbox infrastructure (testcontainers)
//! - Security test utilities
//! - Custom assertions and helpers
//!
//! # Organization
//!
//! - `common`: Shared test utilities and helpers
//! - `fixtures`: Pre-built test data and factories
//! - `mocks`: Mock implementations of platform services
//! - `generators`: Property-based test generators (proptest, quickcheck, arbitrary)
//! - `builders`: Fluent builders for test data
//! - `asserts`: Custom assertion macros and helpers
//! - `sandbox`: E2E infrastructure orchestration (testcontainers)
//! - `security`: Security testing utilities and payloads

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod asserts;
pub mod builders;
pub mod common;
pub mod fixtures;
pub mod generators;
pub mod mocks;
pub mod sandbox;
pub mod security;

// Re-export commonly used testing utilities
pub use asserts::*;
pub use builders::*;
pub use common::*;
pub use fixtures::*;
pub use generators::*;
pub use nun::testing::*;

// Re-export testing dependencies for convenience
pub use httptest;
pub use pretty_assertions::{assert_eq, assert_ne};
pub use proptest;
pub use quickcheck;
pub use rstest;
pub use serial_test::serial;
pub use test_case::test_case;
pub use testcontainers;
