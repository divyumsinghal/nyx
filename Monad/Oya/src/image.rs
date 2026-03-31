use std::io::Cursor;
use std::num::NonZeroU32;
use std::path::Path;

use fast_image_resize::{FilterType, Image, PixelType, ResizeAlg, Resizer};
use image::DynamicImage;
use image::GenericImageView;
use thiserror::Error;

use crate::config::{ImageFormat, ImageVariant};

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("failed to decode image: {0}")]
    Decode(String),

    #[error("failed to encode image: {0}")]
    Encode(String),

    #[error("failed to resize image: {0}")]
    Resize(String),

    #[error("image too large: {width}x{height} exceeds {max_width}x{max_height}")]
    TooLarge {
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    },
}

#[derive(Debug, Clone)]
pub struct VariantResult {
    pub name: String,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
}

pub fn decode_image(data: &[u8]) -> Result<DynamicImage, ImageError> {
    image::load_from_memory(data).map_err(|e| ImageError::Decode(e.to_string()))
}

fn resize_image(
    img: &DynamicImage,
    max_width: u32,
    max_height: u32,
) -> Result<DynamicImage, ImageError> {
    let (src_w, src_h) = img.dimensions();

    if src_w <= max_width && src_h <= max_height {
        return Ok(img.clone());
    }

    let src_rgba = img.to_rgba8();
    let src_view = Image::from_vec_u8(
        NonZeroU32::new(src_w).unwrap(),
        NonZeroU32::new(src_h).unwrap(),
        src_rgba.as_raw().clone(),
        PixelType::U8x4,
    )
    .map_err(|e| ImageError::Resize(e.to_string()))?;

    let aspect = src_w as f64 / src_h as f64;
    let (dst_w, dst_h) = if src_w as f64 / max_width as f64 > src_h as f64 / max_height as f64 {
        (max_width, (max_width as f64 / aspect) as u32)
    } else {
        ((max_height as f64 * aspect) as u32, max_height)
    };

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
        .ok_or_else(|| ImageError::Resize("failed to create output image".into()))?;

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

pub fn process_variant(
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

pub fn process_all_variants(
    data: &[u8],
    variants: &[ImageVariant],
) -> Result<Vec<VariantResult>, ImageError> {
    let img = decode_image(data)?;
    variants.iter().map(|v| process_variant(&img, v)).collect()
}

pub fn save_variant(path: &Path, data: &[u8]) -> Result<(), ImageError> {
    std::fs::write(path, data).map_err(|e| ImageError::Encode(format!("write failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> Vec<u8> {
        let img = DynamicImage::new_rgb8(width, height);
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
            .unwrap();
        buf
    }

    #[test]
    fn decode_valid_image() {
        let data = create_test_image(100, 100);
        let img = decode_image(&data);
        assert!(img.is_ok());
    }

    #[test]
    fn decode_invalid_data_fails() {
        let result = decode_image(b"not an image");
        assert!(result.is_err());
    }

    #[test]
    fn resize_smaller_image_returns_original() {
        let data = create_test_image(100, 100);
        let img = decode_image(&data).unwrap();
        let variant = ImageVariant {
            name: "test".into(),
            max_width: 200,
            max_height: 200,
            format: ImageFormat::Jpeg(85),
        };
        let result = process_variant(&img, &variant);
        assert!(result.is_ok());
    }

    #[test]
    fn process_all_variants_returns_correct_count() {
        let data = create_test_image(1920, 1080);
        let variants = vec![
            ImageVariant {
                name: "640".into(),
                max_width: 640,
                max_height: 640,
                format: ImageFormat::Jpeg(80),
            },
            ImageVariant {
                name: "320".into(),
                max_width: 320,
                max_height: 320,
                format: ImageFormat::Jpeg(75),
            },
        ];
        let results = process_all_variants(&data, &variants);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 2);
    }

    #[test]
    fn variant_result_has_mime_type() {
        let data = create_test_image(100, 100);
        let img = decode_image(&data).unwrap();
        let variant = ImageVariant {
            name: "test".into(),
            max_width: 50,
            max_height: 50,
            format: ImageFormat::Jpeg(85),
        };
        let result = process_variant(&img, &variant).unwrap();
        assert_eq!(result.mime_type, "image/jpeg");
    }
}
