# Uzume-stories — Stories Service

> Port 3003. Handles 24h ephemeral stories, highlights, and interactive stickers.

## Purpose

- Stories with 24h TTL
- Highlights (saved story collections)
- Interactive stickers (polls, questions, sliders)
- Story views and interactions

## Key Dependencies

- Nun, Mnemosyne (PostgreSQL)
- Akash (media storage)
- nyx-events (NATS)

## API Surface

See `apps/Uzume/README.md` for the full API documentation.

## Data Schema

Tables in `Uzume` schema: `stories`, `story_views`, `story_interactions`, `highlights`, `highlight_items`
