# CLAUDE.md — Nyx Project Context

> This file provides context for Claude when working on the Nyx codebase. It is the single source of project knowledge across sessions. Update it as the project evolves.

## What is Nyx

Nyx is an open-source, privacy-first ecosystem of apps. The tagline: "Own your digital experience." It replaces manipulative platforms with transparent, user-respecting alternatives.

Nyx is the **platform layer** — reusable microservices and library crates that any app can compose. Apps are thin domain-specific layers built on top. The litmus test for what belongs in Nyx vs an app: **"Would even a slightly different app clone also need this?"** Yes → Nyx. No → app-specific.

**License**: AGPL-3.0 across the entire repository. All code, all apps. This prevents closed forks while keeping everything free.

## Current apps

| App          | Name        | What it is                     | Status             |
| ------------ | ----------- | ------------------------------ | ------------------ |
| Platform     | **Nyx**     | Super-account, shared services | Active development |
| Social media | **Uzume**   | Social media app               | Active development |
| Dating       | **Anteros** | Dating App                     | Planned            |
| Housing      | **Themis**  | levels.fyi for housing         | Planned            |

Focus is currently on **Nyx + Uzume** only. Anteros and Themis are future work but the architecture must support them without structural changes.

## Architecture — the key decisions

### Microservices, not monoliths
Each app is composed of multiple microservices, each its own binary crate, process, and container. Uzume has 5 services: `Uzume-profiles`, `Uzume-feed`, `Uzume-stories`, `Uzume-reels`, `Uzume-discover`. Platform has 3 processes: `nyx-gateway`, `nyx-media` worker, `nyx-notify` worker.

### Platform crates are libraries, not services
`nyx-auth`, `nyx-db`, `nyx-cache`, `nyx-events`, `nyx-storage`, `nyx-search`, `nyx-messaging` are Rust library crates. They compile directly into app service binaries. Zero network hops to the platform layer. They contain typed HTTP clients that talk to actual infrastructure (Kratos, Continuwuity, Meilisearch, etc.).

### REST everywhere
One protocol: HTTP/JSON. No gRPC, no protobuf. Shared Rust types via library crates provide compile-time safety without code generation. Debuggable with curl.

### NATS JetStream for async events
Synchronous service-to-service calls use HTTP (cached in DragonflyDB). Asynchronous events (post.created triggers media processing, search indexing, notifications) use NATS JetStream with at-least-once delivery.

### Privacy-isolated messaging
One Continuwuity (Matrix) homeserver shared across all apps. Each app creates rooms tagged with `nyx.app` metadata. A user's Anteros match cannot discover their Uzume profile unless the user explicitly opts in via cross-app linking in Nyx account settings. The `nyx-messaging` library crate enforces this.

### App-scoped aliases
Every Nyx user has one Kratos identity (phone + email). Each app creates an **app-scoped alias** — the only identifier visible within that app. Aliases are stored in the `nyx.app_aliases` PostgreSQL table, not in Kratos.

### One PostgreSQL, separate schemas
One instance, one connection pool. Schemas: `nyx` (aliases, links, push tokens), `Uzume` (profiles, posts, stories, reels), `Anteros` (future), `Themis` (future). Migrations are per-schema in `migrations/{app}/`.

## Monorepo structure

```
nyx/
├── Cargo.toml                 # Rust workspace (all platform + app crates)
├── rust-toolchain.toml        # Rust 1.85, clippy, rustfmt, rust-analyzer
├── justfile                   # Task runner
├── package.json + pnpm-workspace.yaml  # Frontend workspace
│
├── platform/                  # Nyx shared crates
│   ├── nyx-common/            # Types, errors, config, IDs (UUIDv7)
│   ├── nyx-api/               # Axum framework: NyxServer builder, middleware, extractors
│   ├── nyx-db/                # PostgreSQL pool, migrations, transactions
│   ├── nyx-auth/              # Ory Kratos client + app-scoped alias system
│   ├── nyx-events/            # NATS JetStream typed pub/sub
│   ├── nyx-cache/             # DragonflyDB client, rate limiting, sessions
│   ├── nyx-storage/           # MinIO/S3 client, presigned URLs
│   ├── nyx-search/            # Meilisearch client, index management
│   ├── nyx-messaging/         # Matrix/Continuwuity client, privacy enforcement
│   ├── nyx-media/             # Image/video processing (lib + worker binary)
│   ├── nyx-notify/            # Push + in-app notifications (lib + worker binary)
│   ├── nyx-gateway/           # API gateway binary (thin reverse proxy)
│   └── nyx-xtask/             # Dev CLI: migrate, seed, openapi
│
├── apps/Uzume/                # Uzume microservices (Instagram clone)
│   ├── Uzume-profiles/        # Profiles, follow graph, block/mute (:3001)
│   ├── Uzume-feed/            # Posts, likes, comments, home timeline (:3002)
│   ├── Uzume-stories/         # Stories, highlights, 24h TTL (:3003)
│   ├── Uzume-reels/           # Short-form video, algorithmic feed (:3004)
│   └── Uzume-discover/        # Explore page, trending, search (:3005)
│
├── clients/                   # Frontend (pnpm workspace)
│   ├── shared/                # @nyx/ui Svelte component library
│   ├── Uzume-web/             # Uzume SvelteKit app
│   └── nyx-web/               # Nyx account portal
│
├── infra/                     # Deployment
│   ├── compose/               # infra.yml, platform.yml, Uzume.yml, dev.yml, prod.yml
│   ├── docker/                # Dockerfile.service, Dockerfile.worker, Dockerfile.web
│   ├── config/                # Kratos, Continuwuity, Gorush, NATS, Prometheus, Grafana
│   ├── terraform/             # Oracle Cloud + Cloudflare
│   └── scripts/               # deploy.sh, backup.sh, health-check.sh
│
├── migrations/                # SQL migrations per schema
│   ├── platform/              # nyx schema
│   └── Uzume/                 # Uzume schema
│
├── docs/                      # Architecture, API docs, ADRs
└── tools/                     # Seed data, benchmarks (k6), scripts
```

## Tech stack — final choices

### Rust backend

| Concern           | Choice                                   | Why                                         |
| ----------------- | ---------------------------------------- | ------------------------------------------- |
| HTTP framework    | **Axum 0.8**                             | Tower middleware, async, Tokio team         |
| Database          | **sqlx 0.8**                             | Compile-time checked SQL, no ORM            |
| Cache client      | **fred 10**                              | Async Redis client (works with DragonflyDB) |
| Event bus client  | **async-nats 0.39**                      | Official NATS Rust client                   |
| S3 client         | **rust-s3 0.36**                         | S3-compatible (MinIO + R2)                  |
| Search client     | **meilisearch-sdk 0.28**                 | Official Rust SDK                           |
| HTTP client       | **reqwest 0.12**                         | For calling Kratos, Gorush, inter-service   |
| JWT               | **jsonwebtoken 9**                       | JWT encoding/decoding                       |
| IDs               | **uuid 1** (v7)                          | Time-sortable, no coordination              |
| Serialization     | **serde + serde_json**                   | Standard                                    |
| Errors            | **thiserror** (libs) + **anyhow** (bins) | Community standard pairing                  |
| Tracing           | **tracing + tracing-opentelemetry**      | Structured logging + distributed tracing    |
| Config            | **config 0.14**                          | Multi-source (env + TOML)                   |
| Validation        | **validator 0.19**                       | Derive-based validation                     |
| API docs          | **utoipa 5**                             | OpenAPI from Rust types                     |
| Image processing  | **fast_image_resize + image**            | Pure Rust, SIMD                             |
| Video processing  | **FFmpeg** (shell out)                   | Unbeatable, not worth wrapping              |
| Test runner       | **cargo-nextest**                        | 3-6x faster than cargo test                 |
| Integration tests | **testcontainers 0.24**                  | Real DBs in Docker                          |

### Infrastructure (don't rewrite, deploy as services)

| Concern            | Tool                            | License            |
| ------------------ | ------------------------------- | ------------------ |
| Identity/auth      | **Ory Kratos**                  | Apache 2.0         |
| Messaging/DMs      | **Continuwuity** (Matrix, Rust) | Apache 2.0         |
| Push notifications | **Gorush**                      | MIT                |
| Search engine      | **Meilisearch**                 | MIT                |
| Event bus          | **NATS JetStream**              | Apache 2.0         |
| Object storage     | **MinIO**                       | AGPL 3.0           |
| Cache              | **DragonflyDB**                 | BSL 1.1            |
| Database           | **PostgreSQL 17**               | PostgreSQL License |
| Metrics            | **Prometheus**                  | Apache 2.0         |
| Logs               | **Grafana Loki**                | AGPL 3.0           |
| Dashboards         | **Grafana**                     | AGPL 3.0           |

### Frontend

| Concern         | Choice                      |
| --------------- | --------------------------- |
| Framework       | **SvelteKit**               |
| Package manager | **pnpm**                    |
| Matrix SDK      | **matrix-js-sdk**           |
| E2E tests       | **Playwright**              |
| Hosting         | **Cloudflare Pages** (free) |

### Deployment

| Concern          | Choice                                               |
| ---------------- | ---------------------------------------------------- |
| Compute          | **Oracle Cloud Always-Free** (4 ARM cores, 24GB RAM) |
| CDN/media        | **Cloudflare R2** (zero egress)                      |
| Container images | **Multi-stage Rust builder → distroless runtime**    |
| CI/CD            | **GitHub Actions** → GHCR → SSH deploy               |
| Task runner      | **just**                                             |

## Conventions

### Rust crate internal structure (every microservice follows this)

```
Uzume-{service}/
├── Cargo.toml
├── src/
│   ├── main.rs            # NyxServer::builder() → serve
│   ├── config.rs          # Service-specific config
│   ├── routes/            # Axum Router definitions
│   ├── handlers/          # Extract → service call → response
│   ├── services/          # Business logic (pure, testable, no I/O)
│   ├── models/            # Domain types + sqlx FromRow structs
│   ├── queries/           # Raw SQL (sqlx::query_as! macros)
│   └── workers/           # Background tokio tasks (NATS subscribers)
└── tests/
    ├── api/               # Integration tests (testcontainers)
    └── services/          # Unit tests (pure logic, mock data)
```

### Naming conventions

- NATS subjects: `{app}.{entity}.{action}` (e.g., `Uzume.post.created`)
- Cache keys: `{app}:{entity}:{id}` (e.g., `Uzume:user:abc123`)
- Storage paths: `{app}/{entity}/{id}/{variant}.{ext}`
- Meilisearch indexes: `{app}_{entity}` (e.g., `Uzume_posts`)
- PostgreSQL schemas: one per app (`nyx`, `Uzume`, `Anteros`, `Themis`)
- Migration files: `migrations/{app}/NNNN_description.sql`

### API conventions

- All endpoints return `ApiResponse<T>`: `{ data: T, pagination?: CursorResponse }`
- All list endpoints use cursor-based pagination (never offset)
- All errors return `ErrorResponse`: `{ error, code, request_id }`
- All mutation endpoints require JWT auth (validated by gateway)
- OpenAPI docs auto-generated via `utoipa` at `/docs`

### Every service gets for free (via NyxServer builder)

- Auth middleware (JWT validation via nyx-auth)
- Rate limiting (token bucket via DragonflyDB)
- Request ID injection + propagation
- Structured logging + OpenTelemetry tracing
- CORS
- `/healthz` endpoint
- Graceful shutdown

## Service ports (Uzume deployment)

| Service           | Port                |
| ----------------- | ------------------- |
| nyx-gateway       | 3000                |
| Uzume-profiles    | 3001                |
| Uzume-feed        | 3002                |
| Uzume-stories     | 3003                |
| Uzume-reels       | 3004                |
| Uzume-discover    | 3005                |
| nyx-media worker  | — (NATS subscriber) |
| nyx-notify worker | — (NATS subscriber) |

## Gateway routing

```
/api/nyx/auth/*          →  Ory Kratos
/api/nyx/account/*       →  Kratos + alias enrichment
/api/nyx/messaging/*     →  Continuwuity (Matrix CS API)
/api/Uzume/profiles/*     →  Uzume-profiles :3001
/api/Uzume/feed/*         →  Uzume-feed :3002
/api/Uzume/stories/*      →  Uzume-stories :3003
/api/Uzume/reels/*        →  Uzume-reels :3004
/api/Uzume/discover/*     →  Uzume-discover :3005
```

## NATS event map

```
nyx.user.created             → Uzume-profiles (create stub)
nyx.user.deleted             → all Uzume services (cascade)
Uzume.post.created            → nyx-media, nyx-search, nyx-notify, Uzume-feed (fanout)
Uzume.post.liked              → nyx-notify, Uzume-feed (score)
Uzume.comment.created         → nyx-notify, Uzume-feed (score)
Uzume.user.followed           → nyx-notify, Uzume-feed (timeline)
Uzume.user.blocked            → Uzume-feed (filter), Uzume-discover (filter)
Uzume.story.created           → nyx-media, nyx-notify
Uzume.story.viewed            → nyx-notify
Uzume.reel.created            → nyx-media (transcode), nyx-search
Uzume.reel.viewed             → Uzume-reels (scoring)
Uzume.profile.updated         → nyx-search
Uzume.media.processed         → Uzume-feed / Uzume-stories / Uzume-reels (update URLs)
```

## Crate dependency hierarchy (no cycles)

```
nyx-common → nyx-db, nyx-cache, nyx-events, nyx-storage, nyx-search, nyx-auth, nyx-messaging
         └→ nyx-api → nyx-media, nyx-notify, nyx-gateway
                   └→ Uzume-profiles, Uzume-feed, Uzume-stories, Uzume-reels, Uzume-discover
```

## Adding a new app

1. Create `apps/{app}/` with domain microservices
2. Add workspace members in root `Cargo.toml`
3. Create `migrations/{app}/` with own PostgreSQL schema
4. Add variant to `NyxApp` enum in `nyx-common`
5. Add NATS subjects in `nyx-events`
6. Add gateway routes in `nyx-gateway`
7. Add compose file in `infra/compose/{app}.yml`
8. Create `clients/{app}-web/` SvelteKit app

Zero changes to platform crate code. Zero changes to other apps.

## Project reference docs

These files in the project contain the full detailed specs:
- `nyx-architecture.md` — definitive folder-by-folder structure, all APIs, data models, service boundaries
- `nyx-ecosystem-brainstorm.md` — naming research, "don't rewrite" toolkit, privacy model, fair dating algorithm, license rationale
- `pixelgram-architecture.md` — original Pixelgram design (Uzume's origin), full data schemas, feed strategy, scaling path, security architecture

## Current state

<!-- UPDATE THIS SECTION AS THE PROJECT PROGRESSES -->
- [ ] Repository initialized
- [ ] `nyx-common` crate created
- [ ] `nyx-api` crate created (NyxServer builder)
- [ ] `nyx-db` crate created
- [ ] `nyx-auth` crate created (Kratos client + aliases)
- [ ] `nyx-events` crate created
- [ ] `nyx-cache` crate created
- [ ] `nyx-storage` crate created
- [ ] `nyx-search` crate created
- [ ] `nyx-messaging` crate created
- [ ] `nyx-gateway` binary
- [ ] `nyx-media` worker binary
- [ ] `nyx-notify` worker binary
- [ ] `Uzume-profiles` service
- [ ] `Uzume-feed` service
- [ ] `Uzume-stories` service
- [ ] `Uzume-reels` service
- [ ] `Uzume-discover` service
- [ ] `infra/compose/infra.yml` (all infrastructure)
- [ ] `infra/config/kratos/` (identity schema)
- [ ] `migrations/platform/` (nyx schema)
- [ ] `migrations/Uzume/` (Uzume schema)
- [ ] `clients/shared/` (@nyx/ui)
- [ ] `clients/Uzume-web/`
- [ ] `clients/nyx-web/`
- [ ] CI/CD pipeline (.github/workflows/)

## Style and quality

- `#![warn(clippy::pedantic)]` across the workspace
- `cargo fmt` enforced in CI
- `cargo deny check` for license + vulnerability auditing
- Every `services/` module has unit tests (pure logic, no I/O)
- Every service has integration tests in `tests/api/` (testcontainers)
- Frontend E2E tests with Playwright
- ADRs in `docs/decisions/` for every significant choice