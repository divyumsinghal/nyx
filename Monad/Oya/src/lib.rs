//! # Oya — Media processing pipeline
//!
//! Image and video processing for the Nyx platform.
//!
//! ## Modules
//!
//! - [`config`] — Variant definitions per entity type
//! - [`image`] — Image resizing with fast_image_resize
//! - [`video`] — Video transcoding via FFmpeg (shell-out)
//! - [`pipeline`] — Processing pipeline orchestrator
//! - [`events`] — NATS event types for media upload/processed lifecycle
//!
//! ## Example
//!
//! ```rust,ignore
//! use oya::{MediaPipeline, ProcessingConfig};
//! use oya::pipeline::MediaJob;
//!
//! let config = ProcessingConfig::default();
//! let pipeline = MediaPipeline::new(config);
//! let result = pipeline.process_image(
//!     MediaJob {
//!         job_id: uuid::Uuid::now_v7(),
//!         entity_type: "story",
//!         entity_id: "abc",
//!         mime_type: "image/jpeg",
//!     },
//!     &image_bytes,
//! )?;
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod config;
pub mod events;
pub mod image;
pub mod pipeline;
pub mod video;

pub use config::ProcessingConfig;
pub use pipeline::MediaPipeline;
