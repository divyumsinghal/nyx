use std::time::Instant;

use thiserror::Error;

use crate::config::{EntityConfig, ProcessingConfig};
use crate::image::{self, VariantResult as ImageVariantResult};
use crate::video::{self, VideoProcessingResult};

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("unknown entity type: {0}")]
    UnknownEntity(String),

    #[error("image processing failed: {0}")]
    Image(#[from] image::ImageError),

    #[error("video processing failed: {0}")]
    Video(#[from] video::VideoError),

    #[error("unsupported mime type: {0}")]
    UnsupportedMimeType(String),

    #[error("file too large: {size} bytes exceeds {max} bytes")]
    FileTooLarge { size: u64, max: u64 },
}

/// Processing state for tracking pipeline progress.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingState {
    Accepted,
    Processing,
    Ready,
    Failed(String),
}

/// Result of processing an image through the pipeline.
#[derive(Debug)]
pub struct ImagePipelineResult {
    pub job_id: uuid::Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub variants: Vec<ImageVariantResult>,
    pub processing_ms: u64,
}

/// Result of processing a video through the pipeline.
#[derive(Debug)]
pub struct VideoPipelineResult {
    pub job_id: uuid::Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub video_result: VideoProcessingResult,
    pub processing_ms: u64,
}

/// Result of any media processing through the pipeline.
#[derive(Debug)]
pub enum PipelineResult {
    Image(ImagePipelineResult),
    Video(VideoPipelineResult),
}

impl PipelineResult {
    pub fn job_id(&self) -> uuid::Uuid {
        match self {
            Self::Image(r) => r.job_id,
            Self::Video(r) => r.job_id,
        }
    }

    pub fn entity_type(&self) -> &str {
        match self {
            Self::Image(r) => &r.entity_type,
            Self::Video(r) => &r.entity_type,
        }
    }

    pub fn entity_id(&self) -> &str {
        match self {
            Self::Image(r) => &r.entity_id,
            Self::Video(r) => &r.entity_id,
        }
    }

    pub fn processing_ms(&self) -> u64 {
        match self {
            Self::Image(r) => r.processing_ms,
            Self::Video(r) => r.processing_ms,
        }
    }
}

/// Media processing pipeline orchestrator.
///
/// Takes raw media data, processes it according to the entity configuration,
/// and returns the processed variants.
pub struct MediaPipeline {
    config: ProcessingConfig,
}

impl MediaPipeline {
    pub fn new(config: ProcessingConfig) -> Self {
        Self { config }
    }

    /// Process an image through all configured variants.
    pub fn process_image(
        &self,
        job_id: uuid::Uuid,
        entity_type: &str,
        entity_id: &str,
        data: &[u8],
        mime_type: &str,
    ) -> Result<ImagePipelineResult, PipelineError> {
        let entity = self
            .config
            .get_entity(entity_type)
            .ok_or_else(|| PipelineError::UnknownEntity(entity_type.to_string()))?;

        validate_mime_type(mime_type, entity)?;
        validate_size(data.len() as u64, entity.max_image_size_bytes)?;

        let start = Instant::now();
        let variants = image::process_all_variants(data, &entity.image_variants)?;
        let processing_ms = start.elapsed().as_millis() as u64;

        Ok(ImagePipelineResult {
            job_id,
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            variants,
            processing_ms,
        })
    }

    /// Process a video through all configured HLS variants.
    pub fn process_video(
        &self,
        job_id: uuid::Uuid,
        entity_type: &str,
        entity_id: &str,
        input_path: &std::path::Path,
        output_dir: &std::path::Path,
        mime_type: &str,
    ) -> Result<VideoPipelineResult, PipelineError> {
        let entity = self
            .config
            .get_entity(entity_type)
            .ok_or_else(|| PipelineError::UnknownEntity(entity_type.to_string()))?;

        validate_mime_type(mime_type, entity)?;

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
            job_id,
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            video_result,
            processing_ms,
        })
    }
}

fn validate_mime_type(mime_type: &str, entity: &EntityConfig) -> Result<(), PipelineError> {
    let normalized = mime_type.to_lowercase().trim().to_string();
    if entity.allowed_mime_types.contains(&normalized) {
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
    use crate::config::{ImageFormat, ImageVariant, VideoVariant};

    fn test_config() -> ProcessingConfig {
        let mut config = ProcessingConfig::default();
        config.entities.insert(
            "test".to_string(),
            crate::config::EntityConfig {
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

    #[test]
    fn pipeline_rejects_unknown_entity() {
        let pipeline = MediaPipeline::new(test_config());
        let result = pipeline.process_image(
            uuid::Uuid::now_v7(),
            "unknown",
            "test-id",
            &[],
            "image/jpeg",
        );
        assert!(matches!(result, Err(PipelineError::UnknownEntity(_))));
    }

    #[test]
    fn pipeline_rejects_unsupported_mime() {
        let pipeline = MediaPipeline::new(test_config());
        let result =
            pipeline.process_image(uuid::Uuid::now_v7(), "test", "test-id", &[], "image/gif");
        assert!(matches!(result, Err(PipelineError::UnsupportedMimeType(_))));
    }

    #[test]
    fn pipeline_rejects_oversized_file() {
        let pipeline = MediaPipeline::new(test_config());
        let large_data = vec![0u8; 2 * 1024 * 1024];
        let result = pipeline.process_image(
            uuid::Uuid::now_v7(),
            "test",
            "test-id",
            &large_data,
            "image/jpeg",
        );
        assert!(matches!(result, Err(PipelineError::FileTooLarge { .. })));
    }

    #[test]
    fn pipeline_result_accessors() {
        let result = PipelineResult::Image(ImagePipelineResult {
            job_id: uuid::Uuid::now_v7(),
            entity_type: "story".into(),
            entity_id: "123".into(),
            variants: vec![],
            processing_ms: 100,
        });

        assert_eq!(result.entity_type(), "story");
        assert_eq!(result.entity_id(), "123");
        assert_eq!(result.processing_ms(), 100);
    }

    #[test]
    fn processing_state_equality() {
        assert_eq!(ProcessingState::Accepted, ProcessingState::Accepted);
        assert_ne!(ProcessingState::Accepted, ProcessingState::Processing);
        assert_ne!(
            ProcessingState::Failed("err".into()),
            ProcessingState::Ready
        );
    }
}
