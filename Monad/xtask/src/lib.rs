//! `nyx-xtask` library — public API re-exported so integration tests can call
//! command implementations directly without going through the binary entry point.
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod commands;
pub mod env;

pub mod auth;
