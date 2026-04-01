# Uzume-profiles — User Profiles Service

> Port 3001. Handles user profiles, follow graph, and block/mute functionality.

## Purpose

- User profiles with app-scoped aliases
- Follow/unfollow with privacy-aware visibility
- Block/mute user relationships

## Key Dependencies

- Nun, Mnemosyne (PostgreSQL)
- Heka (identity/auth)
- Lethe (cache)
- Brizo (search)
- nyx-events (NATS pub/sub)

## API Surface

See `apps/Uzume/README.md` for the full API documentation.

## Data Schema

Tables in `Uzume` schema: `profiles`, `follows`, `blocks`
