//! Ushas — Push + in-app notification dispatch for the Nyx platform.
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

pub mod error;
pub mod gorush;
pub mod grouping;
pub mod in_app;
pub mod preferences;
