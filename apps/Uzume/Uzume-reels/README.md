# Uzume-reels — Short-form Video Service

> Port 3004. Handles short-form video, algorithmic feed, and audio reuse.

## Purpose

- Short-form video reels (up to 90s)
- Algorithmic feed based on engagement scoring
- Audio track reuse across reels
- View tracking and watch time analytics

## Key Dependencies

- Nun (core types)
- Akash (video storage)
- nyx-events (NATS)

## API Surface

See `apps/Uzume/README.md` for the full API documentation.

## Data Schema

Tables in `Uzume` schema: `reels`, `reel_audio`
