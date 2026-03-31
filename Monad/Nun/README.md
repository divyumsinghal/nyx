# Nun: The Common

> In Egyptian cosmology, Nun is the infinite, formless, primordial ocean that existed before creation and on which the world continues to float. When Ra created the world, he did so on a mound that emerged from Nun. The universe is not separate from Nun, it floats on him perpetually, and he has never ceased to exist. He is the permanent, invisible substrate.

## Intro

Zero deps on any other nyx crate. Every other crate depends on this.

Key types: `NyxId` (UUIDv7 newtype), `NyxApp` enum (Uzume, Anteros, Themis, ...), `CursorRequest`/`CursorResponse` (cursor-based pagination), `ErrorResponse` (standardized `{ error, code, request_id }`), config loading (env + TOML), custom validators (phone, email).

The foundational crate. Zero dependencies on any other `nyx-*` crate. Every other crate in the workspace depends on this.

## What Nun contains

```
Nun/
├── src/
│   ├── lib.rs             # Re-exports everything, feature-gated modules
│   ├── id.rs              # Id<T>, platform entity markers, IdentityId -> UUIDv7 generation, NyxId newtype
│   ├── time.rs            # Timestamp alias, now(), ttl constants -> Timestamp helpers, UTC enforcement
│   ├── error.rs           # NyxError, ErrorKind, ErrorResponse, FieldError, Result, HTTP status mapping, error response body
│   ├── config.rs          # NyxConfig, all infra config structs, loading -> Config loading trait (env + TOML), service config structs
│   ├── pagination.rs      # Cursor, CursorValue, PageRequest, PageResponse -> Cursor-based pagination types (CursorRequest, CursorResponse)
│   ├── validation.rs      # Custom validators (phone, alias, display_name, etc.) -> Re-exports of validator, custom validators (phone, email)
│   └──  types/
│       ├── mod.rs
│       ├── app.rs         # NyxApp enum (Uzume, Anteros, Themis, ...), app-scoped alias types
│       ├── media.rs       # MediaType, MediaVariant, upload metadata
│       └── user.rs        # Shared user types (NyxIdentityId, AppAlias)
├── tests/                 # Comrehensive testing will all edge cases and features covered.
├── Cargo.toml             # Cargo manifest for Nun
├── AGENTS.md              # File for the Coding Agents to write as needed.
└── README.md              # This file
```

## Key Types

Key types defined here:

```rust
/// Every entity ID across the platform. UUIDv7 for time-sortability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct NyxId(Uuid);

/// Identifies which app context a request/entity belongs to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NyxApp {
    Uzume,
    Anteros,
    Themis,
}

/// Cursor-based pagination. Every list endpoint uses this.
#[derive(Debug, Deserialize)]
pub struct CursorRequest {
    pub cursor: Option<String>,    // Base64-encoded (created_at, id)
    pub limit: Option<u16>,        // Default 20, max 100
}

/// Standardized error response body. Every error from every service looks like this.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,              // Machine-readable: "post_not_found", "rate_limited"
    pub request_id: String,
}
```

---

## What Nun explicitly does NOT contain

- **No Axum, no HTTP framework types.** Nun is framework-agnostic.
- **No database connection management.** That's nyx-db - Mnemosyne.
- **No cache client.** That's nyx-cache (Lethe).
- **No NATS/event types.** That's nyx-events.
- **No business logic.** Not even shared business logic.
- **No middleware.** That's nyx-api.
- **No IntoResponse impls.** That's nyx-api.
- **No HTTP client.** That's reqwest, used by nyx-auth and others.
- **No media types beyond basic enums.** Processing logic is nyx-media (Oya).

Nun is types, errors, config, and utilities. Nothing more. It should compile in under 5 seconds with zero features enabled.

---