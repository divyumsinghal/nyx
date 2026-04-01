use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

use crate::config::VideoVariant;

#[derive(Debug, Error)]
pub enum VideoError {
    #[error("ffmpeg not found at {path}")]
    FfmpegNotFound { path: String },

    #[error("ffmpeg failed: {message}")]
    FfmpegFailed { message: String },

    #[error("failed to extract poster frame: {0}")]
    PosterExtraction(String),

    #[error("invalid video input: {0}")]
    InvalidInput(String),
}

/// Result of processing a single video variant.
#[derive(Debug, Clone)]
pub struct VideoVariantResult {
    pub name: String,
    pub output_path: PathBuf,
    pub playlist_path: Option<PathBuf>,
}

/// Result of full video processing.
#[derive(Debug, Clone)]
pub struct VideoProcessingResult {
    pub variants: Vec<VideoVariantResult>,
    pub poster_path: PathBuf,
    pub master_playlist: PathBuf,
}

/// Check if ffmpeg is available.
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

/// Extract a poster frame from a video at the given timestamp.
pub fn extract_poster(
    ffmpeg_path: &str,
    input_path: &Path,
    output_path: &Path,
    timestamp: &str,
) -> Result<(), VideoError> {
    let output = Command::new(ffmpeg_path)
        .args([
            "-i",
            input_path.to_str().unwrap_or(""),
            "-ss",
            timestamp,
            "-vframes",
            "1",
            "-q:v",
            "2",
            output_path.to_str().unwrap_or(""),
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

/// Transcode a video to a single HLS variant.
fn transcode_variant(
    ffmpeg_path: &str,
    input_path: &Path,
    variant: &VideoVariant,
    output_dir: &Path,
) -> Result<VideoVariantResult, VideoError> {
    let variant_dir = output_dir.join(&variant.name);
    std::fs::create_dir_all(&variant_dir).map_err(|e| VideoError::FfmpegFailed {
        message: format!("failed to create variant directory: {e}"),
    })?;

    let (width, height) = variant.resolution;
    let playlist_path = variant_dir.join("playlist.m3u8");

    let output = Command::new(ffmpeg_path)
        .args([
            "-i",
            input_path.to_str().unwrap_or(""),
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-b:v",
            &variant.video_bitrate,
            "-maxrate",
            &variant.video_bitrate,
            "-bufsize",
            &variant.video_bitrate,
            "-vf",
            &format!("scale={width}:{height}:force_original_aspect_ratio=decrease"),
            "-c:a",
            "aac",
            "-b:a",
            &variant.audio_bitrate,
            "-hls_time",
            "4",
            "-hls_playlist_type",
            "vod",
            "-hls_segment_filename",
            variant_dir.join("%03d.ts").to_str().unwrap_or(""),
            playlist_path.to_str().unwrap_or(""),
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
    })
}

/// Create a master HLS playlist referencing all variant playlists.
fn create_master_playlist(
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
            content.push_str(&format!(
                "#EXT-X-STREAM-INF:BANDWIDTH=1000000\n{relative}\n"
            ));
        }
    }

    std::fs::write(&master_path, content).map_err(|e| VideoError::FfmpegFailed {
        message: format!("failed to write master playlist: {e}"),
    })?;

    Ok(master_path)
}

/// Process a video file: transcode to all HLS variants and extract poster.
pub fn process_video(
    ffmpeg_path: &str,
    input_path: &Path,
    output_dir: &Path,
    variants: &[VideoVariant],
) -> Result<VideoProcessingResult, VideoError> {
    std::fs::create_dir_all(output_dir).map_err(|e| VideoError::FfmpegFailed {
        message: format!("failed to create output directory: {e}"),
    })?;

    let poster_path = output_dir.join("poster.jpg");
    extract_poster(ffmpeg_path, input_path, &poster_path, "00:00:01")?;

    let mut variant_results = Vec::new();
    for variant in variants {
        let result = transcode_variant(ffmpeg_path, input_path, variant, output_dir)?;
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
        let result = check_ffmpeg("/nonexistent/ffmpeg");
        assert!(result.is_err());
    }

    #[test]
    fn video_error_display() {
        let err = VideoError::FfmpegNotFound {
            path: "/usr/bin/ffmpeg".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("/usr/bin/ffmpeg"));
    }

    #[test]
    fn video_variant_result_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<VideoVariantResult>();
    }
}
