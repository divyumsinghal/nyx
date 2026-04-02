# Uzume — Full System Architecture

This is a standalone architecture, we will be integrating this into Nyx (Monad), so that most of the stuff is offloaded and only the bespoke Instagram-specific logic lives here. The goal is to have a fully functional Instagram clone that can be self-hosted on free-tier infrastructure, demonstrating the power and efficiency of Rust microservices.

> An open-source, self-hostable Instagram clone built on Rust microservices.
> Feature-complete: Feed, Stories, Reels, DMs, Explore, Profiles.
> Target: $0/month on free-tier infrastructure for early-stage traffic (≤50k users).

---

## 1. Design philosophy

**Backend owns everything.** Every frontend — web, iOS, Android, desktop — is a thin API consumer. No business logic lives in the client. A CLI could drive the entire platform.

**Microservice boundaries follow domains, not verbs.** Each service owns its data, exposes a gRPC API to other services, and a REST/GraphQL API (via the gateway) to clients. Services communicate asynchronously through NATS JetStream for anything that doesn't need a synchronous response.

**Rust everywhere it matters.** The performance and memory safety of Rust lets us run more services on fewer resources — critical when we're targeting free-tier VMs with 1GB RAM per service. Where Rust would slow development velocity (ML pipelines, data science), we use Python.

---

## 2. Microservices breakdown

### 2.1 API gateway (`Uzume-gateway`)
- **Framework**: Axum (tokio-based, tower middleware ecosystem)
- **Responsibilities**: Request routing, JWT validation, rate limiting (token bucket via DragonflyDB), request/response transformation, WebSocket upgrade for real-time features, GraphQL federation endpoint
- **Key crates**: `axum`, `tower`, `jsonwebtoken`, `tonic` (gRPC client), `async-graphql`
- **Why Axum over Actix**: Axum composes with the tower middleware ecosystem. Rate limiters, tracing, compression, CORS — all reusable tower layers. Actix has its own middleware system that doesn't interop.

### 2.2 Auth service (`Uzume-auth`)
- **Responsibilities**: Registration, login (email/password + OAuth2 via Google/Apple/GitHub), JWT issuance + refresh, 2FA (TOTP), session management, password reset flows
- **Key crates**: `argon2` (password hashing), `totp-rs`, `oauth2`, `lettre` (email)
- **Data**: PostgreSQL (users_credentials table), DragonflyDB (refresh token blacklist, rate limit counters)
- **Tokens**: Short-lived access JWT (15min) + long-lived refresh token (30 days, stored hashed in DB). Refresh rotation with reuse detection.

### 2.3 User/profile service (`Uzume-users`)
- **Responsibilities**: Profile CRUD, avatar/bio management, follow/unfollow, follower/following lists, block/mute, account settings, profile search
- **Data model** (PostgreSQL):
  - `users`: id (UUIDv7), username, display_name, bio, avatar_url, is_private, is_verified, created_at
  - `follows`: follower_id, following_id, status (pending/accepted), created_at
  - `blocks`: blocker_id, blocked_id
- **Social graph**: Follow relationships stored in PostgreSQL with a materialized adjacency list. For follower counts, we maintain denormalized counters updated via NATS events (eventual consistency, accurate within seconds).
- **Key design**: UUIDv7 for all IDs — sortable by time, no coordination needed across services.

### 2.4 Post/feed service (`Uzume-feed`)
- **Responsibilities**: Post creation (photo + caption + tags + location), like/unlike, comment/reply, save/unsave, feed generation (home timeline), hashtag feeds
- **Feed strategy — hybrid push/pull (fanout-on-write + fanout-on-read)**:
  - For users with ≤10k followers: push model. On new post, write to all followers' timelines in ScyllaDB. Fast reads.
  - For users with >10k followers: pull model. Don't fan out. When a follower loads their feed, merge the celebrity's recent posts at read time.
  - This is exactly how Twitter/Instagram handle the "celebrity problem."
- **Data**:
  - PostgreSQL: `posts` (id, user_id, caption, location, created_at), `comments`, `likes`, `saves`, `hashtags`
  - ScyllaDB: `user_timeline` (user_id, post_id, score, created_at) — pre-computed feed per user, ordered by ranking score
  - DragonflyDB: hot post cache (top 1000 posts by engagement), user feed cache (last 200 items)

### 2.5 Media service (`Uzume-media`)
- **Responsibilities**: Image upload (multipart), image processing pipeline, video upload + transcoding, thumbnail generation, EXIF stripping, content-aware cropping, multiple resolution generation
- **Image pipeline** (all in Rust):
  1. Client uploads original → stored in MinIO `/raw/`
  2. NATS event triggers processing worker
  3. Worker generates variants: 1080px (feed), 640px (grid), 320px (thumbnail), 150px (avatar crop)
  4. Strip EXIF, apply optional filters (Rust bindings to libvips via `libvips-rs`)
  5. Store variants in MinIO `/processed/`, update post record with URLs
  6. CDN-ready: Cloudflare R2 or a caching reverse proxy in front of MinIO
- **Video pipeline** (Reels + Stories):
  1. Client uploads video (≤90s for Reels, ≤15s for Stories)
  2. Stored in MinIO `/raw-video/`
  3. NATS event triggers FFmpeg transcoding worker (Rust wrapper: `ffmpeg-cli-wrapper` or shell out)
  4. Transcode to H.264 at 720p, 480p, 360p + HLS segments for adaptive streaming
  5. Generate poster frame (thumbnail) at 1s mark
  6. Store HLS manifests + segments in MinIO `/video/`
- **Key crates**: `image`, `fast_image_resize`, `libvips` (via FFI), `tokio` for async I/O
- **Why not a cloud transcoding service**: FFmpeg is free, runs on ARM, and Oracle's free-tier ARM instances are powerful enough for moderate transcoding loads.

### 2.6 Stories service (`Uzume-stories`)
- **Responsibilities**: Story creation (photo/video + stickers + text overlays + polls + questions), story viewing, story expiration (24h TTL), viewer list, highlights (persisted story collections)
- **Data**:
  - PostgreSQL: `stories` (id, user_id, media_url, type, sticker_data JSON, expires_at, created_at), `story_views`, `highlights`, `highlight_items`
  - ScyllaDB: `story_feed` (user_id → list of active stories from followed users, TTL=24h)
- **Expiration**: Background Rust worker runs every minute, queries `stories WHERE expires_at < NOW()`, marks as expired, emits NATS event. Highlight stories are exempt from TTL.
- **Interactive elements**: Polls, questions, quizzes, emoji sliders stored as JSONB in `sticker_data`. Responses stored in `story_interactions` table. Results aggregated on-read with caching.

### 2.7 Reels service (`Uzume-reels`)
- **Responsibilities**: Reel creation, reel feed (algorithmic), audio track management, reel remix/duet, trending audio, reel-specific engagement metrics
- **Feed algorithm**: Separate from the main feed. Uses a lightweight scoring model:
  - `score = (likes × 1.0 + comments × 2.0 + shares × 3.0 + watch_completions × 5.0) / (age_hours ^ 1.5)`
  - Personalization layer: boost scores for content from categories the user engages with (tracked as sparse feature vectors in DragonflyDB)
  - Cold start: new users get globally trending reels until the model has ≥20 interactions
- **Audio**: Audio tracks extracted from uploaded reels, stored with fingerprint for "use this audio" feature. Track metadata in PostgreSQL, audio files in MinIO.

### 2.8 DM/messaging service (`Uzume-dm`)
- **Responsibilities**: 1:1 and group direct messages, message types (text, photo, video, reel share, post share, story reply, voice message), read receipts, typing indicators, message reactions, vanish mode
- **Protocol**: WebSocket connections through the API gateway. Each connected client maintains a persistent WS connection. Messages are:
  1. Received by gateway → forwarded to DM service via gRPC
  2. Persisted to ScyllaDB (conversation_id, message_id, sender_id, content, type, created_at)
  3. Published to NATS channel `dm.{recipient_id}`
  4. Gateway subscribers for that user's NATS channel push via WebSocket
- **Offline delivery**: If recipient is offline, message is persisted and a push notification is queued. On reconnect, client fetches unread messages via REST.
- **Encryption**: End-to-end encryption using the Signal protocol (Double Ratchet). Key exchange via X3DH. Server never sees plaintext. Implemented via `libsignal-protocol-rust` (Signal's own Rust implementation).
- **Group messages**: Up to 32 members. Server-side fan-out (each message written to each member's inbox in ScyllaDB).

### 2.9 Notification service (`Uzume-notify`)
- **Responsibilities**: In-app notifications (likes, comments, follows, mentions, story reactions), push notifications (APNs + FCM), notification preferences, notification grouping ("user1 and 42 others liked your post")
- **Architecture**: Pure event consumer. Subscribes to NATS subjects: `post.liked`, `post.commented`, `user.followed`, `story.reacted`, etc. For each event:
  1. Check recipient's notification preferences
  2. Create notification record in ScyllaDB
  3. If push enabled: queue push notification via APNs/FCM
  4. Group similar notifications (e.g., multiple likes on the same post within 5 minutes → single grouped notification)
- **Push**: `a]pns2` crate for Apple, HTTP/2 to FCM for Android. Push tokens stored in PostgreSQL.

### 2.10 Discovery/explore service (`Uzume-discover`)
- **Responsibilities**: Explore page (personalized content grid), hashtag trending, user search, content recommendation, location-based discovery
- **Search**: Meilisearch (Rust-native, fast, typo-tolerant). Indexes: users (username, display_name, bio), posts (captions, hashtags), locations. Synced from PostgreSQL via NATS events.
- **Explore algorithm**: Two-stage retrieval:
  1. **Candidate generation**: Pull top-engagement posts from the last 48h that the user hasn't seen, weighted by category affinity
  2. **Ranking**: Lightweight Rust scoring model (no ML dependency for v1). Features: post engagement rate, author follow-overlap with user's graph, content category match, recency. Score = weighted sum, top-K returned.
- **Future ML path**: When scaling justifies it, swap the ranker for a Python microservice running an ONNX model (exported from PyTorch). The Rust service calls it via gRPC. Start simple, add ML later.

---

## 3. Data architecture

### 3.1 PostgreSQL — source of truth
Single PostgreSQL instance (Neon free tier: 0.5GB storage, auto-suspend, branching). All relational data lives here. This is the system of record.

Key tables (simplified):
```
users (id UUIDv7 PK, username UNIQUE, email UNIQUE, display_name, bio, avatar_url,
       is_private BOOL, is_verified BOOL, follower_count INT, following_count INT,
       post_count INT, created_at TIMESTAMPTZ)

follows (follower_id UUID FK, following_id UUID FK, status follow_status,
         created_at TIMESTAMPTZ, PK(follower_id, following_id))

posts (id UUIDv7 PK, user_id UUID FK, caption TEXT, location JSONB,
       like_count INT, comment_count INT, type post_type, created_at TIMESTAMPTZ)

post_media (id UUIDv7 PK, post_id UUID FK, media_url TEXT, media_type media_type,
            width INT, height INT, sort_order INT)

comments (id UUIDv7 PK, post_id UUID FK, user_id UUID FK, parent_id UUID FK NULL,
          text TEXT, like_count INT, created_at TIMESTAMPTZ)

likes (user_id UUID, post_id UUID, created_at TIMESTAMPTZ, PK(user_id, post_id))

stories (id UUIDv7 PK, user_id UUID FK, media_url TEXT, media_type media_type,
         sticker_data JSONB, expires_at TIMESTAMPTZ, created_at TIMESTAMPTZ)

conversations (id UUIDv7 PK, type conv_type, created_at TIMESTAMPTZ)
conversation_members (conversation_id UUID FK, user_id UUID FK)
```

Indexes: B-tree on all foreign keys, GIN on `posts.caption` for full-text backup, partial index on `stories WHERE expires_at > NOW()` for active stories.

### 3.2 ScyllaDB — high-throughput reads
ScyllaDB (self-hosted, open-source Cassandra-compatible, written in C++ — much faster per node). Used for access patterns that are write-heavy and read-by-partition-key.

Key tables:
```
user_timeline (user_id UUID, bucket_date DATE, post_id TIMEUUID, score FLOAT,
               PRIMARY KEY ((user_id, bucket_date), post_id)) WITH CLUSTERING ORDER BY (post_id DESC)

user_activity (user_id UUID, activity_type TEXT, activity_id TIMEUUID, data TEXT,
               created_at TIMESTAMP, PRIMARY KEY (user_id, created_at)) WITH CLUSTERING ORDER BY (created_at DESC)

messages (conversation_id UUID, message_id TIMEUUID, sender_id UUID, content_encrypted BLOB,
          type TEXT, created_at TIMESTAMP, PRIMARY KEY (conversation_id, message_id))
          WITH CLUSTERING ORDER BY (message_id DESC)

story_feed (user_id UUID, story_user_id UUID, story_id UUID, created_at TIMESTAMP,
            PRIMARY KEY (user_id, created_at)) WITH default_time_to_live = 86400
```

### 3.3 DragonflyDB — caching layer
DragonflyDB (drop-in Redis replacement, multi-threaded, 25x throughput). Self-hosted, uses far less memory than Redis for the same dataset.

Cache patterns:
- `user:{id}` → serialized user profile (TTL 5min)
- `feed:{user_id}` → list of post_ids for the user's home feed (TTL 10min)
- `post:{id}` → serialized post with counts (TTL 2min)
- `session:{token}` → user_id + metadata (TTL = refresh token lifetime)
- `rate:{ip}:{endpoint}` → counter (TTL 60s, for rate limiting)
- `story_viewers:{story_id}` → set of user_ids who viewed (TTL 24h)
- `trending_hashtags` → sorted set by score (refreshed every 5min by background worker)
- `user_categories:{user_id}` → sparse vector of content category affinities (for explore personalization)

### 3.4 MinIO — object storage
S3-compatible, self-hosted. Bucket structure:
```
Uzume-media/
  raw/              # Original uploads (retained 30 days, then deleted)
  processed/        # Resized images (1080, 640, 320, 150)
  video/            # HLS segments + manifests
  avatars/          # User profile photos
  stories/          # Story media (could apply lifecycle policy for expiration)
  audio/            # Extracted audio tracks from reels
```

All media served through a Cloudflare R2 mirror or caching reverse proxy (Varnish/nginx) for CDN-like behavior on free tier.

### 3.5 Meilisearch — search engine
Rust-native full-text search. Indexes synced from PostgreSQL via NATS event consumers.

Indexes:
- `users`: searchable on username, display_name, bio. Filterable by is_verified.
- `posts`: searchable on caption. Filterable by hashtags, created_at range.
- `hashtags`: searchable on name. Sortable by post_count.
- `locations`: searchable on name. Filterable by geo coordinates (Meilisearch supports geo search).

---

## 4. Inter-service communication

### 4.1 Synchronous: gRPC + Protobuf
All service-to-service calls that need an immediate response use gRPC with protobuf schemas. Rust crate: `tonic`.

Example: when the feed service needs user profile data to render a post, it calls the user service via gRPC. Responses are cached in DragonflyDB to reduce cross-service chatter.

### 4.2 Asynchronous: NATS JetStream
Events that don't need an immediate response go through NATS JetStream (persistent, at-least-once delivery).

Key event subjects:
```
post.created      → Feed service (fanout), Media service (process), Search (index)
post.liked        → Notification service, Feed service (update score)
post.commented    → Notification service, Feed service (update score)
user.followed     → Notification service, Feed service (rebuild timeline)
user.blocked      → Feed service (filter), DM service (block)
story.created     → Story feed builder
story.expired     → Cleanup worker
media.uploaded    → Media processing pipeline
media.processed   → Post/Story service (update URLs)
dm.sent           → Gateway (WebSocket push), Notification service
```

NATS JetStream provides persistence, replay, and consumer groups — so if a service is down, it catches up on missed events when it restarts.

---

## 5. API design

### 5.1 Client-facing: REST + GraphQL hybrid
- **REST** for simple CRUD and media upload (file upload over multipart doesn't work well with GraphQL)
- **GraphQL** for complex reads (feed with nested author profiles, comments with replies, explore with mixed content types). Implemented via `async-graphql` in the gateway, which federates queries to downstream services.

### 5.2 Key REST endpoints
```
POST   /api/v1/auth/register
POST   /api/v1/auth/login
POST   /api/v1/auth/refresh
POST   /api/v1/auth/oauth/{provider}

GET    /api/v1/users/{id}
PATCH  /api/v1/users/me
POST   /api/v1/users/{id}/follow
DELETE /api/v1/users/{id}/follow
GET    /api/v1/users/{id}/followers
GET    /api/v1/users/{id}/following

POST   /api/v1/posts                    (multipart: media[] + caption + tags)
GET    /api/v1/posts/{id}
DELETE /api/v1/posts/{id}
POST   /api/v1/posts/{id}/like
DELETE /api/v1/posts/{id}/like
POST   /api/v1/posts/{id}/comments
GET    /api/v1/posts/{id}/comments

GET    /api/v1/feed                     (home timeline, paginated cursor-based)
GET    /api/v1/explore                  (discover page)
GET    /api/v1/reels/feed               (reels-specific algorithmic feed)

POST   /api/v1/stories                  (multipart: media + stickers JSON)
GET    /api/v1/stories/feed             (stories from followed users)
GET    /api/v1/stories/{id}/viewers

GET    /api/v1/dm/conversations
GET    /api/v1/dm/conversations/{id}/messages
POST   /api/v1/dm/conversations/{id}/messages
WS     /api/v1/dm/ws                    (WebSocket for real-time messaging)

GET    /api/v1/search?q={query}&type={users|posts|hashtags|locations}
GET    /api/v1/notifications
```

### 5.3 Pagination
Cursor-based everywhere (never offset-based — offset pagination breaks on insertion/deletion). Cursor = base64-encoded `(created_at, id)` tuple. This works naturally with both PostgreSQL and ScyllaDB's clustering keys.

```json
{
  "data": [...],
  "pagination": {
    "next_cursor": "eyJjIjoiMjAyNS0wMS0xNVQxMDozMDowMFoiLCJpIjoiMDE5NGU1YTItLi4uIn0=",
    "has_more": true
  }
}
```

---

## 6. Free-tier deployment strategy

This is where it gets creative. Here's how to run the entire platform for $0/month:

### 6.1 Compute: Oracle Cloud Always-Free Tier
Oracle Cloud's free tier is the backbone. It provides — permanently, not trial:
- **4 ARM Ampere A1 cores** (can be 1×4-core or 4×1-core VMs)
- **24 GB RAM** (distributable across VMs)
- **200 GB block storage**
- **10 TB/month outbound data**

Recommended VM allocation:
```
VM1 (2 cores, 12GB RAM): API Gateway + Auth + Users + Feed services
VM2 (1 core, 6GB RAM):   Media + Stories + Reels services + FFmpeg workers
VM3 (1 core, 6GB RAM):   PostgreSQL + ScyllaDB + DragonflyDB + Meilisearch + MinIO + NATS
```

Yes, VM3 is doing a lot — but at low traffic these are all lightweight. ScyllaDB idles at ~200MB, DragonflyDB at ~50MB, Meilisearch at ~100MB, NATS at ~30MB. PostgreSQL with small data fits in 1GB. Total: ~1.5GB RAM at idle with headroom.

### 6.2 Additional free resources
| Service                | Free tier                               | What we use it for                                                                  |
| ---------------------- | --------------------------------------- | ----------------------------------------------------------------------------------- |
| **Cloudflare R2**      | 10 GB storage, 1M reads, no egress fees | CDN for media. Mirror MinIO bucket → serve through Cloudflare. Zero bandwidth cost. |
| **Cloudflare Workers** | 100k requests/day                       | Edge caching logic, image resizing at edge, geo-routing                             |
| **Neon PostgreSQL**    | 0.5 GB, auto-suspend                    | Backup/read replica for PostgreSQL (or primary if Oracle's disk IO is a bottleneck) |
| **Upstash Redis**      | 10k commands/day                        | Fallback cache / rate-limit state if DragonflyDB is at capacity                     |
| **GitHub Actions**     | 2000 min/month                          | CI/CD pipeline: build Rust binaries, run tests, deploy to Oracle                    |
| **Cloudflare Pages**   | Unlimited sites                         | Host the web frontend (SPA/SSR)                                                     |
| **Fly.io**             | 3 shared VMs                            | Overflow compute if Oracle isn't enough, or for the web frontend                    |
| **Backblaze B2**       | 10 GB free                              | Off-site backup for PostgreSQL dumps                                                |

### 6.3 Container orchestration
At 3 VMs, full Kubernetes is overkill. Use **Docker Compose + Podman** on each VM:
- Each VM runs a `docker-compose.yml` defining its services
- Services are Rust binaries compiled to `linux/arm64` (cross-compiled via `cross` or built natively on ARM)
- Service discovery: hardcoded internal IPs (3 VMs) or a lightweight Consul/etcd if needed
- Zero-downtime deploys: blue-green via Compose profiles (bring up new container, health check, swap traffic, kill old)

For users who self-host at larger scale: provide Helm charts + Kubernetes manifests in the repo.

### 6.4 CI/CD pipeline
```
GitHub Push → GitHub Actions:
  1. cargo check + clippy + fmt
  2. cargo test (unit + integration against testcontainers)
  3. cross-compile to linux/arm64 (cargo-cross)
  4. Build Docker images (multi-stage: builder + distroless runtime)
  5. Push images to GitHub Container Registry (ghcr.io, free for public repos)
  6. SSH into Oracle VMs, pull new images, docker-compose up -d
```

---

## 7. Repo structure (monorepo)

```
Uzume/
├── Cargo.toml                    # Workspace root
├── proto/                        # Shared protobuf definitions
│   ├── auth.proto
│   ├── users.proto
│   ├── feed.proto
│   ├── media.proto
│   └── ...
├── crates/
│   ├── Uzume-common/                # Shared types, error handling, config, tracing
│   ├── Uzume-gateway/               # API gateway (Axum + async-graphql)
│   ├── Uzume-auth/                  # Auth service
│   ├── Uzume-users/                 # User/profile service
│   ├── Uzume-feed/                  # Feed service
│   ├── Uzume-media/                 # Media processing service
│   ├── Uzume-stories/               # Stories service
│   ├── Uzume-reels/                 # Reels service
│   ├── Uzume-dm/                    # DM/messaging service
│   ├── Uzume-notify/                # Notification service
│   ├── Uzume-discover/              # Discovery/explore service
│   └── Uzume-worker/                # Background job runner (media processing, feed fanout)
├── migrations/                   # SQL migrations (sqlx)
├── deploy/
│   ├── docker/
│   │   ├── Dockerfile.service    # Multi-stage build for any service
│   │   └── docker-compose.yml    # Per-VM compose files
│   ├── k8s/                      # Kubernetes manifests (for larger deployments)
│   └── terraform/                # Oracle Cloud infrastructure as code
├── clients/
│   ├── web/                      # React/Typescript web app
│   ├── mobile/                   # Flutter or React Native app
│   └── sdk/                      # TypeScript SDK auto-generated from OpenAPI spec
├── docs/
│   ├── architecture.md
│   ├── api.md
│   └── self-hosting.md
└── README.md
```

Cargo workspace means all crates share dependencies, compile together, and can be tested with a single `cargo test --workspace`.

---

## 8. Key Rust crate choices

| Domain                 | Crate                               | Why                                                       |
| ---------------------- | ----------------------------------- | --------------------------------------------------------- |
| HTTP framework         | `axum` 0.8+                         | Tower ecosystem, async, composable middleware             |
| gRPC                   | `tonic`                             | First-class Rust gRPC, works with tower                   |
| Database (Uzume)       | `sqlx`                              | Compile-time checked queries, async, no ORM overhead      |
| Database (Scylla)      | `scylla-rust-driver`                | Official driver, async, shard-aware                       |
| Cache (Redis protocol) | `fred`                              | Full-featured async Redis client (works with DragonflyDB) |
| Message broker         | `async-nats`                        | Official NATS Rust client                                 |
| Search                 | `meilisearch-sdk`                   | Official Rust SDK                                         |
| Object storage         | `rust-s3`                           | S3-compatible client (MinIO + R2)                         |
| Serialization          | `serde` + `serde_json`              | De facto standard                                         |
| Auth/JWT               | `jsonwebtoken`                      | Battle-tested JWT encoding/decoding                       |
| Password               | `argon2`                            | Winner of the Password Hashing Competition                |
| Image processing       | `image` + `fast_image_resize`       | Pure Rust, no C deps for basic ops                        |
| Video (FFmpeg)         | Shell out to `ffmpeg`               | FFmpeg is unbeatable; wrapping in Rust FFI isn't worth it |
| Tracing                | `tracing` + `tracing-opentelemetry` | Structured logging + distributed tracing                  |
| Config                 | `config`                            | Multi-source config (env, TOML, CLI)                      |
| Error handling         | `thiserror` + `anyhow`              | Typed errors in libraries, ergonomic errors in services   |
| Testing                | `tokio::test` + `testcontainers`    | Integration tests with real databases in Docker           |

---

## 9. Security architecture

- **Transport**: TLS everywhere. Cloudflare provides free TLS termination at the edge. Internal traffic: mutual TLS between services (via `rustls`).
- **Authentication**: JWT (RS256, asymmetric — gateway verifies with public key, auth service signs with private key). No service except auth touches the private key.
- **Authorization**: RBAC at the gateway level. Each endpoint declares required permissions. The JWT carries a minimal claim set: `{ sub, exp, iat, scopes }`.
- **DM encryption**: Signal protocol (E2EE). Server stores only ciphertext. Key bundles (identity key, signed pre-key, one-time pre-keys) stored in PostgreSQL, never exposed except to conversation participants.
- **Input validation**: All inputs validated at the gateway (request body schemas via `validator` crate) AND at the service level (defense in depth).
- **Rate limiting**: Per-IP and per-user token bucket. Stored in DragonflyDB. Limits configurable per-endpoint.
- **CSRF**: Not applicable (API is stateless, JWT in Authorization header, no cookies).
- **Media safety**: Uploaded images stripped of EXIF. Optional: integrate with a self-hosted NSFW classifier (ONNX model) in the media pipeline.

---

## 10. Observability

All free/open-source:
- **Logs**: `tracing` crate → structured JSON → Grafana Loki (self-hosted on VM3, or Grafana Cloud free tier: 50GB/month)
- **Metrics**: Prometheus (scrapes /metrics endpoints from each service) → Grafana dashboards
- **Traces**: OpenTelemetry → Jaeger (self-hosted) or Grafana Tempo
- **Alerts**: Grafana alerting → free Slack/Discord webhook notifications
- **Uptime**: Uptime Kuma (self-hosted, lightweight) or Better Stack free tier

Key metrics to track: p50/p95/p99 latency per endpoint, error rate, feed generation time, media processing queue depth, WebSocket connection count, cache hit rate.

---

## 11. Scaling path

The architecture is designed to scale horizontally. Here's the progression:

**Stage 1: 0–50k users (free tier)**
- 3 Oracle VMs, everything as described above
- Single PostgreSQL, single ScyllaDB node, single MinIO
- Total cost: $0/month

**Stage 2: 50k–500k users (~$50–200/month)**
- Add Hetzner ARM servers ($4-8/month each) for compute
- Move PostgreSQL to managed (Neon pro or Supabase pro)
- Add a second ScyllaDB node
- Cloudflare R2 handles all media serving (still near-free)
- Introduce read replicas for PostgreSQL

**Stage 3: 500k–5M users (~$500–2000/month)**
- Kubernetes (k3s cluster across 5-10 nodes)
- PostgreSQL with Citus for horizontal sharding
- ScyllaDB 3-node cluster
- Dedicated media processing workers
- Meilisearch cluster
- CDN: Cloudflare Pro or Bunny.net

**Stage 4: 5M+ users**
- Full Kubernetes on managed cloud (EKS/GKE)
- Dedicated ML inference service for recommendations
- Multi-region deployment
- Dedicated video transcoding fleet

---

## 12. Feature parity checklist

| Instagram feature                               | Uzume implementation                           | Status |
| ----------------------------------------------- | ---------------------------------------------- | ------ |
| Photo posts (single + carousel)                 | `Uzume-feed` + `Uzume-media`                   | v1     |
| Captions, hashtags, mentions                    | `Uzume-feed`, parsed + indexed in Meilisearch  | v1     |
| Like, comment, save                             | `Uzume-feed`                                   | v1     |
| Stories (photo/video, 24h TTL)                  | `Uzume-stories`                                | v1     |
| Story stickers (polls, questions, emoji slider) | `Uzume-stories` sticker_data JSONB             | v1     |
| Story highlights                                | `Uzume-stories`                                | v1     |
| Reels (short video, algorithmic feed)           | `Uzume-reels` + `Uzume-media` (FFmpeg)         | v1     |
| Reels audio reuse                               | `Uzume-reels` audio fingerprinting             | v1     |
| Direct messages (text, photo, video)            | `Uzume-dm` (WebSocket + Signal E2EE)           | v1     |
| Group DMs                                       | `Uzume-dm` (server-side fanout)                | v1     |
| Explore page                                    | `Uzume-discover` (engagement-weighted scoring) | v1     |
| Search (users, hashtags, locations)             | `Uzume-discover` + Meilisearch                 | v1     |
| Push notifications                              | `Uzume-notify` (APNs + FCM)                    | v1     |
| Profile (bio, avatar, follower counts)          | `Uzume-users`                                  | v1     |
| Private accounts + follow requests              | `Uzume-users` (follow status enum)             | v1     |
| Block, mute, restrict                           | `Uzume-users`                                  | v1     |
| Two-factor authentication                       | `Uzume-auth` (TOTP)                            | v1     |
| Live streaming                                  | WHIP/WHEP + SRS media server                   | v2     |
| AR filters                                      | MediaPipe + on-device processing               | v2     |
| Shopping/checkout                               | Separate commerce service                      | v2     |
| Ads                                             | Not applicable (open source, no ads)           | N/A    |

---

## 13. Getting started (self-hosting)

For anyone who downloads and runs this:

```bash
# Clone the repo
git clone https://github.com/Uzume/Uzume.git
cd Uzume

# Start all infrastructure (PostgreSQL, ScyllaDB, DragonflyDB, MinIO, NATS, Meilisearch)
docker compose -f deploy/docker/infra.yml up -d

# Run database migrations
cargo run --bin Uzume-migrate

# Start all services in development mode
cargo run --bin Uzume-gateway &
cargo run --bin Uzume-auth &
cargo run --bin Uzume-users &
cargo run --bin Uzume-feed &
cargo run --bin Uzume-media &
cargo run --bin Uzume-stories &
cargo run --bin Uzume-reels &
cargo run --bin Uzume-dm &
cargo run --bin Uzume-notify &
cargo run --bin Uzume-discover &

# Or, single command with the dev orchestrator:
cargo run --bin Uzume-dev     # Starts everything with hot-reload via cargo-watch

# Web client
cd clients/web && npm install && npm run dev
```

For production, every service has a `Dockerfile` and the full deployment is a single `docker compose up -d` per VM.

---

*This architecture is designed to be Instagram-complete on day one, deployable for free on day one, and scalable to millions without a rewrite. The microservice boundaries are clean enough that any single service can be rewritten or replaced without affecting the others. The Rust foundation means we're memory-safe, fast, and resource-efficient — critical when every megabyte of RAM counts on free-tier infrastructure.*