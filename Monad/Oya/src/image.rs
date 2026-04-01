//! Image processing — decode, resize, re-encode.
//!
//! Uses the `image` crate for decode/encode and `fast_image_resize` for
//! high-quality SIMD-accelerated downscaling.
//!
//! EXIF metadata is stripped automatically because the `image` crate re-encodes
//! the raw pixel data; no EXIF-specific pass is needed.

use std::io::Cursor;
use std::num::NonZeroU32;

use fast_image_resize::{FilterType, Image, PixelType, ResizeAlg, Resizer};
use image::DynamicImage;
use image::GenericImageView;
use thiserror::Error;

use crate::config::{ImageFormat, ImageVariant};

/// Errors that can occur during image processing.
#[derive(Debug, Error)]
pub enum ImageError {
    /// The input bytes could not be decoded as a known image format.
    #[error("failed to decode image: {0}")]
    Decode(String),

    /// The output image could not be encoded or written.
    #[error("failed to encode image: {0}")]
    Encode(String),

    /// The resize operation failed.
    #[error("failed to resize image: {0}")]
    Resize(String),

    /// The image exceeds the configured maximum dimensions.
    #[error("image too large: {width}x{height} exceeds {max_width}x{max_height}")]
    TooLarge {
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    },
}

/// Processed result for a single image variant.
#[derive(Debug, Clone)]
pub struct VariantResult {
    /// Variant name, e.g. `"1080"`, `"640"`.
    pub name: String,
    /// Encoded image bytes.
    pub data: Vec<u8>,
    /// Actual output width in pixels.
    pub width: u32,
    /// Actual output height in pixels.
    pub height: u32,
    /// MIME type of the encoded output.
    pub mime_type: String,
}

/// Decode raw bytes into a [`DynamicImage`].
///
/// EXIF data is stripped implicitly — the `image` crate decodes into raw pixels
/// and any re-encoding step below does not include EXIF.
///
/// # Errors
///
/// Returns [`ImageError::Decode`] if the bytes are not a recognised image format.
pub fn decode_image(data: &[u8]) -> Result<DynamicImage, ImageError> {
    image::load_from_memory(data).map_err(|e| ImageError::Decode(e.to_string()))
}

/// Resize and re-encode a single [`DynamicImage`] according to a variant spec.
///
/// If the image is already smaller than the variant's max dimensions, it is
/// re-encoded without resizing (no upscaling).
///
/// # Errors
///
/// Returns an [`ImageError`] if the resize or encode step fails.
pub fn resize_and_encode(
    img: &DynamicImage,
    variant: &ImageVariant,
) -> Result<VariantResult, ImageError> {
    let resized = resize_image(img, variant.max_width, variant.max_height)?;
    let (width, height) = resized.dimensions();
    let data = encode_image(&resized, &variant.format)?;

    let mime_type = match &variant.format {
        ImageFormat::Jpeg(_) => "image/jpeg".to_string(),
        ImageFormat::Png => "image/png".to_string(),
        ImageFormat::Webp(_) => "image/webp".to_string(),
    };

    Ok(VariantResult {
        name: variant.name.clone(),
        data,
        width,
        height,
        mime_type,
    })
}

/// Decode raw bytes and produce all requested variants in one pass.
///
/// The image is decoded once; each variant is produced by a separate resize +
/// encode step.
///
/// # Errors
///
/// Returns the first [`ImageError`] encountered during decode or any variant
/// processing.
pub fn process_image_to_variants(
    data: &[u8],
    variants: &[ImageVariant],
) -> Result<Vec<VariantResult>, ImageError> {
    let img = decode_image(data)?;
    variants.iter().map(|v| resize_and_encode(&img, v)).collect()
}

// ── Internal helpers ─────────────────────────────────────────────────────────

fn resize_image(
    img: &DynamicImage,
    max_width: u32,
    max_height: u32,
) -> Result<DynamicImage, ImageError> {
    let (src_w, src_h) = img.dimensions();

    // No upscaling — return a clone if already within bounds.
    if src_w <= max_width && src_h <= max_height {
        return Ok(img.clone());
    }

    // Convert to RGBA8 for fast_image_resize.
    let src_rgba = img.to_rgba8();
    let src_view = Image::from_vec_u8(
        NonZeroU32::new(src_w).unwrap(),
        NonZeroU32::new(src_h).unwrap(),
        src_rgba.as_raw().clone(),
        PixelType::U8x4,
    )
    .map_err(|e| ImageError::Resize(e.to_string()))?;

    // Compute output dimensions preserving aspect ratio.
    let aspect = src_w as f64 / src_h as f64;
    let (dst_w, dst_h) =
        if src_w as f64 / max_width as f64 > src_h as f64 / max_height as f64 {
            (max_width, (max_width as f64 / aspect) as u32)
        } else {
            ((max_height as f64 * aspect) as u32, max_height)
        };

    let dst_w = dst_w.max(1);
    let dst_h = dst_h.max(1);

    let mut dst_buffer = vec![0u8; (dst_w * dst_h * 4) as usize];
    let mut dst_view = Image::from_slice_u8(
        NonZeroU32::new(dst_w).unwrap(),
        NonZeroU32::new(dst_h).unwrap(),
        &mut dst_buffer,
        PixelType::U8x4,
    )
    .map_err(|e| ImageError::Resize(e.to_string()))?;

    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    resizer
        .resize(&src_view.view(), &mut dst_view.view_mut())
        .map_err(|e| ImageError::Resize(e.to_string()))?;

    let rgba_img = image::RgbaImage::from_raw(dst_w, dst_h, dst_buffer)
        .ok_or_else(|| ImageError::Resize("failed to construct output image buffer".into()))?;

    Ok(DynamicImage::ImageRgba8(rgba_img))
}

fn encode_image(img: &DynamicImage, format: &ImageFormat) -> Result<Vec<u8>, ImageError> {
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);

    match format {
        ImageFormat::Jpeg(_) => {
            img.write_to(&mut cursor, image::ImageFormat::Jpeg)
                .map_err(|e| ImageError::Encode(e.to_string()))?;
        }
        ImageFormat::Png => {
            img.write_to(&mut cursor, image::ImageFormat::Png)
                .map_err(|e| ImageError::Encode(e.to_string()))?;
        }
        ImageFormat::Webp(_) => {
            img.write_to(&mut cursor, image::ImageFormat::WebP)
                .map_err(|e| ImageError::Encode(e.to_string()))?;
        }
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid JPEG from a synthetic image.
    fn create_test_jpeg(width: u32, height: u32) -> Vec<u8> {
        let img = DynamicImage::new_rgb8(width, height);
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
            .unwrap();
        buf
    }

    // ── decode_image ─────────────────────────────────────────────────────────

    #[test]
    fn test_decode_valid_jpeg() {
        let data = create_test_jpeg(100, 100);
        let result = decode_image(&data);
        assert!(result.is_ok(), "valid JPEG should decode without error");
        let img = result.unwrap();
        assert_eq!(img.dimensions(), (100, 100));
    }

    #[test]
    fn test_invalid_image_returns_error() {
        let result = decode_image(b"definitely not an image");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("failed to decode"));
    }

    #[test]
    fn test_empty_bytes_returns_error() {
        let result = decode_image(&[]);
        assert!(result.is_err());
    }

    // ── resize_and_encode ────────────────────────────────────────────────────

    #[test]
    fn test_resize_image_to_variants() {
        let data = create_test_jpeg(1920, 1080);
        let img = decode_image(&data).unwrap();
        let variant = ImageVariant {
            name: "640".to_string(),
            max_width: 640,
            max_height: 640,
            format: ImageFormat::Jpeg(80),
        };
        let result = resize_and_encode(&img, &variant).unwrap();
        // Width should be ≤ 640 and height ≤ 640 after proportional resize.
        assert!(result.width <= 640);
        assert!(result.height <= 640);
        assert_eq!(result.name, "640");
        assert_eq!(result.mime_type, "image/jpeg");
    }

    #[test]
    fn test_smaller_image_not_upscaled() {
        let data = create_test_jpeg(100, 100);
        let img = decode_image(&data).unwrap();
        let variant = ImageVariant {
            name: "large".to_string(),
            max_width: 500,
            max_height: 500,
            format: ImageFormat::Jpeg(85),
        };
        let result = resize_and_encode(&img, &variant).unwrap();
        // Must not upscale — output should equal original dimensions.
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 100);
    }

    #[test]
    fn test_variant_result_has_non_empty_data() {
        let data = create_test_jpeg(200, 200);
        let img = decode_image(&data).unwrap();
        let variant = ImageVariant {
            name: "100".to_string(),
            max_width: 100,
            max_height: 100,
            format: ImageFormat::Jpeg(80),
        };
        let result = resize_and_encode(&img, &variant).unwrap();
        assert!(!result.data.is_empty());
    }

    #[test]
    fn test_png_variant_mime_type() {
        let data = create_test_jpeg(100, 100);
        let img = decode_image(&data).unwrap();
        let variant = ImageVariant {
            name: "png".to_string(),
            max_width: 100,
            max_height: 100,
            format: ImageFormat::Png,
        };
        let result = resize_and_encode(&img, &variant).unwrap();
        assert_eq!(result.mime_type, "image/png");
    }

    // ── process_image_to_variants ─────────────────────────────────────────────

    #[test]
    fn test_process_image_generates_four_variants() {
        let data = create_test_jpeg(1920, 1080);
        let variants = vec![
            ImageVariant {
                name: "1080".to_string(),
                max_width: 1080,
                max_height: 1080,
                format: ImageFormat::Jpeg(85),
            },
            ImageVariant {
                name: "640".to_string(),
                max_width: 640,
                max_height: 640,
                format: ImageFormat::Jpeg(80),
            },
            ImageVariant {
                name: "320".to_string(),
                max_width: 320,
                max_height: 320,
                format: ImageFormat::Jpeg(75),
            },
            ImageVariant {
                name: "150".to_string(),
                max_width: 150,
                max_height: 150,
                format: ImageFormat::Jpeg(70),
            },
        ];
        let results = process_image_to_variants(&data, &variants).unwrap();
        assert_eq!(results.len(), 4);
        let names: Vec<&str> = results.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"1080"));
        assert!(names.contains(&"640"));
        assert!(names.contains(&"320"));
        assert!(names.contains(&"150"));
    }

    #[test]
    fn test_process_image_invalid_input_fails() {
        let variants = vec![ImageVariant {
            name: "test".to_string(),
            max_width: 100,
            max_height: 100,
            format: ImageFormat::Jpeg(80),
        }];
        let result = process_image_to_variants(b"not an image", &variants);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_image_empty_variants_returns_empty_vec() {
        let data = create_test_jpeg(100, 100);
        let results = process_image_to_variants(&data, &[]).unwrap();
        assert!(results.is_empty());
    }
}
