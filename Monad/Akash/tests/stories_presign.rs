use std::time::Duration;

use akash::{ObjectKey, StorageClient, UploadMetadata};
use nun::{config::StorageConfig, ErrorKind, Sensitive};

fn storage_config() -> StorageConfig {
    StorageConfig {
        endpoint: "http://localhost:9000".to_string(),
        region: "us-east-1".to_string(),
        bucket: "nyx".to_string(),
        access_key: Sensitive::new("minioadmin".to_string()),
        secret_key: Sensitive::new("minioadmin".to_string()),
    }
}

#[tokio::test]
async fn stories_upload_presign_accepts_allowed_media() {
    // #given
    let client = StorageClient::new(&storage_config()).expect("client should initialize");
    let key = ObjectKey::parse("Uzume/stories/0194e5a2-7b3c-7def-8a12-123456789abc/original.jpg")
        .expect("valid key");
    let metadata = UploadMetadata::new("image/jpeg", 1_024).expect("valid metadata");

    // #when
    let presigned = client
        .presign_put(&key, Duration::from_secs(60), metadata.clone())
        .await
        .expect("presign should succeed");

    // #then
    assert_eq!(presigned.method(), "PUT");
    assert!(presigned.url().contains("X-Amz-Algorithm"));
    assert_eq!(presigned.metadata(), &metadata);
    assert_eq!(
        presigned.headers().get("content-type"),
        Some(&"image/jpeg".to_string())
    );
    assert_eq!(
        presigned.headers().get("content-length"),
        Some(&"1024".to_string())
    );
}

#[test]
fn stories_upload_presign_rejects_disallowed_mime() {
    // #given #when
    let err = UploadMetadata::new("application/x-msdownload", 128).expect_err("must reject");

    // #then
    assert_eq!(err.kind(), ErrorKind::UnprocessableEntity);
    assert_eq!(err.code(), "validation_failed");
}

#[test]
fn stories_upload_presign_rejects_oversized_metadata() {
    // #given #when
    let err = UploadMetadata::new("video/mp4", UploadMetadata::max_size_bytes() + 1)
        .expect_err("must reject");

    // #then
    assert_eq!(err.kind(), ErrorKind::PayloadTooLarge);
    assert_eq!(err.code(), "upload_too_large");
}

#[test]
fn stories_upload_presign_rejects_unsafe_key_segment() {
    // #given #when
    let err = ObjectKey::parse("Uzume/stories/../original.jpg").expect_err("must reject");

    // #then
    assert_eq!(err.kind(), ErrorKind::UnprocessableEntity);
}

#[tokio::test]
async fn highlights_download_presign_generates_typed_response() {
    // #given
    let client = StorageClient::new(&storage_config()).expect("client should initialize");
    let key =
        ObjectKey::parse("Uzume/highlights/0194e5a2-7b3c-7def-8a12-123456789abc/original.mp4")
            .expect("valid key");

    // #when
    let presigned = client
        .presign_get(&key, Duration::from_secs(120))
        .await
        .expect("presign should succeed");

    // #then
    assert_eq!(presigned.method(), "GET");
    assert!(presigned.url().contains("X-Amz-Algorithm"));
    assert_eq!(
        presigned.headers().get("x-content-type-options"),
        Some(&"nosniff".to_string())
    );
}

#[test]
fn storage_client_rejects_empty_bucket_config() {
    // #given
    let mut config = storage_config();
    config.bucket = "".to_string();

    // #when
    let err = StorageClient::new(&config).expect_err("empty bucket should fail");

    // #then
    assert_eq!(err.kind(), ErrorKind::BadRequest);
    assert_eq!(err.code(), "storage_bucket_required");
}

#[test]
fn upload_metadata_normalizes_content_type() {
    // #given #when
    let metadata = UploadMetadata::new(" IMAGE/JPEG ", 512).expect("valid metadata");

    // #then
    assert_eq!(metadata.content_type(), "image/jpeg");
    assert_eq!(metadata.content_length(), 512);
}
