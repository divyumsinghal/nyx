# Uzume-feed — Posts & Timeline Service

> Port 3002. Handles posts, likes, comments, saves, and home timeline.

## Purpose

- Posts (photo, carousel, video) with media
- Likes, comments, saves
- Home timeline with hybrid push/pull fanout
- Real-time notifications via WebSocket

## Key Dependencies

- Nun, Mnemosyne (PostgreSQL)
- Heka (identity)
- Lethe (cache)
- Brizo (search)
- nyx-events (NATS)

## API Surface

See `apps/Uzume/README.md` for the full API documentation.

## Data Schema

Tables in `Uzume` schema: `posts`, `post_media`, `likes`, `comments`, `saves`, `user_timeline`
