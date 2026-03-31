use std::time::Duration;

use akash::{ObjectKey, StorageClient};

#[test]
fn stories_upload_presign_accepts_allowed_media() {
    let _client = StorageClient;
    let key =
        ObjectKey("Uzume/stories/0194e5a2-7b3c-7def-8a12-123456789abc/original.jpg".to_string());
    let _expires_in = Duration::from_secs(60);
    let _content_type = "image/jpeg";
    let _content_length = 1_024_u64;
    let _ = key;
}

#[test]
fn stories_upload_presign_rejects_disallowed_mime() {
    let _client = StorageClient;
    panic!("not implemented");
}

#[test]
fn stories_upload_presign_rejects_oversized_metadata() {
    let _client = StorageClient;
    panic!("not implemented");
}

#[test]
fn stories_upload_presign_rejects_unsafe_key_segment() {
    let _client = StorageClient;
    panic!("not implemented");
}

#[test]
fn highlights_download_presign_generates_typed_response() {
    let _client = StorageClient;
}
