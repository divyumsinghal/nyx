//! # Heka — Kratos identity client + app-scoped alias system
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod alias;
pub mod client;
pub mod identity;
pub mod jwt;

pub use alias::AliasResolver;
pub use client::KratosClient;
pub use identity::{Identity, Session};
pub use jwt::{validate_jwt, Claims};
