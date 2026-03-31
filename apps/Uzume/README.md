## Uzume

> When Amaterasu (the sun goddess) hid in a cave out of grief, the world fell into permanent darkness. All the gods tried everything to coax her out and failed. Then Ame-no-Uzume climbed on an overturned tub, performed a wild and comedic dance, and progressively shed her clothes, causing every god watching to burst into laughter and cheering. Amaterasu, confused by the impossible sound of joy in a dark world, opened the cave to look. The light returned. A dance went viral and saved the world. She is the Shinto patron deity of performing arts, joy, and entertainment.


**Uzume**: Five domain microservices — each a separate binary crate, a separate process, a separate container.

```
apps/Uzume/
├── Uzume-profiles/       # User profiles, follow graph, block/mute
├── Uzume-feed/           # Posts, likes, comments, saves, home timeline
├── Uzume-stories/        # Stories, highlights, 24h TTL, stickers
├── Uzume-reels/          # Short-form video, algorithmic feed, audio
└── Uzume-discover/       # Explore page, trending, search
```

All five share the same Cargo dependency pattern:

```toml
[dependencies]
Nun  = { path = "../../Monad/Nun" }
nyx-api     = { path = "../../Monad/nyx-api" }
Mnemosyne      = { path = "../../Monad/Mnemosyne" }
nyx-events  = { path = "../../Monad/nyx-events" }
Lethe   = { path = "../../Monad/Lethe" }
Akash = { path = "../../Monad/Akash" }
Brizo  = { path = "../../Monad/Brizo" }
```

All five share the same internal module structure:

```
Uzume-{service}/
├── Cargo.toml
├── src/
│   ├── main.rs            # NyxServer::builder() → serve
│   ├── config.rs          # Service-specific config
│   ├── routes/            # Axum Router: path + method → handler
│   ├── handlers/          # Extract → service call → response
│   ├── services/          # Business logic (pure, testable, no I/O)
│   ├── models/            # Domain types + sqlx FromRow structs
│   ├── queries/           # Raw SQL (sqlx::query_as! macros)
│   └── workers/           # Background tokio tasks (NATS subscribers)
└── tests/
    ├── api/               # Integration: real DB + HTTP (testcontainers)
    └── services/          # Unit: pure logic, mock data
```

### apps/Uzume/Uzume-profiles

**Port 3001.** Uzume user profiles, follow graph, block/mute.

**Data** (`Uzume` schema):
- `Uzume.profiles` — id, nyx_identity_id (FK, never exposed), alias (UNIQUE), display_name, bio, avatar_url, is_private, is_verified, follower_count, following_count, post_count, created_at
- `Uzume.follows` — follower_id, following_id, status (accepted|pending), created_at
- `Uzume.blocks` — blocker_id, blocked_id

**API** (prefix `/api/Uzume/profiles`):

```
GET    /{alias}              Public profile
GET    /me                   Own profile
PATCH  /me                   Update own profile
GET    /{alias}/followers    Follower list (cursor)
GET    /{alias}/following    Following list (cursor)
POST   /{alias}/follow       Follow (or request if private)
DELETE /{alias}/follow       Unfollow
POST   /{alias}/block        Block
DELETE /{alias}/block        Unblock
```

**Publishes**: `Uzume.user.followed`, `Uzume.user.blocked`, `Uzume.profile.updated`
**Consumes**: `nyx.user.created` → create profile stub with generated alias

**Called by**: Uzume-feed, Uzume-stories, Uzume-discover (HTTP, cached in DragonflyDB TTL 5min)

### apps/Uzume/Uzume-feed

**Port 3002.** Posts, likes, comments, saves, home timeline.

**Data** (`Uzume` schema):
- `Uzume.posts` — id, author_id, caption, hashtags[], location (JSONB), like_count, comment_count, type (photo|carousel|video), created_at
- `Uzume.post_media` — id, post_id, media_url, media_type, width, height, sort_order
- `Uzume.likes` — user_id, post_id, created_at (PK user_id+post_id)
- `Uzume.comments` — id, post_id, author_id, parent_id (nullable), text, like_count, created_at
- `Uzume.saves` — user_id, post_id (PK)
- `Uzume.user_timeline` — user_id, post_id, score, created_at (PK user_id+post_id)

**Feed strategy**: Hybrid push/pull (fanout-on-write for ≤10k followers, fanout-on-read for >10k). Exactly how other apps (like instagram, tiktok, etc) handle the celebrity problem.

**API** (prefix `/api/Uzume/feed`):

```
POST   /posts                 Create post (multipart: media[] + caption + tags)
GET    /posts/{id}            Get post
DELETE /posts/{id}            Delete own post
POST   /posts/{id}/like       Like
DELETE /posts/{id}/like       Unlike
GET    /posts/{id}/comments   Comments (cursor)
POST   /posts/{id}/comments   Create comment
DELETE /posts/{id}/comments/{cid}  Delete own comment
POST   /posts/{id}/save       Save
DELETE /posts/{id}/save       Unsave
GET    /feed                  Home timeline (cursor)
GET    /feed/hashtag/{tag}    Hashtag feed (cursor)
WS     /ws                    Real-time notifications
```

**Publishes**: `Uzume.post.created`, `Uzume.post.liked`, `Uzume.comment.created`
**Consumes**: `Uzume.user.followed` (rebuild timeline), `Uzume.media.processed` (update URLs)

**Workers**: feed_fanout (post → follower timelines), score_updater (engagement → score), search_sync (posts → Meilisearch)

**Calls**: Uzume-profiles (HTTP, author info, cached)

### apps/Uzume/Uzume-stories

**Port 3003.** Stories (24h TTL), highlights, interactive stickers.

**Data** (`Uzume` schema):
- `Uzume.stories` — id, author_id, media_url, media_type, sticker_data (JSONB), expires_at, created_at
- `Uzume.story_views` — story_id, viewer_id, viewed_at
- `Uzume.story_interactions` — id, story_id, user_id, type (poll_vote|question_reply|slider), data (JSONB)
- `Uzume.highlights` — id, owner_id, title, cover_url
- `Uzume.highlight_items` — highlight_id, story_id, sort_order

**API** (prefix `/api/Uzume/stories`):

```
POST   /stories               Create story (multipart)
GET    /stories/feed          Stories from followed users
GET    /stories/{id}          Single story (marks viewed)
GET    /stories/{id}/viewers  Viewer list (own stories)
POST   /stories/{id}/interact Submit poll/question/slider response
POST   /highlights            Create highlight
PATCH  /highlights/{id}       Update highlight
POST   /highlights/{id}/stories  Add stories
GET    /profiles/{alias}/highlights  User's highlights
```

**Publishes**: `Uzume.story.created`, `Uzume.story.viewed`
**Workers**: story_expiry (every 60s, expire past-TTL stories)

### apps/Uzume/Uzume-reels

**Port 3004.** Short-form video, algorithmic feed, audio reuse.

**Data** (`Uzume` schema):
- `Uzume.reels` — id, author_id, video_url, thumbnail_url, caption, audio_track_id, duration_seconds, like_count, comment_count, share_count, view_count, created_at
- `Uzume.reel_audio` — id, title, artist, audio_url, fingerprint, usage_count, original_reel_id

**Feed algorithm**: `score = (likes×1 + comments×2 + shares×3 + completions×5) / (age_hours^1.5)`. Personalization: category affinity vectors in DragonflyDB. Cold start: global trending.

**API** (prefix `/api/Uzume/reels`):

```
POST   /reels                 Create reel (multipart: video + caption)
GET    /reels/{id}            Single reel
GET    /reels/feed            Algorithmic feed (cursor)
POST   /reels/{id}/like       Like
DELETE /reels/{id}/like       Unlike
POST   /reels/{id}/view       Record view + watch duration
GET    /reels/audio/{id}      Audio track + reels using it
GET    /reels/audio/trending  Trending audio
```

**Publishes**: `Uzume.reel.created`, `Uzume.reel.viewed`

### apps/Uzume/Uzume-discover

**Port 3005.** Explore page, trending, search.

**Explore algorithm**: Two-stage. (1) Candidate generation: top-engagement posts from last 48h unseen by user, weighted by category. (2) Ranking: engagement rate, follow-overlap, category match, recency → weighted sum → top-K.

**API** (prefix `/api/Uzume/discover`):

```
GET    /discover                  Explore page (cursor, personalized)
GET    /discover/trending/hashtags  Trending hashtags
GET    /discover/trending/reels    Trending reels
GET    /search?q={q}&type={t}      Search (users|posts|hashtags|locations)
```

**Calls**: Uzume-feed (HTTP, engagement data), Uzume-profiles (HTTP, follow-overlap), Uzume-reels (HTTP, trending), Brizo (library, Meilisearch queries). All cached in DragonflyDB.

**Workers**: trending_updater (every 5min, recalculate trending), search_sync (NATS → Meilisearch)

---
