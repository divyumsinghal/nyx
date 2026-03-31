//! # Mnemosyne — PostgreSQL pool + migration runner
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod ext;
pub mod migrate;
pub mod pool;
pub mod transaction;

pub use pool::{connect, DbPool};
pub use transaction::Transaction;
