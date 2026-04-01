use std::collections::HashMap;

/// A single image variant definition.
#[derive(Debug, Clone)]
pub struct ImageVariant {
    pub name: String,
    pub max_width: u32,
    pub max_height: u32,
    pub format: ImageFormat,
}

/// Supported image output formats.
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Jpeg(u8),
    Png,
    Webp(u8),
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg(_) => "jpg",
            Self::Png => "png",
            Self::Webp(_) => "webp",
        }
    }
}

/// A single video variant definition.
#[derive(Debug, Clone)]
pub struct VideoVariant {
    pub name: String,
    pub resolution: (u32, u32),
    pub video_bitrate: String,
    pub audio_bitrate: String,
}

/// Processing configuration for a specific entity type.
#[derive(Debug, Clone)]
pub struct EntityConfig {
    pub entity_type: String,
    pub image_variants: Vec<ImageVariant>,
    pub video_variants: Vec<VideoVariant>,
    pub max_image_size_bytes: u64,
    pub max_video_size_bytes: u64,
    pub allowed_mime_types: Vec<String>,
}

/// Global processing configuration.
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub entities: HashMap<String, EntityConfig>,
    pub ffmpeg_path: String,
    pub temp_dir: String,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        let mut entities = HashMap::new();

        entities.insert(
            "story".to_string(),
            EntityConfig {
                entity_type: "story".to_string(),
                image_variants: vec![
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
                ],
                video_variants: vec![
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
                ],
                max_image_size_bytes: 10 * 1024 * 1024,
                max_video_size_bytes: 500 * 1024 * 1024,
                allowed_mime_types: vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                    "image/webp".to_string(),
                    "video/mp4".to_string(),
                    "video/quicktime".to_string(),
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
    pub fn get_entity(&self, entity_type: &str) -> Option<&EntityConfig> {
        self.entities.get(entity_type)
    }
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
}
