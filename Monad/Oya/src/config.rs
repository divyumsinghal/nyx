//! Processing configuration — variant definitions per entity type.
//!
//! [`ProcessingConfig`] holds a map of entity types (e.g., "story", "post",
//! "reel") to their [`EntityConfig`], which specifies the image and video
//! variants to generate.

use std::collections::HashMap;

/// A single image variant definition.
#[derive(Debug, Clone)]
pub struct ImageVariant {
    /// Short human-readable name, e.g. `"1080"`, `"640"`, `"thumb"`.
    pub name: String,
    /// Maximum pixel width for this variant.
    pub max_width: u32,
    /// Maximum pixel height for this variant.
    pub max_height: u32,
    /// Output image format and quality.
    pub format: ImageFormat,
}

/// Supported image output formats.
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    /// JPEG with quality 0–100.
    Jpeg(u8),
    /// Lossless PNG.
    Png,
    /// WebP with quality 0–100.
    Webp(u8),
}

impl ImageFormat {
    /// Returns the canonical file extension for this format.
    ///
    /// # Example
    ///
    /// ```
    /// use oya::config::ImageFormat;
    /// assert_eq!(ImageFormat::Jpeg(85).extension(), "jpg");
    /// assert_eq!(ImageFormat::Png.extension(), "png");
    /// assert_eq!(ImageFormat::Webp(80).extension(), "webp");
    /// ```
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg(_) => "jpg",
            Self::Png => "png",
            Self::Webp(_) => "webp",
        }
    }
}

/// A single HLS video variant definition.
#[derive(Debug, Clone)]
pub struct VideoVariant {
    /// Short human-readable name, e.g. `"720p"`, `"480p"`.
    pub name: String,
    /// Output resolution as `(width, height)`.
    pub resolution: (u32, u32),
    /// Target video bitrate, e.g. `"2500k"`.
    pub video_bitrate: String,
    /// Target audio bitrate, e.g. `"128k"`.
    pub audio_bitrate: String,
}

/// Processing configuration for a specific entity type.
#[derive(Debug, Clone)]
pub struct EntityConfig {
    /// Entity type identifier, e.g. `"story"`, `"post"`, `"reel"`.
    pub entity_type: String,
    /// Ordered list of image variants to produce.
    pub image_variants: Vec<ImageVariant>,
    /// Ordered list of HLS video variants to produce.
    pub video_variants: Vec<VideoVariant>,
    /// Maximum allowed image file size in bytes.
    pub max_image_size_bytes: u64,
    /// Maximum allowed video file size in bytes.
    pub max_video_size_bytes: u64,
    /// MIME types accepted for this entity (images and/or videos).
    pub allowed_mime_types: Vec<String>,
}

/// Global processing configuration.
///
/// Holds per-entity-type configurations and global settings like the FFmpeg
/// binary path and the temporary directory for intermediate files.
///
/// # Example
///
/// ```
/// use oya::config::ProcessingConfig;
/// let config = ProcessingConfig::default();
/// let story = config.get_entity("story").unwrap();
/// assert_eq!(story.image_variants.len(), 4);
/// ```
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    /// Per-entity-type configurations, keyed by entity type string.
    pub entities: HashMap<String, EntityConfig>,
    /// Path to the FFmpeg binary. Defaults to `"ffmpeg"` (assumes `$PATH`).
    pub ffmpeg_path: String,
    /// Temporary directory for intermediate files. Defaults to `"/tmp/oya"`.
    pub temp_dir: String,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        let mut entities = HashMap::new();

        entities.insert(
            "post".to_string(),
            EntityConfig {
                entity_type: "post".to_string(),
                image_variants: standard_post_image_variants(),
                video_variants: standard_video_variants(),
                max_image_size_bytes: 20 * 1024 * 1024,
                max_video_size_bytes: 500 * 1024 * 1024,
                allowed_mime_types: all_allowed_mime_types(),
            },
        );

        entities.insert(
            "story".to_string(),
            EntityConfig {
                entity_type: "story".to_string(),
                image_variants: story_image_variants(),
                video_variants: story_video_variants(),
                max_image_size_bytes: 10 * 1024 * 1024,
                max_video_size_bytes: 500 * 1024 * 1024,
                allowed_mime_types: all_allowed_mime_types(),
            },
        );

        entities.insert(
            "reel".to_string(),
            EntityConfig {
                entity_type: "reel".to_string(),
                image_variants: vec![
                    ImageVariant {
                        name: "thumb".to_string(),
                        max_width: 720,
                        max_height: 1280,
                        format: ImageFormat::Jpeg(85),
                    },
                ],
                video_variants: standard_video_variants(),
                max_image_size_bytes: 5 * 1024 * 1024,
                max_video_size_bytes: 1024 * 1024 * 1024, // 1 GB
                allowed_mime_types: vec![
                    "video/mp4".to_string(),
                    "video/quicktime".to_string(),
                    "image/jpeg".to_string(),
                ],
            },
        );

        entities.insert(
            "avatar".to_string(),
            EntityConfig {
                entity_type: "avatar".to_string(),
                image_variants: vec![
                    ImageVariant {
                        name: "320".to_string(),
                        max_width: 320,
                        max_height: 320,
                        format: ImageFormat::Jpeg(85),
                    },
                    ImageVariant {
                        name: "150".to_string(),
                        max_width: 150,
                        max_height: 150,
                        format: ImageFormat::Jpeg(80),
                    },
                ],
                video_variants: vec![],
                max_image_size_bytes: 5 * 1024 * 1024,
                max_video_size_bytes: 0,
                allowed_mime_types: vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                    "image/webp".to_string(),
                ],
            },
        );

        Self {
            entities,
            ffmpeg_path: "ffmpeg".to_string(),
            temp_dir: "/tmp/oya".to_string(),
        }
    }
}

impl ProcessingConfig {
    /// Look up entity configuration by type string.
    ///
    /// Returns `None` if the entity type has no configuration.
    pub fn get_entity(&self, entity_type: &str) -> Option<&EntityConfig> {
        self.entities.get(entity_type)
    }
}

// ── Shared variant presets ───────────────────────────────────────────────────

fn standard_post_image_variants() -> Vec<ImageVariant> {
    vec![
        ImageVariant {
            name: "1080".to_string(),
            max_width: 1080,
            max_height: 1350,
            format: ImageFormat::Jpeg(85),
        },
        ImageVariant {
            name: "640".to_string(),
            max_width: 640,
            max_height: 800,
            format: ImageFormat::Jpeg(80),
        },
        ImageVariant {
            name: "320".to_string(),
            max_width: 320,
            max_height: 400,
            format: ImageFormat::Jpeg(75),
        },
        ImageVariant {
            name: "150".to_string(),
            max_width: 150,
            max_height: 188,
            format: ImageFormat::Jpeg(70),
        },
    ]
}

fn story_image_variants() -> Vec<ImageVariant> {
    vec![
        ImageVariant {
            name: "1080".to_string(),
            max_width: 1080,
            max_height: 1920,
            format: ImageFormat::Jpeg(85),
        },
        ImageVariant {
            name: "640".to_string(),
            max_width: 640,
            max_height: 1138,
            format: ImageFormat::Jpeg(80),
        },
        ImageVariant {
            name: "320".to_string(),
            max_width: 320,
            max_height: 569,
            format: ImageFormat::Jpeg(75),
        },
        ImageVariant {
            name: "150".to_string(),
            max_width: 150,
            max_height: 267,
            format: ImageFormat::Jpeg(70),
        },
    ]
}

fn story_video_variants() -> Vec<VideoVariant> {
    vec![
        VideoVariant {
            name: "720p".to_string(),
            resolution: (720, 1280),
            video_bitrate: "2500k".to_string(),
            audio_bitrate: "128k".to_string(),
        },
        VideoVariant {
            name: "480p".to_string(),
            resolution: (480, 854),
            video_bitrate: "1200k".to_string(),
            audio_bitrate: "96k".to_string(),
        },
        VideoVariant {
            name: "360p".to_string(),
            resolution: (360, 640),
            video_bitrate: "800k".to_string(),
            audio_bitrate: "64k".to_string(),
        },
    ]
}

fn standard_video_variants() -> Vec<VideoVariant> {
    vec![
        VideoVariant {
            name: "1080p".to_string(),
            resolution: (1080, 1920),
            video_bitrate: "4000k".to_string(),
            audio_bitrate: "192k".to_string(),
        },
        VideoVariant {
            name: "720p".to_string(),
            resolution: (720, 1280),
            video_bitrate: "2500k".to_string(),
            audio_bitrate: "128k".to_string(),
        },
        VideoVariant {
            name: "480p".to_string(),
            resolution: (480, 854),
            video_bitrate: "1200k".to_string(),
            audio_bitrate: "96k".to_string(),
        },
        VideoVariant {
            name: "360p".to_string(),
            resolution: (360, 640),
            video_bitrate: "800k".to_string(),
            audio_bitrate: "64k".to_string(),
        },
    ]
}

fn all_allowed_mime_types() -> Vec<String> {
    vec![
        "image/jpeg".to_string(),
        "image/png".to_string(),
        "image/webp".to_string(),
        "video/mp4".to_string(),
        "video/quicktime".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_story_entity() {
        let config = ProcessingConfig::default();
        let story = config.get_entity("story").unwrap();
        assert_eq!(story.entity_type, "story");
    }

    #[test]
    fn story_config_has_four_image_variants() {
        let config = ProcessingConfig::default();
        let story = config.get_entity("story").unwrap();
        assert_eq!(story.image_variants.len(), 4);

        let names: Vec<&str> = story
            .image_variants
            .iter()
            .map(|v| v.name.as_str())
            .collect();
        assert!(names.contains(&"1080"));
        assert!(names.contains(&"640"));
        assert!(names.contains(&"320"));
        assert!(names.contains(&"150"));
    }

    #[test]
    fn story_config_has_three_video_variants() {
        let config = ProcessingConfig::default();
        let story = config.get_entity("story").unwrap();
        assert_eq!(story.video_variants.len(), 3);

        let names: Vec<&str> = story
            .video_variants
            .iter()
            .map(|v| v.name.as_str())
            .collect();
        assert!(names.contains(&"720p"));
        assert!(names.contains(&"480p"));
        assert!(names.contains(&"360p"));
    }

    #[test]
    fn image_format_extension() {
        assert_eq!(ImageFormat::Jpeg(85).extension(), "jpg");
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Webp(80).extension(), "webp");
    }

    #[test]
    fn unknown_entity_returns_none() {
        let config = ProcessingConfig::default();
        assert!(config.get_entity("unknown").is_none());
    }

    #[test]
    fn post_config_has_four_image_variants() {
        let config = ProcessingConfig::default();
        let post = config.get_entity("post").unwrap();
        assert_eq!(post.image_variants.len(), 4);
    }

    #[test]
    fn reel_config_exists() {
        let config = ProcessingConfig::default();
        assert!(config.get_entity("reel").is_some());
    }

    #[test]
    fn avatar_config_has_two_image_variants() {
        let config = ProcessingConfig::default();
        let avatar = config.get_entity("avatar").unwrap();
        assert_eq!(avatar.image_variants.len(), 2);
    }
}
