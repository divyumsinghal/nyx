//! # Oya — Media processing pipeline
//!
//! Image and video processing for the Nyx platform.
//!
//! ## Modules
//!
//! - [`config`] — Variant definitions per entity type
//! - [`image`] — Image resizing with fast_image_resize
//! - [`video`] — Video transcoding via FFmpeg
//! - [`pipeline`] — Processing pipeline orchestrator

pub mod config;
pub mod image;
pub mod pipeline;
pub mod video;

pub use config::ProcessingConfig;
pub use pipeline::MediaPipeline;
