use std::collections::BTreeMap;

use nun::{NyxError, Result, error::FieldError};

const MAX_UPLOAD_SIZE_BYTES: u64 = 100 * 1024 * 1024;
const ALLOWED_MIME_TYPES: [&str; 6] = [
    "image/jpeg",
    "image/png",
    "image/webp",
    "video/mp4",
    "video/webm",
    "image/gif",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadMetadata {
    content_type: String,
    content_length: u64,
}

impl UploadMetadata {
    pub fn new(content_type: impl Into<String>, content_length: u64) -> Result<Self> {
        let normalized_content_type = content_type.into().trim().to_ascii_lowercase();

        if normalized_content_type.is_empty() {
            return Err(NyxError::validation(vec![FieldError::new(
                "content_type",
                "required",
                "Content type is required",
            )]));
        }

        if !ALLOWED_MIME_TYPES.contains(&normalized_content_type.as_str()) {
            return Err(NyxError::validation(vec![FieldError::new(
                "content_type",
                "unsupported",
                "Content type is not allowed for story media",
            )]));
        }

        if content_length == 0 {
            return Err(NyxError::validation(vec![FieldError::new(
                "content_length",
                "required",
                "Content length must be greater than zero",
            )]));
        }

        if content_length > MAX_UPLOAD_SIZE_BYTES {
            return Err(NyxError::payload_too_large(
                "upload_too_large",
                format!(
                    "Upload payload exceeds {MAX_UPLOAD_SIZE_BYTES} bytes limit for story media"
                ),
            ));
        }

        Ok(Self {
            content_type: normalized_content_type,
            content_length,
        })
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn content_length(&self) -> u64 {
        self.content_length
    }

    pub const fn max_size_bytes() -> u64 {
        MAX_UPLOAD_SIZE_BYTES
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresignedUpload {
    url: String,
    method: String,
    headers: BTreeMap<String, String>,
    metadata: UploadMetadata,
}

impl PresignedUpload {
    pub fn new(
        url: String,
        method: impl Into<String>,
        headers: BTreeMap<String, String>,
        metadata: UploadMetadata,
    ) -> Self {
        Self {
            url,
            method: method.into(),
            headers,
            metadata,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }

    pub fn metadata(&self) -> &UploadMetadata {
        &self.metadata
    }
}
