### platform/nyx-api

The shared Axum framework layer. Provides middleware, extractors, and response types that every app service uses. Depends on `Monad`, `Heka`, `nyx-cache`.

Shared Axum framework layer. The `NyxServer` builder is the entry point every service uses:

```rust
NyxServer::builder()
    .with_config(config)
    .with_db_pool(pool)
    .with_cache(cache)
    .with_events(nats)
    .with_routes(app_routes)
    .build()
    .serve()
    .await
```

Every service gets auth middleware, rate limiting, tracing, CORS, `/healthz`, OpenAPI docs, graceful shutdown — zero boilerplate.

Middleware: `auth.rs` (JWT extraction + validation), `rate_limit.rs` (token bucket via DragonflyDB), `request_id.rs`, `tracing.rs`, `app_context.rs`.

Extractors: `AuthUser` (validated identity from request), `ValidatedJson<T>` (deserialize + validate body), cursor pagination extractor.

Response: `ApiResponse<T>` envelope wrapping `{ data: T, pagination?: CursorResponse }`.


```
nyx-api/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── server.rs          # NyxServer builder: creates Axum router with all middleware pre-configured
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs        # JWT extraction + validation layer (calls Heka)
│   │   ├── rate_limit.rs  # Token bucket rate limiter (uses nyx-cache/DragonflyDB)
│   │   ├── request_id.rs  # X-Request-Id generation + propagation
│   │   ├── tracing.rs     # Request/response tracing span
│   │   └── app_context.rs # Injects NyxApp context from request path prefix
│   ├── extract/
│   │   ├── mod.rs
│   │   ├── auth.rs        # AuthUser extractor — extracts validated user identity from request
│   │   ├── cursor.rs      # Pagination extractor (parses CursorRequest from query params)
│   │   └── validated.rs   # ValidatedJson<T> — deserializes + validates request body
│   ├── response.rs        # ApiResponse<T> wrapper: { data: T, pagination?: ... }
│   └── openapi.rs         # OpenAPI doc builder, merges per-route schemas via utoipa
└── tests/
```

The `NyxServer` builder is the entry point every app service uses:

```rust
// In any app's main.rs:
NyxServer::builder()
    .with_config(config)
    .with_db_pool(pool)
    .with_cache(cache)
    .with_events(nats)
    .with_routes(Uzume_routes)     // App-specific routes
    .build()
    .serve()
    .await
```

This ensures every app service gets: structured logging, request tracing, auth middleware, rate limiting, CORS, health check endpoint (`/healthz`), OpenAPI docs (`/docs`), and graceful shutdown — with zero boilerplate in the app code.
