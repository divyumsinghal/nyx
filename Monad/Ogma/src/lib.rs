//! # Ogma — Matrix/Continuwuity messaging + privacy enforcement
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod privacy;
pub mod room;

pub use client::MatrixClient;
pub use privacy::PrivacyGuard;
