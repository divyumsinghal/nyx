
### platform/nyx-storage

MinIO/S3 client wrapper. Depends on `Monad`.

MinIO/S3 client. Path convention: `{app}/{entity}/{id}/{variant}.{ext}`. Provides: `put_object`, `get_object`, `presigned_upload_url`, `presigned_download_url`.

```
nyx-storage/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs          # StorageClient (wraps rust-s3 Bucket), connection config
│   ├── upload.rs          # Upload helpers: put_object, presigned_upload_url
│   ├── download.rs        # Download helpers: get_object, presigned_download_url
│   └── paths.rs           # Path convention: {app}/{entity}/{id}/{variant}.{ext}
└── tests/
```

Storage path convention:

```
Uzume/posts/{post_id}/original.jpg
Uzume/posts/{post_id}/1080.jpg
Uzume/posts/{post_id}/640.jpg
Uzume/posts/{post_id}/320.jpg
Uzume/avatars/{user_id}/150.jpg
Uzume/stories/{story_id}/original.mp4
Uzume/reels/{reel_id}/hls/master.m3u8
Anteros/photos/{profile_id}/{n}.jpg
Themis/listings/{listing_id}/{n}.jpg
```

One MinIO bucket, app-level path prefixes. Clean, predictable, CDN-cacheable.
