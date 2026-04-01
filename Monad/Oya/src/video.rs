//! Video processing — HLS transcoding and thumbnail extraction via FFmpeg.
//!
//! All video operations shell out to `ffmpeg`. The binary path is configurable
//! in [`ProcessingConfig`](crate::config::ProcessingConfig).
//!
//! ## HLS output layout
//!
//! ```text
//! output_dir/
//!   {variant.name}/
//!     playlist.m3u8       # per-variant HLS manifest
//!     000.ts, 001.ts, … # 4-second segments
//!   master.m3u8           # master HLS playlist referencing all variants
//!   poster.jpg            # thumbnail at 00:00:01
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

use crate::config::VideoVariant;

/// Errors that can occur during video processing.
#[derive(Debug, Error)]
pub enum VideoError {
    /// FFmpeg binary was not found or failed to start.
    #[error("ffmpeg not found at {path}")]
    FfmpegNotFound {
        /// The path that was tried.
        path: String,
    },

    /// FFmpeg ran but exited with a non-zero code.
    #[error("ffmpeg failed: {message}")]
    FfmpegFailed {
        /// Captured stderr from FFmpeg.
        message: String,
    },

    /// Poster/thumbnail extraction failed.
    #[error("failed to extract poster frame: {0}")]
    PosterExtraction(String),

    /// Input path is invalid or the file cannot be accessed.
    #[error("invalid video input: {0}")]
    InvalidInput(String),

    /// Filesystem operation (create dir, write file) failed.
    #[error("I/O error: {0}")]
    Io(String),
}

/// Result of processing a single HLS video variant.
#[derive(Debug, Clone)]
pub struct VideoVariantResult {
    /// Variant name, e.g. `"720p"`.
    pub name: String,
    /// Directory containing the variant's segments and playlist.
    pub output_path: PathBuf,
    /// Path to the per-variant HLS playlist (`.m3u8`).
    pub playlist_path: Option<PathBuf>,
    /// Actual output resolution `(width, height)`.
    pub resolution: (u32, u32),
}

/// Result of full video processing (all variants + poster + master playlist).
#[derive(Debug, Clone)]
pub struct VideoProcessingResult {
    /// Results for each configured variant, in order.
    pub variants: Vec<VideoVariantResult>,
    /// Path to the extracted poster frame (JPEG).
    pub poster_path: PathBuf,
    /// Path to the master HLS playlist referencing all variants.
    pub master_playlist: PathBuf,
}

/// Verify that the FFmpeg binary is reachable.
///
/// # Errors
///
/// Returns [`VideoError::FfmpegNotFound`] if `ffmpeg -version` fails.
pub fn check_ffmpeg(ffmpeg_path: &str) -> Result<(), VideoError> {
    let output = Command::new(ffmpeg_path)
        .arg("-version")
        .output()
        .map_err(|_| VideoError::FfmpegNotFound {
            path: ffmpeg_path.to_string(),
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(VideoError::FfmpegNotFound {
            path: ffmpeg_path.to_string(),
        })
    }
}

/// Extract a single video frame as a JPEG thumbnail.
///
/// Uses `ffmpeg -ss {timestamp} -vframes 1 -q:v 2`.
///
/// # Errors
///
/// Returns [`VideoError::PosterExtraction`] if FFmpeg fails.
pub fn extract_thumbnail(
    ffmpeg_path: &str,
    input_path: &Path,
    output_path: &Path,
    timestamp: &str,
) -> Result<(), VideoError> {
    let input_str = input_path.to_str().ok_or_else(|| {
        VideoError::InvalidInput("non-UTF-8 input path".into())
    })?;
    let output_str = output_path.to_str().ok_or_else(|| {
        VideoError::PosterExtraction("non-UTF-8 output path".into())
    })?;

    let output = Command::new(ffmpeg_path)
        .args([
            "-i", input_str,
            "-ss", timestamp,
            "-vframes", "1",
            "-q:v", "2",
            output_str,
            "-y",
        ])
        .output()
        .map_err(|e| VideoError::PosterExtraction(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(VideoError::PosterExtraction(stderr.to_string()));
    }

    Ok(())
}

/// Transcode a video to HLS for a single quality variant.
///
/// Output layout:
/// ```text
/// output_dir/{variant.name}/
///   playlist.m3u8
///   000.ts, 001.ts, …
/// ```
///
/// # Errors
///
/// Returns a [`VideoError`] if directory creation or FFmpeg fails.
pub fn transcode_to_hls(
    ffmpeg_path: &str,
    input_path: &Path,
    output_dir: &Path,
    variant: &VideoVariant,
) -> Result<VideoVariantResult, VideoError> {
    let variant_dir = output_dir.join(&variant.name);
    std::fs::create_dir_all(&variant_dir).map_err(|e| VideoError::Io(e.to_string()))?;

    let (width, height) = variant.resolution;
    let playlist_path = variant_dir.join("playlist.m3u8");
    let segment_pattern = variant_dir.join("%03d.ts");

    let input_str = input_path
        .to_str()
        .ok_or_else(|| VideoError::InvalidInput("non-UTF-8 input path".into()))?;
    let playlist_str = playlist_path
        .to_str()
        .ok_or_else(|| VideoError::Io("non-UTF-8 playlist path".into()))?;
    let segment_str = segment_pattern
        .to_str()
        .ok_or_else(|| VideoError::Io("non-UTF-8 segment path".into()))?;

    let scale_filter = format!(
        "scale={width}:{height}:force_original_aspect_ratio=decrease"
    );

    let output = Command::new(ffmpeg_path)
        .args([
            "-i", input_str,
            "-c:v", "libx264",
            "-preset", "fast",
            "-b:v", &variant.video_bitrate,
            "-maxrate", &variant.video_bitrate,
            "-bufsize", &variant.video_bitrate,
            "-vf", &scale_filter,
            "-c:a", "aac",
            "-b:a", &variant.audio_bitrate,
            "-hls_time", "4",
            "-hls_playlist_type", "vod",
            "-hls_segment_filename", segment_str,
            playlist_str,
            "-y",
        ])
        .output()
        .map_err(|e| VideoError::FfmpegFailed {
            message: format!("ffmpeg execution failed: {e}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(VideoError::FfmpegFailed {
            message: stderr.to_string(),
        });
    }

    Ok(VideoVariantResult {
        name: variant.name.clone(),
        output_path: variant_dir,
        playlist_path: Some(playlist_path),
        resolution: variant.resolution,
    })
}

/// Create a master HLS playlist referencing all per-variant playlists.
///
/// # Errors
///
/// Returns [`VideoError::Io`] if writing the file fails.
pub fn create_master_playlist(
    output_dir: &Path,
    variants: &[VideoVariantResult],
) -> Result<PathBuf, VideoError> {
    let master_path = output_dir.join("master.m3u8");
    let mut content = String::from("#EXTM3U\n#EXT-X-VERSION:3\n");

    for variant in variants {
        if let Some(ref playlist) = variant.playlist_path {
            let relative = playlist
                .strip_prefix(output_dir)
                .unwrap_or(playlist)
                .to_str()
                .unwrap_or("");
            let (width, height) = variant.resolution;
            // Bandwidth is approximate; a real implementation would probe the file.
            content.push_str(&format!(
                "#EXT-X-STREAM-INF:BANDWIDTH=1000000,RESOLUTION={width}x{height}\n{relative}\n"
            ));
        }
    }

    std::fs::write(&master_path, content).map_err(|e| VideoError::Io(e.to_string()))?;

    Ok(master_path)
}

/// Process a video file: extract poster frame, transcode to all HLS variants,
/// and write the master playlist.
///
/// # Errors
///
/// Returns the first [`VideoError`] encountered.
pub fn process_video(
    ffmpeg_path: &str,
    input_path: &Path,
    output_dir: &Path,
    variants: &[VideoVariant],
) -> Result<VideoProcessingResult, VideoError> {
    std::fs::create_dir_all(output_dir).map_err(|e| VideoError::Io(e.to_string()))?;

    let poster_path = output_dir.join("poster.jpg");
    extract_thumbnail(ffmpeg_path, input_path, &poster_path, "00:00:01")?;

    let mut variant_results = Vec::with_capacity(variants.len());
    for variant in variants {
        let result = transcode_to_hls(ffmpeg_path, input_path, output_dir, variant)?;
        variant_results.push(result);
    }

    let master_playlist = create_master_playlist(output_dir, &variant_results)?;

    Ok(VideoProcessingResult {
        variants: variant_results,
        poster_path,
        master_playlist,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_ffmpeg_not_found_returns_error() {
        let result = check_ffmpeg("/nonexistent/path/to/ffmpeg");
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), VideoError::FfmpegNotFound { .. })
        );
    }

    #[test]
    fn video_error_display_ffmpeg_not_found() {
        let err = VideoError::FfmpegNotFound {
            path: "/usr/bin/ffmpeg".into(),
        };
        assert!(err.to_string().contains("/usr/bin/ffmpeg"));
    }

    #[test]
    fn video_error_display_ffmpeg_failed() {
        let err = VideoError::FfmpegFailed {
            message: "codec not found".into(),
        };
        assert!(err.to_string().contains("codec not found"));
    }

    #[test]
    fn video_variant_result_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<VideoVariantResult>();
    }

    #[test]
    fn video_processing_result_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<VideoProcessingResult>();
    }

    #[test]
    #[ignore = "requires ffmpeg binary in PATH"]
    fn test_hls_transcode() {
        // This test requires a real video file and ffmpeg.
        // Run with: cargo test -- --ignored test_hls_transcode
        use std::path::PathBuf;
        let tmp = tempfile::tempdir().unwrap();
        let input = PathBuf::from("tests/fixtures/sample.mp4");
        let output = tmp.path().join("hls");
        let variant = VideoVariant {
            name: "360p".to_string(),
            resolution: (360, 640),
            video_bitrate: "800k".to_string(),
            audio_bitrate: "64k".to_string(),
        };
        let result = transcode_to_hls("ffmpeg", &input, &output, &variant);
        assert!(result.is_ok(), "HLS transcode failed: {:?}", result);
    }

    #[test]
    #[ignore = "requires ffmpeg binary in PATH and a test video"]
    fn test_thumbnail_extraction() {
        use std::path::PathBuf;
        let tmp = tempfile::tempdir().unwrap();
        let input = PathBuf::from("tests/fixtures/sample.mp4");
        let output = tmp.path().join("thumb.jpg");
        let result = extract_thumbnail("ffmpeg", &input, &output, "00:00:01");
        assert!(result.is_ok());
        assert!(output.exists());
    }
}
