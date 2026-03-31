# Oya

> Orisha of storms and transformation, raw input becomes structured output in the moment of the storm. She governs the marketplace where raw goods become valued product.

### Monad/Oya

Image: `fast_image_resize` + `image` crate (pure Rust). Generates variants: 1080, 640, 320, 150px. Strips EXIF.
Video: Shells out to FFmpeg. H.264 → HLS segments at 720p/480p/360p. Poster frame generation.
Worker: NATS subscriber on `*.media.uploaded` → process → store in MinIO → emit `*.media.processed`.

Media processing pipeline. This crate is both a library (processing functions) AND a binary (background worker process).

Depends on `Nun`, `Akash`, `nyx-events`, `Lethe`.

```
Oya/
├── Cargo.toml
├── src/
│   ├── lib.rs             # Library: re-exports processing functions
│   ├── bin/
│   │   └── worker.rs      # Binary: NATS subscriber, picks up media.uploaded events, processes
│   ├── image.rs           # Image processing: resize, strip EXIF, generate variants
│   ├── video.rs           # Video processing: FFmpeg transcode, HLS segmentation, thumbnails
│   ├── pipeline.rs        # Processing pipeline orchestrator: raw → variants → store → emit event
│   └── config.rs          # Variant definitions per entity type (post = 4 sizes, avatar = 1, etc.)
└── tests/
```

**Tool choice — Image processing:**
- **fast_image_resize + image crate (chosen)**: Pure Rust, no C dependencies for basic operations. `fast_image_resize` uses SIMD for 3-10x faster resizing than the `image` crate alone.
- Alternative — libvips (via FFI): Fastest image processing library in existence. But requires C library installation, complicates Docker builds, and FFI is unsafe.
- Alternative — thumbhash + blurhash: For generating placeholder hashes only. Not a replacement for full processing.

**Video processing** shells out to FFmpeg. FFmpeg is unbeatable and wrapping it in Rust FFI is not worth the complexity. The `std::process::Command` API is sufficient.