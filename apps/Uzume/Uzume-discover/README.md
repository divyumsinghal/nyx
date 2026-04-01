# Uzume-discover — Explore & Search Service

> Port 3005. Handles the explore/discover page, trending, and full-text search.

## Purpose

- Personalized explore feed
- Trending hashtags and reels
- Full-text search across users, posts, hashtags, locations
- Two-stage recommendation algorithm

## Key Dependencies

- Nun (core types)
- Brizo (Meilisearch queries)
- Lethe (cache)

## API Surface

See `apps/Uzume/README.md` for the full API documentation.

## Notes

This service aggregates data from other Uzume services (feed, profiles, reels) and provides search functionality via Meilisearch.
