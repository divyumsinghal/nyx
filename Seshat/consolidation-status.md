# Nyx Consolidation Status
> Generated: 2026-04-01 after branch consolidation into `Nun`

## What Was Done (This Session)

### Branch Consolidation
All work merged into `Nun` branch:
- `feat/nyx-step2-stories` → merged (added Akash, Lethe, Mnemosyne, Oya, nyx-api)
- `origin/copilot/continue-nyx-uxume-monad-nun-foundation` → selectively salvaged (Brizo, Ogma, Heka alias)
- Conflicts resolved: kept Nun's Heimdall/xtask (full impl), took stories' full NATS events

---

## Phase Completion Status

### Phase 0 — Infrastructure (COMPLETE ✅)
| Item                                | Status |
| ----------------------------------- | ------ |
| `Prithvi/compose/infra.yml`         | ✅ Done |
| `Prithvi/compose/platform.yml`      | ✅ Done |
| `Prithvi/compose/uzume.yml`         | ✅ Done |
| `Prithvi/compose/dev.yml`           | ✅ Done |
| `Prithvi/compose/prod.yml`          | ✅ Done |
| `Prithvi/docker/Dockerfile.service` | ✅ Done |
| `Prithvi/docker/Dockerfile.worker`  | ✅ Done |
| `Prithvi/config/kratos/`            | ✅ Done |
| `Prithvi/config/nats/`              | ✅ Done |
| `Prithvi/config/prometheus/`        | ✅ Done |
| `Prithvi/config/grafana/`           | ✅ Done |
| `migrations/Monad/0001-0004`        | ✅ Done |
| `migrations/Uzume/0001-0008`        | ✅ Done |
| `tools/seed-data/`                  | ✅ Done |
| `.env.example`                      | ✅ Done |
| `justfile`                          | ✅ Done |

### Phase 1 — Platform Foundation (COMPLETE ✅)
| Crate             | Status | Notes                                                     |
| ----------------- | ------ | --------------------------------------------------------- |
| `Monad/Nun`       | ✅ Done | Types, errors, config, IDs, pagination, validation        |
| `Monad/Heka`      | ✅ Done | KratosClient, AppAlias, AliasResolver, LinkPolicyEngine   |
| `Monad/events`    | ✅ Done | Full NATS JetStream + compat layer (EventPublisher trait) |
| `Monad/Mnemosyne` | ✅ Done | PG pool, migrations, transactions, cursor pagination      |
| `Monad/Lethe`     | ✅ Done | DragonflyDB cache, rate limiting, sessions, helpers       |
| `Monad/Akash`     | ✅ Done | S3/MinIO client, presigned URLs, path conventions         |
| `Monad/api`       | ✅ Done | NyxServer builder, middleware, extractors                 |
| `Monad/Heimdall`  | ✅ Done | Full API gateway with JWT auth, proxy, WS, health         |
| `Monad/xtask`     | ✅ Done | CLI: migrate, seed, db-reset, new-app scaffold            |

### Phase 2 Agent 1 — Brizo + Ogma + Uzume-profiles (PARTIAL ⚠️)
| Item             | Status        | Notes                                                                                      |
| ---------------- | ------------- | ------------------------------------------------------------------------------------------ |
| `Monad/Brizo`    | ⚠️ Basic       | SearchClient + index constants. Missing: query.rs, sync.rs                                 |
| `Monad/Ogma`     | ⚠️ Basic       | MatrixClient, PrivacyGuard, room types. Missing: messages.rs, aliases.rs                   |
| `Monad/Ushas`    | ❌ Stub        | Cargo.toml + empty lib.rs only                                                             |
| `Uzume-profiles` | ❌ Domain Only | Has domain models + in-memory service. Missing HTTP handlers, sqlx queries, workers, tests |

### Phase 2 Agent 2 — Uzume-feed + Uzume-stories (NOT STARTED ❌)
| Item                       | Status    |
| -------------------------- | --------- |
| `Uzume-feed` HTTP handlers | ❌ Missing |
| `Uzume-stories` service    | ❌ Missing |

### Phase 2 Agent 3 — Already Complete ✅
| Item             | Status                                   |
| ---------------- | ---------------------------------------- |
| `Monad/Heimdall` | ✅ Done (was in Nun before consolidation) |
| `Monad/xtask`    | ✅ Done (was in Nun before consolidation) |

### Phase 3+ (NOT STARTED ❌)
- `Uzume-reels` service
- `Uzume-discover` service
- `Monad/Oya` (has lib structure, needs full implementation)
- `Monad/Ushas` (stub only)
- Frontend (Maya/)

---

## Current Workspace Members
```
Monad/Nun, Heka, Mnemosyne, Lethe, events, Akash, Brizo, Ogma, api, Oya, Ushas, Heimdall, xtask
apps/Uzume/Uzume-profiles, Uzume-feed
```

## What Needs to Happen Next

### Priority 1 — Complete Uzume-profiles (Phase 2 Agent 1)
The service has domain logic but no HTTP layer. Needs:
- `src/models/` — proper sqlx-compatible model files
- `src/queries/` — sqlx::query_as! macros for all profile/follow SQL
- `src/handlers/` — Axum handler functions
- `src/routes/` — Router with all profile + follow endpoints
- `src/workers/` — NATS subscriber stubs (profile_stub, search_sync)
- `src/main.rs` — NyxServer::builder() entry point
- `tests/api/` — integration tests (testcontainers)
- `tests/services/` — unit tests

### Priority 2 — Complete Brizo (full implementation)
Missing: `query.rs` (typed search), `sync.rs` (NATS → Meilisearch event-driven sync)

### Priority 3 — Complete Ogma (full implementation)
Missing: `messages.rs`, `aliases.rs`, `privacy.rs` (cross-app link checks via Heka)

### Priority 4 — Uzume-feed HTTP layer
Same pattern as profiles — domain logic exists, HTTP layer missing

### Priority 5 — Uzume-stories service (new)
Full new service, not yet started

### Priority 6 — Oya worker (full implementation)
Has lib structure + pipeline, needs real image/video processing

### Priority 7 — Ushas worker (stub → full)
Needs all: gorush, in_app, grouping, preferences, bin/worker

---

## Workspace Compile Status
✅ `cargo check` passes clean (all crates compile)
