# Heimdall: The Gateway

> Heimdall stands at the Bifrost, the only path between worlds, and decides what passes. He can see 100 miles in any direction, hear grass and wool growing, and needs almost no sleep. He holds the Gjallarhorn: if he blows it, every being in the nine worlds hears it.

The API gateway. A **binary crate** — the only externally-facing HTTP process. All client requests hit the gateway first.

Depends on `nyx-api`, `Heka`, `nyx-cache`, `Monad`.

```
Heimdall/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs          # Gateway config: upstream service URLs, rate limit settings
│   ├── proxy.rs           # Reverse proxy logic: match path prefix → forward to upstream
│   ├── websocket.rs       # WebSocket upgrade: authenticate → forward to app or Matrix
│   ├── health.rs          # /healthz endpoint, upstream health aggregation
│   └── routes.rs          # Route table: /api/Uzume/* → Uzume, /api/Anteros/* → Anteros, etc.
└── tests/
```

**Routing logic:**

```
Client request                       Gateway action
─────────────────────────────────    ─────────────────────────────────
POST /api/nyx/auth/login         →   Forward to Ory Kratos flow endpoint
GET  /api/nyx/account/me         →   Forward to Kratos + enrich with alias data
*    /api/Uzume/*                 →   Validate JWT → Forward to Uzume service
*    /api/Anteros/*               →   Validate JWT → Forward to Anteros service
*    /api/Themis/*               →   Validate JWT → Forward to Themis service
WS   /api/Uzume/ws                →   Validate JWT → WebSocket upgrade → Uzume
WS   /api/Anteros/ws              →   Validate JWT → WebSocket upgrade → Anteros
GET  /api/nyx/messaging/*        →   Validate JWT → Forward to Continuwuity (Matrix CS API)
GET  /healthz                    →   Health check (self + upstreams)
GET  /docs                       →   Aggregated OpenAPI docs
```

The gateway is a thin proxy. It does NOT contain business logic. It does:
1. TLS termination (or defers to Cloudflare/reverse proxy)
2. JWT validation via `Heka`
3. Per-IP and per-user rate limiting via `nyx-cache`
4. Request ID injection
5. CORS
6. Request forwarding to the correct app service

**Tool choice — Reverse proxy approach:**
- **Axum + hyper (chosen)**: The gateway is a Rust Axum app that forwards requests using `hyper::Client`. Full control over middleware, auth injection, and WebSocket handling.
- Alternative — Pingora (Cloudflare's Rust proxy): Purpose-built for proxying, extremely fast. But heavier dependency and less flexibility for custom auth logic.
- Alternative — Nginx/Caddy in front + thin Rust auth service: Simpler to operate, but splits the auth concern across two processes.

Thin reverse proxy. Does NOT contain business logic.

```
/api/nyx/auth/*          →  Ory Kratos
/api/nyx/account/*       →  Kratos + alias enrichment
/api/nyx/messaging/*     →  Continuwuity (Matrix CS API)
/api/Uzume/profiles/*     →  Uzume-profiles :3001
/api/Uzume/feed/*         →  Uzume-feed :3002
/api/Uzume/stories/*      →  Uzume-stories :3003
/api/Uzume/reels/*        →  Uzume-reels :3004
/api/Uzume/discover/*     →  Uzume-discover :3005
WS  /api/Uzume/ws         →  Uzume-feed (notifications)
GET /healthz             →  self + upstream health
GET /docs                →  aggregated OpenAPI
```
