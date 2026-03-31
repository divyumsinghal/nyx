# Nyx — 6-Hour Build Plan

> 3 agents, git worktrees, TDD, production grade. No fluff.

---

## Naming Reference

| Code name      | Role                            | Type         |
| -------------- | ------------------------------- | ------------ |
| **Nun**        | nyx-common (types, errors, IDs) | Library      |
| **nyx-api**    | Axum framework layer            | Library      |
| **Mnemosyne**  | nyx-db (PG pool, migrations)    | Library      |
| **Heka**       | nyx-auth (Kratos + aliases)     | Library      |
| **nyx-events** | NATS JetStream pub/sub          | Library      |
| **Lethe**      | nyx-cache (DragonflyDB)         | Library      |
| **Akash**      | nyx-storage (MinIO/S3)          | Library      |
| **Brizo**      | nyx-search (Meilisearch)        | Library      |
| **Ogma**       | nyx-messaging (Matrix)          | Library      |
| **Oya**        | nyx-media (processing worker)   | Lib + Binary |
| **Ushas**      | nyx-notify (push + in-app)      | Lib + Binary |
| **Heimdall**   | nyx-gateway (API proxy)         | Binary       |

---

## Critical Path

```
Nun ──→ nyx-api ──→ ANY SERVICE
  │
  ├──→ Mnemosyne ──→ (queries work)
  ├──→ Lethe ──→ (caching works)
  ├──→ nyx-events ──→ (async events work)
  ├──→ Heka ──→ (auth works)
  ├──→ Akash ──→ (uploads work)
  └──→ Brizo ──→ (search works)
```

**Nun is the single bottleneck.** Once Nun compiles, everything else fans out.
**nyx-api is the second bottleneck.** Every service's main.rs needs NyxServer.

---

## Current State (already running)

| Agent   | Working on              | Worktree branch      |
| ------- | ----------------------- | -------------------- |
| Agent 1 | Nun → Heka → Uzume-feed | `feat/nun-heka-feed` |
| Agent 2 | Uzume-stories           | `feat/uzume-stories` |

---

## The Plan — 6 Phases × 3 Agents

### PHASE 0 — Right Now (minute 0)

**Start Agent 3 immediately.** It works on zero-Rust files only — no compile conflicts possible.

| Agent       | Task                               | Files touched                       |
| ----------- | ---------------------------------- | ----------------------------------- |
| **Agent 3** | **Prithvi + Migrations + Configs** | `Prithvi/`, `migrations/`, `tools/` |

Agent 3's full scope:

```
□ Prithvi/compose/infra.yml
    - PostgreSQL 17 (with nyx + uzume schemas created on init)
    - DragonflyDB
    - NATS (JetStream enabled)
    - MinIO (auto-create bucket)
    - Meilisearch
    - Ory Kratos (+ kratos-migrate)
    - Continuwuity
    - Gorush
    - Prometheus + Loki + Grafana

□ Prithvi/compose/platform.yml
    - Heimdall :3000
    - Oya worker
    - Ushas worker

□ Prithvi/compose/uzume.yml
    - Uzume-profiles :3001
    - Uzume-feed :3002
    - Uzume-stories :3003
    - Uzume-reels :3004
    - Uzume-discover :3005

□ Prithvi/compose/dev.yml (override: debug ports, logs)
□ Prithvi/compose/prod.yml (override: resource limits, restart)

□ Prithvi/docker/Dockerfile.service (multi-stage Rust → distroless)
□ Prithvi/docker/Dockerfile.worker
□ Prithvi/docker/Dockerfile.web (SvelteKit → nginx)

□ Prithvi/config/kratos/kratos.yml
□ Prithvi/config/kratos/identity.schema.json (phone + email + display_name)
□ Prithvi/config/nats/nats-server.conf (JetStream, streams)
□ Prithvi/config/continuwuity/conduwuit.toml
□ Prithvi/config/gorush/config.yml
□ Prithvi/config/prometheus/prometheus.yml
□ Prithvi/config/grafana/provisioning/ (datasources)

□ migrations/Monad/0001_create_schemas.sql
□ migrations/Monad/0002_nyx_app_aliases.sql
□ migrations/Monad/0003_nyx_app_links.sql
□ migrations/Monad/0004_nyx_push_tokens.sql

□ migrations/Uzume/0001_profiles.sql
□ migrations/Uzume/0002_posts.sql
□ migrations/Uzume/0003_interactions.sql (likes, comments, saves)
□ migrations/Uzume/0004_follows.sql
□ migrations/Uzume/0005_stories.sql (stories, views, highlights)
□ migrations/Uzume/0006_reels.sql (reels, audio)
□ migrations/Uzume/0007_notifications.sql
□ migrations/Uzume/0008_timeline.sql (user_timeline materialized feed)

□ tools/seed-data/users.json
□ tools/seed-data/uzume_posts.json

□ .env.example
□ justfile (all recipes)
```

**Why this is Agent 3's job:** It's 100% YAML/SQL/TOML/JSON. Zero Cargo conflicts. Can be built and tested (`docker compose config`, `docker compose up`) independently. When Rust agents merge, the infra is already waiting.

---

### PHASE 1 — Hours 0–1.5: Foundation

Agent 1 is already on this. The critical output is **Nun compiling + nyx-api compiling**.

| Agent       | Task                             | Depends on                                        | Output                             |
| ----------- | -------------------------------- | ------------------------------------------------- | ---------------------------------- |
| **Agent 1** | Nun → nyx-api → Mnemosyne → Heka | nothing                                           | Foundation crates compile          |
| **Agent 2** | Lethe + nyx-events + Akash       | Nun (pull from Agent 1's branch once Nun is done) | 3 platform libs compile            |
| **Agent 3** | (continues Prithvi work)         | nothing                                           | Infra ready to `docker compose up` |

**Agent 1 — detailed:**
```
NUN (nyx-common):
  □ lib.rs, id.rs (NyxId UUIDv7 newtype)
  □ time.rs (UTC helpers)
  □ error.rs (NyxError enum, HTTP status mapping, ErrorResponse)
  □ config.rs (config loading trait, env + TOML)
  □ pagination.rs (CursorRequest, CursorResponse)
  □ validation.rs (phone, email validators)
  □ types/app.rs (NyxApp enum: Uzume, Anteros, Themis)
  □ types/media.rs (MediaType, MediaVariant)
  □ types/user.rs (NyxIdentityId, AppAlias)
  □ testing.rs (random ID generators, test config)
  □ Tests for all of the above

NYX-API:
  □ server.rs (NyxServer builder)
  □ middleware/auth.rs (JWT extraction layer)
  □ middleware/rate_limit.rs (token bucket stub — calls Lethe)
  □ middleware/request_id.rs
  □ middleware/tracing.rs
  □ middleware/app_context.rs
  □ extract/auth.rs (AuthUser extractor)
  □ extract/cursor.rs (pagination extractor)
  □ extract/validated.rs (ValidatedJson<T>)
  □ response.rs (ApiResponse<T> envelope)
  □ openapi.rs (utoipa doc builder)
  □ Tests

MNEMOSYNE (nyx-db):
  □ pool.rs (PgPool builder from config)
  □ migrate.rs (per-schema migration runner)
  □ transaction.rs (auto-rollback helper)
  □ ext.rs (bulk insert, RETURNING helpers)
  □ Tests

HEKA (nyx-auth):
  □ client.rs (KratosClient wrapping reqwest)
  □ session.rs (validate_session → NyxIdentity)
  □ identity.rs (identity CRUD via admin API)
  □ types.rs (Session, Identity, Traits structs)
  □ alias.rs (resolve_alias, identity_from_alias, are_linked)
  □ Tests
```

**Agent 2 — detailed (once Nun is available):**
```
LETHE (nyx-cache):
  □ client.rs (CacheClient wrapping fred::Client)
  □ keys.rs (key namespace: {app}:{entity}:{id})
  □ rate_limit.rs (token bucket: INCR + EXPIRE)
  □ session.rs (store/retrieve validated sessions)
  □ helpers.rs (get_or_set cache-aside, TTL constants)
  □ Tests

NYX-EVENTS:
  □ client.rs (NatsClient: connect, publish, subscribe)
  □ subjects.rs (ALL event subject constants)
  □ envelope.rs (NyxEvent<T> envelope)
  □ publisher.rs (typed publish)
  □ subscriber.rs (typed subscribe → Stream<NyxEvent<T>>)
  □ Tests

AKASH (nyx-storage):
  □ client.rs (StorageClient wrapping rust-s3 Bucket)
  □ upload.rs (put_object, presigned_upload_url)
  □ download.rs (get_object, presigned_download_url)
  □ paths.rs (path convention: {app}/{entity}/{id}/{variant}.{ext})
  □ Tests
```

**MERGE POINT 1:** End of Phase 1. All 3 agents merge to `main`. After this merge:
- All platform library crates compile
- `docker compose -f Prithvi/compose/infra.yml up` works
- All migrations run

---

### PHASE 2 — Hours 1.5–3: Remaining Platform + First Services

| Agent       | Task                                     | Depends on    | Output                              |
| ----------- | ---------------------------------------- | ------------- | ----------------------------------- |
| **Agent 1** | Brizo + Ogma + Uzume-profiles            | Phase 1 merge | Search, messaging, profiles service |
| **Agent 2** | Uzume-feed (full) + Uzume-stories (full) | Phase 1 merge | Core social features                |
| **Agent 3** | Heimdall (gateway) + nyx-xtask           | Phase 1 merge | Traffic can flow end-to-end         |

**Agent 1:**
```
BRIZO (nyx-search):
  □ client.rs (SearchClient wrapping meilisearch-sdk)
  □ index.rs (create, update settings, sync)
  □ query.rs (typed search request/response)
  □ sync.rs (event-driven: NATS → Meilisearch)
  □ Tests

OGMA (nyx-messaging):
  □ client.rs (MatrixClient → Continuwuity HTTP API)
  □ rooms.rs (create_app_room, app-scoped tagging)
  □ messages.rs (send/receive, media attachments)
  □ aliases.rs (NyxId + NyxApp → Matrix user mapping)
  □ privacy.rs (filter by app tag, cross-app link checks)
  □ types.rs (Matrix event types, room metadata)
  □ Tests

UZUME-PROFILES (port 3001):
  □ main.rs (NyxServer::builder())
  □ config.rs
  □ models/profile.rs (Profile, ProfileCreate, ProfileResponse)
  □ models/follow.rs (Follow, FollowStatus)
  □ queries/profiles.rs (CRUD SQL)
  □ queries/follow.rs (follow/unfollow/followers/following SQL)
  □ services/follow_graph.rs (follow logic, private account handling)
  □ handlers/ (all profile + follow handlers)
  □ routes/ (GET/PATCH profiles, POST/DELETE follow, block)
  □ workers/profile_stub.rs (nyx.user.created → create profile)
  □ workers/search_sync.rs (profile.updated → Meilisearch)
  □ tests/api/ (integration)
  □ tests/services/ (unit)
```

**Agent 2:**
```
UZUME-FEED (port 3002):
  □ models/post.rs, comment.rs, like.rs, save.rs
  □ queries/posts.rs, feed.rs, comments.rs
  □ services/feed_builder.rs (hybrid push/pull)
  □ handlers/ (post CRUD, like, comment, feed)
  □ routes/ (all feed endpoints)
  □ workers/feed_fanout.rs (post.created → follower timelines)
  □ workers/score_updater.rs (engagement → score)
  □ workers/search_sync.rs
  □ Full integration + unit tests

UZUME-STORIES (port 3003):
  □ models/story.rs, highlight.rs, interaction.rs
  □ queries/stories.rs, highlights.rs
  □ services/story_lifecycle.rs (24h TTL, highlights exempt)
  □ handlers/ (create story, feed, viewers, interact, highlights)
  □ routes/
  □ workers/story_expiry.rs (every 60s, expire past-TTL)
  □ Full integration + unit tests
```

**Agent 3:**
```
HEIMDALL (nyx-gateway, port 3000):
  □ main.rs
  □ config.rs (upstream URLs, rate limit settings)
  □ proxy.rs (reverse proxy: match prefix → forward)
  □ websocket.rs (authenticate → forward WS)
  □ health.rs (/healthz, upstream aggregation)
  □ routes.rs (full route table: /api/nyx/*, /api/uzume/*)
  □ Tests

NYX-XTASK:
  □ main.rs (CLI with subcommands)
  □ migrate command (run all pending migrations)
  □ db-reset command (drop + recreate schemas)
  □ seed command (load tools/seed-data/)
  □ new-app scaffold command
```

**MERGE POINT 2:** End of Phase 2. After this merge:
- Gateway routes traffic to services
- User can register (Kratos) → create profile → create post → see feed → create story
- Search indexes populated
- **This is the first end-to-end flow.**

---

### PHASE 3 — Hours 3–4.5: Reels, Discover, Workers

| Agent       | Task                                       | Depends on    | Output                      |
| ----------- | ------------------------------------------ | ------------- | --------------------------- |
| **Agent 1** | Uzume-reels (full service)                 | Phase 2 merge | Short-form video works      |
| **Agent 2** | Uzume-discover (full service)              | Phase 2 merge | Explore + search works      |
| **Agent 3** | Oya (media worker) + Ushas (notify worker) | Phase 2 merge | Background processing works |

**Agent 1:**
```
UZUME-REELS (port 3004):
  □ models/reel.rs, reel_audio.rs
  □ queries/reels.rs, audio.rs
  □ services/reel_ranker.rs (score formula + personalization)
  □ handlers/ (create reel, feed, like, view, audio)
  □ routes/
  □ workers/search_sync.rs
  □ Full tests
```

**Agent 2:**
```
UZUME-DISCOVER (port 3005):
  □ models/ (trending types, search result types)
  □ queries/ (engagement data, trending aggregation)
  □ services/discover_ranker.rs (two-stage: candidates → ranking)
  □ services/trending.rs (hashtags, reels, audio)
  □ handlers/ (explore, trending, search)
  □ routes/
  □ workers/trending_updater.rs (every 5min recalculate)
  □ workers/search_sync.rs
  □ Full tests
```

**Agent 3:**
```
OYA (nyx-media worker):
  □ image.rs (resize, EXIF strip, generate 1080/640/320/150 variants)
  □ video.rs (FFmpeg transcode, HLS segmentation, thumbnails)
  □ pipeline.rs (orchestrator: raw → process → store → emit event)
  □ config.rs (variant definitions per entity type)
  □ bin/worker.rs (NATS subscriber: *.media.uploaded → process)
  □ Tests

USHAS (nyx-notify worker):
  □ gorush.rs (Gorush HTTP client: APNs/FCM)
  □ in_app.rs (PostgreSQL storage + WebSocket push)
  □ grouping.rs ("X and 42 others liked your post")
  □ preferences.rs (per-app, per-event-type mute)
  □ bin/worker.rs (NATS subscriber → dispatch)
  □ Tests
```

**MERGE POINT 3:** End of Phase 3. After this merge:
- All 5 Uzume services running
- Media uploads get processed into variants
- Push notifications fire on interactions
- Explore page returns personalized content
- **Feature-complete backend.**

---

### PHASE 4 — Hours 4.5–5.5: Frontend

| Agent       | Task                                       | Depends on                                      | Output                             |
| ----------- | ------------------------------------------ | ----------------------------------------------- | ---------------------------------- |
| **Agent 1** | Maya/shared (@nyx/ui) + Maya/nyx-web       | Phase 3 merge                                   | Shared components + account portal |
| **Agent 2** | Maya/Uzume-web (core pages)                | Phase 3 merge (+ shared from Agent 1 mid-phase) | Social media frontend              |
| **Agent 3** | CI/CD + Dockerfiles + E2E test scaffolding | Phase 3 merge                                   | Pipeline works                     |

**Agent 1:**
```
MAYA/SHARED (@nyx/ui):
  □ SvelteKit lib setup, package.json
  □ components/Auth/ (LoginForm, RegisterForm)
  □ components/Common/ (Button, Input, Modal, Avatar, Skeleton, Toast)
  □ components/Media/ (ImageUploader, VideoPlayer, Carousel)
  □ components/Notification/ (NotificationBell, NotificationList)
  □ components/Chat/ (ChatWindow, MessageBubble — Matrix integration)
  □ stores/auth.ts (session, JWT management)
  □ stores/notifications.ts (WebSocket stream)
  □ api/client.ts (fetch wrapper with JWT injection)
  □ api/uzume.ts (typed Uzume API calls)
  □ api/nyx.ts (typed Nyx API calls)

MAYA/NYX-WEB (account portal):
  □ routes/+layout.svelte (shell)
  □ routes/login/ (Kratos login flow)
  □ routes/register/ (Kratos registration flow)
  □ routes/settings/ (profile, linked apps, notification prefs)
```

**Agent 2:**
```
MAYA/UZUME-WEB:
  □ svelte.config.js, vite.config.ts, app.css (Uzume branding)
  □ routes/+layout.svelte (app shell: navbar, sidebar)
  □ routes/+page.svelte (home feed)
  □ routes/explore/ (discover page)
  □ routes/reels/ (reels feed — vertical scroll)
  □ routes/messages/ (Matrix DMs)
  □ routes/notifications/
  □ routes/profile/[alias]/ (public profile)
  □ routes/post/[id]/ (single post view)
  □ routes/settings/
  □ lib/Feed/ (FeedPost, FeedList components)
  □ lib/Stories/ (StoryBar, StoryViewer)
  □ lib/Reels/ (ReelPlayer, ReelFeed)
  □ lib/Profile/ (ProfileHeader, FollowerList)
  □ lib/Post/ (PostCard, CommentList, LikeButton)
```

**Agent 3:**
```
CI/CD:
  □ .github/workflows/ci.yml (lint, test, build on PR)
  □ .github/workflows/release.yml (tag → Docker build → GHCR)
  □ .github/workflows/deploy.yml (SSH → docker compose up)
  □ CODEOWNERS, PR template, issue templates

DOCKERFILES (verify + polish):
  □ Test Dockerfile.service builds for each binary
  □ Test Dockerfile.worker builds
  □ Test Dockerfile.web builds
  □ Verify compose files start everything

E2E SCAFFOLDING:
  □ Playwright config
  □ Test: register → login → create post → see in feed
  □ Test: create story → see in story feed → expires
```

**MERGE POINT 4:** End of Phase 4. After this merge:
- Frontend talks to backend through gateway
- CI pipeline runs on push
- Docker builds work for all services

---

### PHASE 5 — Hours 5.5–6: Integration, Polish, Ship

All 3 agents work on the **same branch** now (or quickly merge + fix).

| Agent       | Task                                                                                    |
| ----------- | --------------------------------------------------------------------------------------- |
| **Agent 1** | End-to-end smoke test: `docker compose up` entire stack, hit every endpoint             |
| **Agent 2** | Fix any broken tests, missing error handling, edge cases found in integration           |
| **Agent 3** | README.md, CONTRIBUTING.md, docs/architecture.md final version, `just` recipes verified |

**Exit criteria — what "done" means at hour 6:**
```
✓ `just dev-infra` starts all 11 infrastructure services
✓ `just dev-uzume` starts all 5 Uzume services + gateway + workers
✓ User can register via Kratos, get JWT
✓ User can create profile with avatar
✓ User can create post (photo upload → processed variants)
✓ User can see home feed (cursor-paginated)
✓ User can like/comment on posts
✓ User can follow/unfollow
✓ User can create/view stories (24h TTL working)
✓ User can upload reels (video transcoded to HLS)
✓ User can browse explore page
✓ User can search users/posts/hashtags
✓ Push notifications fire (via Gorush)
✓ DMs work via Matrix (Continuwuity)
✓ All unit tests pass
✓ All integration tests pass
✓ Frontend renders and basic flows work
✓ CI pipeline passes
✓ Docker images build
```

---

## Parallelism Rules (avoiding conflicts with worktrees)

### File ownership per agent (strict boundaries)

| Files / directories                                               | Owner                |
| ----------------------------------------------------------------- | -------------------- |
| `Monad/Nun/`, `Monad/nyx-api/`, `Monad/Mnemosyne/`, `Monad/Heka/` | Agent 1 (Phase 1)    |
| `Monad/Lethe/`, `Monad/nyx-events/`, `Monad/Akash/`               | Agent 2 (Phase 1)    |
| `Prithvi/`, `migrations/`, `tools/`, `.env.example`, `justfile`   | Agent 3 (Phases 0–2) |
| `Monad/Brizo/`, `Monad/Ogma/`, `apps/Uzume/Uzume-profiles/`       | Agent 1 (Phase 2)    |
| `apps/Uzume/Uzume-feed/`, `apps/Uzume/Uzume-stories/`             | Agent 2 (Phase 2)    |
| `Monad/Heimdall/`, `Monad/nyx-xtask/`                             | Agent 3 (Phase 2)    |
| `apps/Uzume/Uzume-reels/`                                         | Agent 1 (Phase 3)    |
| `apps/Uzume/Uzume-discover/`                                      | Agent 2 (Phase 3)    |
| `Monad/Oya/`, `Monad/Ushas/`                                      | Agent 3 (Phase 3)    |
| `Maya/shared/`, `Maya/nyx-web/`                                   | Agent 1 (Phase 4)    |
| `Maya/Uzume-web/`                                                 | Agent 2 (Phase 4)    |
| `.github/`, `docs/`                                               | Agent 3 (Phase 4)    |

### Root Cargo.toml strategy

**Only Agent 1 modifies root Cargo.toml.** Other agents create their crate directories with their own Cargo.toml, but Agent 1 adds workspace members during merge points. This prevents merge conflicts on the most contended file.

### Merge cadence

- Merge after each phase (every ~1.5 hours)
- Each agent creates a PR from their worktree branch
- You (human) do the merge + resolve any trivial conflicts
- All agents pull fresh `main` before starting next phase

---

## TDD Strategy

### Per-crate test pattern

```
1. Write the types/traits first (models/, types in services/)
2. Write failing unit tests for services/ (pure logic, mock data)
3. Implement services/ until tests pass
4. Write failing integration tests (testcontainers: real PG, real DragonflyDB)
5. Implement handlers/ + routes/ + queries/ until integration tests pass
6. Add edge cases: bad input, unauthorized, not found, rate limited
```

### Test infrastructure (set up once in Nun)

```rust
// Nun/src/testing.rs — shared test utilities
pub fn random_nyx_id() -> NyxId { ... }
pub fn test_config() -> NyxConfig { ... }
pub async fn test_db_pool() -> PgPool { /* testcontainers */ }
pub async fn test_cache() -> CacheClient { /* testcontainers */ }
pub async fn test_nats() -> NatsClient { /* testcontainers */ }
```

### What to test (priority order for 6 hours)

1. **Unit tests for every `services/` module** — pure logic, fast, no I/O
2. **Integration tests for critical paths** — create post, feed generation, follow, story lifecycle
3. **Integration tests for auth flow** — JWT validation, alias resolution
4. **Skip for now:** E2E Playwright tests (scaffold only), load tests

---

## Anteros Introduction Point

**Not in the first 6 hours.** Introduce Anteros after Uzume is stable.

When ready (could be the next session):

| Agent   | Task                                                                         | Time estimate |
| ------- | ---------------------------------------------------------------------------- | ------------- |
| Agent 1 | `apps/Anteros/Anteros-profiles/` + `Anteros-matching/` + `Anteros-discover/` | 3 hours       |
| Agent 2 | `migrations/Anteros/` + fair-show algorithm + tests                          | 2 hours       |
| Agent 3 | `Maya/Anteros-web/` + compose file + gateway routes                          | 2 hours       |

Zero platform crate changes needed. Just:
- Add `NyxApp::Anteros` to Nun enum
- Add NATS subjects in nyx-events
- Add routes in Heimdall
- New compose file + migrations

The fair-show algorithm (guaranteed visibility within 100 swipes) is the only novel logic. Everything else reuses platform crates directly.

---

## Quick Reference — Agent Prompts

### Agent 3 startup prompt (start NOW):

> You are building the infrastructure layer for Nyx. Read the CLAUDE.md and nyx-architecture.md for full context. Your job:
>
> 1. Create ALL Docker Compose files in `Prithvi/compose/` (infra.yml, platform.yml, uzume.yml, dev.yml, prod.yml)
> 2. Create ALL config files in `Prithvi/config/` (kratos, nats, continuwuity, gorush, prometheus, grafana)
> 3. Create ALL SQL migrations in `migrations/Monad/` and `migrations/Uzume/`
> 4. Create Dockerfiles in `Prithvi/docker/`
> 5. Create `.env.example` and `justfile` at root
> 6. Create seed data in `tools/seed-data/`
>
> Do NOT touch any Rust files. Do NOT modify Cargo.toml. Test compose files with `docker compose config`.
> Write production-grade configs. Every migration must be idempotent. Use UUIDv7 for all IDs. Use TIMESTAMPTZ for all timestamps.

### For reassigning agents between phases:

Each agent's prompt at phase start should include:
1. "Read CLAUDE.md for full project context"
2. "You own ONLY these directories: [list]"
3. "Do NOT modify: root Cargo.toml, any file outside your scope"
4. "TDD: write failing tests first, then implement"
5. The specific task list from this plan