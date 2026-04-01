//! Media processing pipeline orchestrator.
//!
//! [`MediaPipeline`] is the central entry-point for Oya. Given a [`MediaJob`]
//! (entity type, mime type, raw data or file path), it validates inputs, selects
//! the right processing path (image vs video), runs the appropriate processor,
//! and returns a structured [`PipelineResult`].

use std::time::Instant;

use thiserror::Error;

use crate::config::{EntityConfig, ProcessingConfig};
use crate::image::{self, VariantResult as ImageVariantResult};
use crate::video::{self, VideoProcessingResult};

/// Errors that can be returned from the pipeline.
#[derive(Debug, Error)]
pub enum PipelineError {
    /// The `entity_type` field of the job has no configured variants.
    #[error("unknown entity type: {0}")]
    UnknownEntity(String),

    /// The image processor returned an error.
    #[error("image processing failed: {0}")]
    Image(#[from] image::ImageError),

    /// The video processor returned an error.
    #[error("video processing failed: {0}")]
    Video(#[from] video::VideoError),

    /// The MIME type is not in the allow-list for this entity.
    #[error("unsupported mime type: {0}")]
    UnsupportedMimeType(String),

    /// The file exceeds the configured size limit.
    #[error("file too large: {size} bytes exceeds {max} bytes")]
    FileTooLarge {
        /// Actual file size in bytes.
        size: u64,
        /// Configured limit in bytes.
        max: u64,
    },
}

/// Lifecycle state of a media processing job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessingState {
    /// Job has been accepted and is queued.
    Accepted,
    /// Job is actively being processed.
    Processing,
    /// All variants are ready and uploaded.
    Ready,
    /// Processing failed; error description included.
    Failed(String),
}

/// Result of processing an image through the pipeline.
#[derive(Debug)]
pub struct ImagePipelineResult {
    /// The job ID for correlation.
    pub job_id: uuid::Uuid,
    /// Entity type, e.g. `"story"`.
    pub entity_type: String,
    /// Entity ID the media belongs to.
    pub entity_id: String,
    /// All produced image variants.
    pub variants: Vec<ImageVariantResult>,
    /// Wall-clock processing duration.
    pub processing_ms: u64,
}

/// Result of processing a video through the pipeline.
#[derive(Debug)]
pub struct VideoPipelineResult {
    /// The job ID for correlation.
    pub job_id: uuid::Uuid,
    /// Entity type.
    pub entity_type: String,
    /// Entity ID.
    pub entity_id: String,
    /// Full HLS output including all variant playlists and poster frame.
    pub video_result: VideoProcessingResult,
    /// Wall-clock processing duration.
    pub processing_ms: u64,
}

/// Unified result of any media processing job.
#[derive(Debug)]
pub enum PipelineResult {
    /// An image was processed.
    Image(ImagePipelineResult),
    /// A video was processed.
    Video(VideoPipelineResult),
}

impl PipelineResult {
    /// The job ID that produced this result.
    pub fn job_id(&self) -> uuid::Uuid {
        match self {
            Self::Image(r) => r.job_id,
            Self::Video(r) => r.job_id,
        }
    }

    /// Entity type string.
    pub fn entity_type(&self) -> &str {
        match self {
            Self::Image(r) => &r.entity_type,
            Self::Video(r) => &r.entity_type,
        }
    }

    /// Entity ID string.
    pub fn entity_id(&self) -> &str {
        match self {
            Self::Image(r) => &r.entity_id,
            Self::Video(r) => &r.entity_id,
        }
    }

    /// Processing duration in milliseconds.
    pub fn processing_ms(&self) -> u64 {
        match self {
            Self::Image(r) => r.processing_ms,
            Self::Video(r) => r.processing_ms,
        }
    }
}

/// Identifies a single media processing job.
///
/// Carries the minimal metadata required to route to the right processor and
/// validate against the entity configuration.
#[derive(Debug, Clone, Copy)]
pub struct MediaJob<'a> {
    /// Unique job ID (UUIDv7) for idempotency tracking.
    pub job_id: uuid::Uuid,
    /// Entity type key, e.g. `"story"`, `"post"`, `"reel"`.
    pub entity_type: &'a str,
    /// Entity ID the media belongs to.
    pub entity_id: &'a str,
    /// MIME type of the raw upload, e.g. `"image/jpeg"`, `"video/mp4"`.
    pub mime_type: &'a str,
}

/// Media processing pipeline orchestrator.
///
/// Validates jobs, dispatches to the image or video processor, and wraps
/// results in the unified [`PipelineResult`] type.
///
/// # Example
///
/// ```rust,ignore
/// use oya::{MediaPipeline, ProcessingConfig};
/// use oya::pipeline::MediaJob;
///
/// let pipeline = MediaPipeline::new(ProcessingConfig::default());
/// let result = pipeline.process_image(
///     MediaJob { job_id: uuid::Uuid::now_v7(), entity_type: "story",
///                entity_id: "xyz", mime_type: "image/jpeg" },
///     &image_bytes,
/// )?;
/// ```
pub struct MediaPipeline {
    config: ProcessingConfig,
}

impl MediaPipeline {
    /// Create a new pipeline with the given configuration.
    pub fn new(config: ProcessingConfig) -> Self {
        Self { config }
    }

    /// Process raw image bytes through all configured variants for the given entity.
    ///
    /// # Errors
    ///
    /// Returns a [`PipelineError`] if the entity type is unknown, the MIME type
    /// is not allowed, the data exceeds the size limit, or image processing fails.
    pub fn process_image(
        &self,
        job: MediaJob<'_>,
        data: &[u8],
    ) -> Result<ImagePipelineResult, PipelineError> {
        let entity = self.resolve_entity(job.entity_type)?;
        validate_mime_type(job.mime_type, entity)?;
        validate_size(data.len() as u64, entity.max_image_size_bytes)?;

        let start = Instant::now();
        let variants = image::process_image_to_variants(data, &entity.image_variants)?;
        let processing_ms = start.elapsed().as_millis() as u64;

        Ok(ImagePipelineResult {
            job_id: job.job_id,
            entity_type: job.entity_type.to_string(),
            entity_id: job.entity_id.to_string(),
            variants,
            processing_ms,
        })
    }

    /// Process a video file (at `input_path`) through all configured HLS variants.
    ///
    /// FFmpeg is invoked for each variant and for poster extraction. Output files
    /// are written to `output_dir`.
    ///
    /// # Errors
    ///
    /// Returns a [`PipelineError`] if the entity type is unknown, the MIME type
    /// is not allowed, the file is too large, or FFmpeg processing fails.
    pub fn process_video(
        &self,
        job: MediaJob<'_>,
        input_path: &std::path::Path,
        output_dir: &std::path::Path,
    ) -> Result<VideoPipelineResult, PipelineError> {
        let entity = self.resolve_entity(job.entity_type)?;
        validate_mime_type(job.mime_type, entity)?;

        let file_size = std::fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
        validate_size(file_size, entity.max_video_size_bytes)?;

        let start = Instant::now();
        let video_result = video::process_video(
            &self.config.ffmpeg_path,
            input_path,
            output_dir,
            &entity.video_variants,
        )?;
        let processing_ms = start.elapsed().as_millis() as u64;

        Ok(VideoPipelineResult {
            job_id: job.job_id,
            entity_type: job.entity_type.to_string(),
            entity_id: job.entity_id.to_string(),
            video_result,
            processing_ms,
        })
    }

    fn resolve_entity<'c>(
        &'c self,
        entity_type: &str,
    ) -> Result<&'c EntityConfig, PipelineError> {
        self.config
            .get_entity(entity_type)
            .ok_or_else(|| PipelineError::UnknownEntity(entity_type.to_string()))
    }
}

// ── Validation helpers ────────────────────────────────────────────────────────

fn validate_mime_type(mime_type: &str, entity: &EntityConfig) -> Result<(), PipelineError> {
    let normalized = mime_type.to_lowercase();
    let normalized = normalized.trim();
    if entity.allowed_mime_types.iter().any(|m| m == normalized) {
        Ok(())
    } else {
        Err(PipelineError::UnsupportedMimeType(mime_type.to_string()))
    }
}

fn validate_size(size: u64, max: u64) -> Result<(), PipelineError> {
    if size > max {
        Err(PipelineError::FileTooLarge { size, max })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EntityConfig, ImageFormat, ImageVariant, VideoVariant};

    fn make_test_config() -> ProcessingConfig {
        let mut config = ProcessingConfig::default();
        config.entities.insert(
            "test".to_string(),
            EntityConfig {
                entity_type: "test".to_string(),
                image_variants: vec![ImageVariant {
                    name: "100".to_string(),
                    max_width: 100,
                    max_height: 100,
                    format: ImageFormat::Jpeg(85),
                }],
                video_variants: vec![VideoVariant {
                    name: "360p".to_string(),
                    resolution: (360, 640),
                    video_bitrate: "800k".to_string(),
                    audio_bitrate: "64k".to_string(),
                }],
                max_image_size_bytes: 1024 * 1024,
                max_video_size_bytes: 100 * 1024 * 1024,
                allowed_mime_types: vec!["image/jpeg".to_string(), "video/mp4".to_string()],
            },
        );
        config
    }

    fn test_jpeg_bytes() -> Vec<u8> {
        use std::io::Cursor;
        let img = image::DynamicImage::new_rgb8(50, 50);
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
            .unwrap();
        buf
    }

    // ── test_media_job_deserialization (via JSON round-trip of events) ────────

    #[test]
    fn test_media_job_deserialization() {
        // MediaJob is a struct with lifetime parameters (no serde), but the
        // underlying payload struct MediaUploadedPayload is serde.
        use crate::events::MediaUploadedPayload;
        let json = r#"{
            "job_id": "018e6b3a-9f4c-7000-8000-000000000001",
            "entity_type": "story",
            "entity_id": "018e6b3a-9f4c-7000-8000-000000000002",
            "source_path": "Uzume/story/abc/original.jpg",
            "mime_type": "image/jpeg",
            "size_bytes": 1048576
        }"#;
        let payload: MediaUploadedPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.entity_type, "story");
        assert_eq!(payload.mime_type, "image/jpeg");
    }

    // ── Pipeline validation ───────────────────────────────────────────────────

    #[test]
    fn pipeline_rejects_unknown_entity() {
        let pipeline = MediaPipeline::new(make_test_config());
        let result = pipeline.process_image(
            MediaJob {
                job_id: uuid::Uuid::now_v7(),
                entity_type: "nonexistent",
                entity_id: "id",
                mime_type: "image/jpeg",
            },
            &[],
        );
        assert!(matches!(result, Err(PipelineError::UnknownEntity(_))));
    }

    #[test]
    fn pipeline_rejects_unsupported_mime() {
        let pipeline = MediaPipeline::new(make_test_config());
        let result = pipeline.process_image(
            MediaJob {
                job_id: uuid::Uuid::now_v7(),
                entity_type: "test",
                entity_id: "id",
                mime_type: "image/gif",
            },
            &[],
        );
        assert!(matches!(result, Err(PipelineError::UnsupportedMimeType(_))));
    }

    #[test]
    fn pipeline_rejects_oversized_file() {
        let pipeline = MediaPipeline::new(make_test_config());
        let large = vec![0u8; 2 * 1024 * 1024];
        let result = pipeline.process_image(
            MediaJob {
                job_id: uuid::Uuid::now_v7(),
                entity_type: "test",
                entity_id: "id",
                mime_type: "image/jpeg",
            },
            &large,
        );
        assert!(matches!(result, Err(PipelineError::FileTooLarge { .. })));
    }

    #[test]
    fn test_pipeline_processes_image_job() {
        let pipeline = MediaPipeline::new(make_test_config());
        let data = test_jpeg_bytes();
        let result = pipeline.process_image(
            MediaJob {
                job_id: uuid::Uuid::now_v7(),
                entity_type: "test",
                entity_id: "entity-abc",
                mime_type: "image/jpeg",
            },
            &data,
        );
        assert!(result.is_ok(), "processing should succeed: {:?}", result);
        let img_result = result.unwrap();
        assert_eq!(img_result.entity_type, "test");
        assert_eq!(img_result.entity_id, "entity-abc");
        assert_eq!(img_result.variants.len(), 1);
    }

    // ── PipelineResult accessors ──────────────────────────────────────────────

    #[test]
    fn pipeline_result_image_accessors() {
        let id = uuid::Uuid::now_v7();
        let result = PipelineResult::Image(ImagePipelineResult {
            job_id: id,
            entity_type: "story".into(),
            entity_id: "xyz".into(),
            variants: vec![],
            processing_ms: 42,
        });
        assert_eq!(result.job_id(), id);
        assert_eq!(result.entity_type(), "story");
        assert_eq!(result.entity_id(), "xyz");
        assert_eq!(result.processing_ms(), 42);
    }

    #[test]
    fn processing_state_equality() {
        assert_eq!(ProcessingState::Accepted, ProcessingState::Accepted);
        assert_ne!(ProcessingState::Accepted, ProcessingState::Ready);
        assert_eq!(
            ProcessingState::Failed("err".into()),
            ProcessingState::Failed("err".into())
        );
        assert_ne!(
            ProcessingState::Failed("a".into()),
            ProcessingState::Failed("b".into())
        );
    }
}
