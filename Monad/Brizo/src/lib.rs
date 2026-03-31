//! # Brizo — Meilisearch search client
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod indexes;

pub use client::{connect, SearchClient};
