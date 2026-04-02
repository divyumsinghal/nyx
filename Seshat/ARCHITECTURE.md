# Nyx — Monorepo Architecture

> The definitive structure for the Nyx platform. This document is the source of truth for every directory, every crate, every tool choice, and every API contract. It is designed for 3 apps today and 30 tomorrow without structural changes.

**Nyx = Monad services + library crates.** They are the building blocks.
**Uzume = five domain microservices** that compose those building blocks and add Uzume-specific logic.

The repository is dual-workspace: a **Cargo workspace** governs all Rust crates (platform libraries + app services), and a **pnpm workspace** governs all frontend packages. They coexist at the same root. Deployments are Docker-based; each deployable unit has its own Dockerfile.

---

## The split: what is Nyx, what is Uzume


| Concern                                | Answer                                                | Lives in                    |
| -------------------------------------- | ----------------------------------------------------- | --------------------------- |
| User registration, login, 2FA          | Yes  — every app needs auth                           | `Monad/Heka`                |
| API gateway, routing, rate limits      | Yes                                                   | `Monad/Heimdall`            |
| Photo/video upload and processing      | Yes                                                   | `Monad/Oya`                 |
| Direct messaging (1:1, group)          | Yes                                                   | `Monad/Ogma`                |
| Push + in-app notifications            | Yes                                                   | `Monad/Ushas`               |
| Full-text search, geo search           | Yes                                                   | `Monad/Brizo`               |
| App-scoped aliases + privacy isolation | Yes  — core to the multi-app model                    | `Monad/Heka`                |
| Instagram-style home feed              | No   — Other apps have a different feed               | `apps/Uzume/Uzume-feed`     |
| 24h ephemeral stories                  | No   — unique to Uzume                                | `apps/Uzume/Uzume-stories`  |
| Short-form video reels                 | No   — unique to Uzume                                | `apps/Uzume/Uzume-reels`    |
| Explore/discover page                  | Semi — every app has discovery, but algorithms differ | `apps/Uzume/Uzume-discover` |
| Social profiles + follow graph         | Semi — follows are Uzume specific                     | `apps/Uzume/Uzume-profiles` |

---

## Adding a new app (Anteros example)

1. `apps/Anteros/Anteros-profiles/` — dating profiles (different fields than Uzume)
2. `apps/Anteros/Anteros-matching/` — swipes, fair-show queue, match → Matrix room
3. `apps/Anteros/Anteros-discover/` — discovery feed, preferences
4. `migrations/Anteros/` — own schema
5. Add `NyxApp::Anteros` to enum
6. Add NATS subjects, gateway routes, compose file

**Zero changes** to platform crates. **Zero changes** to Uzume. The platform does the rest.

---

## Runtime (full Uzume deployment)

**8 Nyx + Uzume processes:**
| Process        | Port | Role                               |
| -------------- | ---- | ---------------------------------- |
| Heimdall       | 3000 | Reverse proxy, auth, rate limiting |
| Uzume-profiles | 3001 | Profiles, follow graph             |
| Uzume-feed     | 3002 | Posts, timeline, comments          |
| Uzume-stories  | 3003 | Stories, highlights                |
| Uzume-reels    | 3004 | Reels, audio, video feed           |
| Uzume-discover | 3005 | Explore, search, trending          |
| Oya worker     | —    | Media processing (NATS)            |
| Ushas worker   | —    | Notification dispatch (NATS)       |

**11 infrastructure processes:**
PostgreSQL, DragonflyDB, NATS, MinIO, Meilisearch, Ory Kratos, Continuwuity, Gorush, Prometheus, Loki, Grafana.

**Total: 19 processes.**

---

## Dependency graph

```
                         ┌─────────────┐
                         │ Nun  │
                         └──────┬──────┘
          ┌──────┬──────┬──────┼──────┬──────┬──────┐
          ▼      ▼      ▼      ▼      ▼      ▼      ▼
       Mnemosyne  nyx-   nyx-   nyx-   nyx-   nyx-   nyx-
               cache  events storage search auth  messaging
          └──────┴──────┴──────┴─────────────┴──────┘
                         │
                         ▼
                    ┌─────────┐
                    │ nyx-api │
                    └────┬────┘
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
     Oya     Ushas     Heimdall     ← Monad binaries
          │
          ├──────┬──────┬──────┬──────┐
          ▼      ▼      ▼      ▼      ▼
       Uzume-  Uzume-  Uzume-  Uzume-  Uzume-           ← app microservices
       profiles feed  stories reels discover
```

No cycles. Clean layers. Monad does the heavy lifting. Apps compose and specialize.

---

## Architectural Decision Records (ADRs)

- [ADR-001: Identity Visibility and Linking](./ADR-001-identity-visibility-and-linking.md)
- [ADR-002: App Isolation Invariants](./ADR-002-app-isolation-invariants.md)
- [ADR-003: Platform App Boundaries](./ADR-003-platform-app-boundaries.md)

---

## Tool Candidates: Still evaluating, some options, research others

**Tool choice — HTTP framework:**
- **Axum (chosen)**: Tower middleware ecosystem, async, maintained by the Tokio team. Composable extractors, first-class WebSocket support. The Rust community consensus choice for new projects.
- Alternative — Actix-Web: Higher raw throughput in benchmarks, but its own middleware system doesn't interop with tower. Ecosystem lock-in.
- Alternative — Poem: Simpler API surface, good for small projects. Smaller ecosystem and community.

**Tool choice — Database client:**
- **sqlx (chosen)**: Compile-time checked SQL queries against a real database. Zero-cost abstraction, no ORM overhead, full PostgreSQL feature support. Queries are raw SQL — no DSL to learn, no ORM surprises.
- Alternative — SeaORM: Full ORM built on sqlx. More ergonomic for simple CRUD, but hides SQL, makes complex queries harder, and adds a layer of abstraction that breaks down at scale.
- Alternative — Diesel: Mature, well-tested, synchronous. Requires a separate CLI for migrations and schema generation. Does not support async natively (needs `tokio::task::spawn_blocking` wrappers).

**Tool choice — Error handling:**
- **thiserror + anyhow (chosen)**: `thiserror` for typed errors in library crates (Monad/), `anyhow` for ergonomic errors in binary crates (apps/). This is the Rust community standard pairing.
- Alternative — eyre + color-eyre: Better error reports with `SpanTrace` integration. Slightly more opinionated. Good for CLIs, overkill for server crates.

**Tool choice — Frontend package manager:**
- **pnpm (chosen)**: Strict dependency resolution (no phantom deps), content-addressable storage (saves disk), fast installs. The modern standard for monorepos.
- Alternative — npm: Universal but slower, flat node_modules causes phantom dependency issues.
- Alternative — bun: Fastest, but runtime compatibility gaps and less mature lockfile format.

**Tool choice — Frontend framework:**
- **Typescript (chosen)**: Smallest bundle sizes (compiler, not runtime), built-in SSR, simpler state management (Svelte stores vs React hooks/context), faster dev iteration. Free hosting on Cloudflare Pages. Kit provides file-based routing, form actions, and server-side data loading.
- Alternative — Next.js (React): Largest ecosystem, most third-party components. Heavier bundle, more complex state management, requires more configuration for optimal performance.
- Alternative — Nuxt (Vue): Good middle ground. Vue's reactivity model is intuitive. Smaller ecosystem than React.

**Tool choice — Matrix client SDK:**
- **matrix-js-sdk (chosen)**: Official Matrix Foundation SDK. Full E2EE support via libolm/vodozemac. Battle-tested by Element (the largest Matrix client). Heavy-ish bundle but it handles crypto, sync, and all Matrix edge cases.
- Alternative — Hydrogen SDK: Lightweight Matrix SDK from Element, designed for embedding. Less complete but smaller bundle.

---