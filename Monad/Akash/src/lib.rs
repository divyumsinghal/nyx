//! # Akash — MinIO/S3 object storage client
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod paths;
pub mod presigned;

pub use client::{connect, StorageClient};
pub use paths::StoragePath;
