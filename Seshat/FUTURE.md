# FUTURE

> Ignore this file for now.

## Prithvi/

```
Prithvi/
├── compose/
│   ├── infra.yml              # Shared infrastructure: PostgreSQL, DragonflyDB, NATS, MinIO,
│   │                          #   Meilisearch, Ory Kratos, Continuwuity, Gorush, Grafana stack
│   ├── apps.yml               # All Nyx app services: gateway, Uzume, Anteros, Themis,
│   │                          #   media-worker, notify-worker
│   ├── dev.yml                # Override: exposes debug ports, enables hot-reload, verbose logging
│   └── prod.yml               # Override: resource limits, restart policies, log drivers
│
├── docker/
│   ├── Dockerfile.service     # Multi-stage Dockerfile for any Rust service binary
│   ├── Dockerfile.worker      # Multi-stage Dockerfile for worker binaries (media, notify)
│   └── Dockerfile.web         # Dockerfile for Typescript frontend (node build → nginx static)
│
├── config/
│   ├── kratos/
│   │   ├── kratos.yml         # Ory Kratos server configuration
│   │   └── identity.schema.json  # Nyx identity schema (phone, email, display_name, avatar)
│   ├── continuwuity/
│   │   └── conduwuit.toml     # Continuwuity (Matrix homeserver) configuration
│   ├── gorush/
│   │   └── config.yml         # Gorush push notification config (APNs certs, FCM keys)
│   ├── meilisearch/
│   │   └── config.toml        # Meilisearch settings (master key, data dir)
│   ├── nats/
│   │   └── nats-server.conf   # NATS JetStream configuration (streams, retention)
│   ├── prometheus/
│   │   └── prometheus.yml     # Scrape targets: all services + infrastructure
│   ├── grafana/
│   │   ├── provisioning/      # Auto-provisioned datasources + dashboards
│   │   └── dashboards/        # Pre-built dashboards (per-service latency, error rate, etc.)
│   └── nginx/
│       └── nginx.conf         # Optional: TLS termination if not using Cloudflare
│
├── terraform/
│   ├── oracle/                # Oracle Cloud Always-Free infrastructure provisioning
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── compute.tf         # ARM VMs (4 cores, 24GB RAM)
│   │   ├── network.tf         # VCN, subnets, security lists
│   │   ├── storage.tf         # Block volumes (200GB)
│   │   └── outputs.tf
│   └── cloudflare/
│       ├── main.tf
│       ├── dns.tf             # DNS records for all services
│       └── r2.tf              # R2 bucket for CDN media serving
│
└── scripts/
    ├── deploy.sh              # Pull images, docker compose up, health check
    ├── backup.sh              # PostgreSQL pg_dump → Backblaze B2
    ├── restore.sh             # Restore from backup
    └── health-check.sh        # Verify all services are running and responding
```

### Prithvi/compose/infra.yml

This is the most important deployment file. It starts every piece of infrastructure the Nyx platform depends on. Any developer runs `docker compose -f Prithvi/compose/infra.yml up -d` and has the full platform infrastructure running locally.

Services defined:

| Service        | Image                                         | Ports      | Purpose                              |
| -------------- | --------------------------------------------- | ---------- | ------------------------------------ |
| postgres       | `postgres:17`                                 | 5432       | Primary database (all schemas)       |
| dragonfly      | `docker.dragonflydb.io/dragonflydb/dragonfly` | 6379       | Cache, sessions, rate limits         |
| nats           | `nats:2-alpine`                               | 4222, 8222 | Event bus (JetStream enabled)        |
| minio          | `minio/minio`                                 | 9000, 9001 | Object storage                       |
| meilisearch    | `getmeili/meilisearch`                        | 7700       | Full-text search                     |
| kratos         | `oryd/kratos`                                 | 4433, 4434 | Identity/auth (public + admin API)   |
| kratos-migrate | `oryd/kratos`                                 | —          | Runs Kratos DB migrations then exits |
| continuwuity   | `girlbossceo/conduwuit`                       | 6167       | Matrix homeserver                    |
| gorush         | `appleboy/gorush`                             | 8088       | Push notification dispatch           |
| prometheus     | `prom/prometheus`                             | 9090       | Metrics collection                   |
| loki           | `grafana/loki`                                | 3100       | Log aggregation                      |
| grafana        | `grafana/grafana`                             | 3200       | Dashboards                           |

### Prithvi/docker/Dockerfile.service

Shared multi-stage Dockerfile for all Rust service binaries. Build arg selects which binary to build.

```dockerfile
# Stage 1: Build
FROM rust:1.85-bookworm AS builder
ARG BIN_NAME
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY Monad/ Monad/
COPY apps/ apps/
RUN cargo build --release --bin ${BIN_NAME}

# Stage 2: Runtime (distroless — no shell, minimal attack surface)
FROM gcr.io/distroless/cc-debian12
ARG BIN_NAME
COPY --from=builder /app/target/release/${BIN_NAME} /usr/local/bin/service
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/service"]
```

Usage: `docker build --build-arg BIN_NAME=Uzume -f Prithvi/docker/Dockerfile.service .`

### Prithvi/config/kratos/identity.schema.json

The Nyx identity schema. Defines what a user identity looks like in Ory Kratos.

```json
{
  "$id": "https://nyx.dev/identity.schema.json",
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "NyxIdentity",
  "type": "object",
  "properties": {
    "traits": {
      "type": "object",
      "properties": {
        "email": {
          "type": "string",
          "format": "email",
          "title": "Email",
          "ory.sh/kratos": {
            "credentials": { "password": { "identifier": true } },
            "verification": { "via": "email" },
            "recovery": { "via": "email" }
          }
        },
        "phone": {
          "type": "string",
          "format": "tel",
          "title": "Phone number"
        },
        "display_name": {
          "type": "string",
          "title": "Display name"
        }
      },
      "required": ["email", "phone"],
      "additionalProperties": false
    }
  }
}
```

Phone and email are required for every Nyx account. Display name is optional (each app has its own profile with its own display name). Kratos handles password hashing (Argon2), email verification, phone verification (via webhook to your SMS provider), 2FA (TOTP + WebAuthn), and social login (Google, Apple, GitHub).

---

## migrations/

```
migrations/
├── Monad/
│   ├── 0001_create_schemas.sql           # CREATE SCHEMA nyx, Uzume, Anteros, Themis
│   ├── 0002_nyx_app_aliases.sql          # nyx.app_aliases: (nyx_identity_id, app, alias) UNIQUE
│   ├── 0003_nyx_app_links.sql            # nyx.app_links: cross-app consent records
│   └── 0004_nyx_push_tokens.sql          # nyx.push_tokens: device tokens for push notifications
│
├── Uzume/
│   ├── 0001_users.sql                    # Uzume.profiles
│   ├── 0002_posts.sql                    # Uzume.posts, Uzume.post_media
│   ├── 0003_interactions.sql             # Uzume.likes, Uzume.comments, Uzume.saves
│   ├── 0004_follow.sql                   # Uzume.follows
│   ├── 0005_stories.sql                  # Uzume.stories, Uzume.story_views, Uzume.highlights
│   ├── 0006_reels.sql                    # Uzume.reels, Uzume.reel_audio
│   ├── 0007_notifications.sql            # Uzume.notifications
│   └── 0008_feed.sql                     # Uzume.user_timeline (materialized feed)
│
├── Anteros/
│   ├── 0001_profiles.sql                 # Anteros.profiles
│   ├── 0002_swipes.sql                   # Anteros.swipes
│   ├── 0003_matches.sql                  # Anteros.matches
│   ├── 0004_fair_show.sql                # Anteros.fair_show_queue
│   └── 0005_preferences.sql              # Anteros.discovery_preferences
│
└── Themis/
    ├── 0001_listings.sql                 # Themis.listings (with PostGIS geography column)
    ├── 0002_reviews.sql                  # Themis.reviews
    ├── 0003_areas.sql                    # Themis.areas (neighborhoods, cities)
    └── 0004_inquiries.sql               # Themis.inquiries
```

Migrations are plain SQL files, run by `nyx-xtask migrate`. The `Mnemosyne` crate discovers migration files by scanning these directories and runs them in order using `sqlx::migrate!`. Each app's migrations are isolated to their own schema — they cannot accidentally affect another app's tables.

sqlx's compile-time query checking requires a running database. In CI, the `ci.yml` workflow starts PostgreSQL via `testcontainers`, runs all migrations, then cargo builds with query checking enabled.

---

## tools/

```
tools/
├── docker-build.sh            # Build all Docker images with correct tags
├── generate-openapi.sh        # Extract OpenAPI specs from running services
├── seed-data/
│   ├── users.json             # Sample Nyx users for development
│   ├── Uzume_posts.json        # Sample Uzume posts with media
│   ├── Anteros_profiles.json   # Sample Anteros profiles
│   └── Themis_listings.json   # Sample Themis listings
└── benchmark/
    ├── k6/                    # k6 load test scripts
    │   ├── Uzume-feed.js       # Load test the feed endpoint
    │   ├── Anteros-swipe.js    # Load test the swipe endpoint
    │   └── gateway-auth.js    # Load test JWT validation throughput
    └── README.md
```

**Tool choice — Load testing:**
- **k6 (chosen)**: Grafana's open-source load testing tool. Write tests in JavaScript, run from CLI. Integrates with Grafana dashboards for real-time metrics. Free and open-source.
- Alternative — wrk2: Simpler, C-based, generates constant-throughput load. Less flexible scripting.
- Alternative — Locust: Python-based, good UI. Heavier setup, slower throughput generation.

---

## Testing strategy

### Unit tests

Every `services/` module in every app and every platform library has `#[cfg(test)] mod tests` at the bottom of the file. These test pure business logic with no I/O. They use mock data, not real databases.

```rust
// In apps/Anteros/src/services/fair_show.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fair_show_inserts_within_window() {
        let queue = FairShowQueue::new(100); // window of 100
        queue.enqueue(profile_a, target_b);
        assert!(queue.must_show_within(target_b, profile_a) <= 100);
    }
}
```

### Integration tests

In each crate's `tests/` directory. These spin up real infrastructure (PostgreSQL, DragonflyDB, NATS, Meilisearch) using `testcontainers-rs`. They test the full HTTP request/response cycle.

```rust
// In apps/Uzume/tests/api/posts_test.rs
#[tokio::test]
async fn test_create_and_get_post() {
    let app = TestApp::spawn().await;          // Starts real DB, real Uzume server
    let user = app.create_test_user().await;

    let post = app.client()
        .post("/api/Uzume/posts")
        .bearer_auth(&user.token)
        .json(&json!({ "caption": "Hello world" }))
        .send()
        .await;

    assert_eq!(post.status(), 201);

    let body: ApiResponse<Post> = post.json().await;
    assert_eq!(body.data.caption, "Hello world");
}
```

### End-to-end tests

Frontend E2E tests in `clients/*/tests/` using Playwright. These run against the full stack (all services + frontend) in CI.

**Tool choice — E2E testing:**
- **Playwright (chosen)**: Multi-browser (Chromium, Firefox, WebKit), fast parallel execution, reliable auto-waiting. The modern standard for web E2E testing.
- Alternative — Cypress: Good DX, but single-browser at a time, slower, struggles with multi-tab/multi-origin scenarios.

---