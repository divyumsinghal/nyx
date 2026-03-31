# Nun: The Common

> In Egyptian cosmology, Nun is the infinite, formless, primordial ocean that existed before creation and on which the world continues to float. When Ra created the world, he did so on a mound that emerged from Nun. The universe is not separate from Nun, it floats on him perpetually, and he has never ceased to exist. He is the permanent, invisible substrate.

Zero deps on any other nyx crate. Every other crate depends on this.

Key types: `NyxId` (UUIDv7 newtype), `NyxApp` enum (Uzume, Anteros, Themis, ...), `CursorRequest`/`CursorResponse` (cursor-based pagination), `ErrorResponse` (standardized `{ error, code, request_id }`), config loading (env + TOML), custom validators (phone, email).

The foundational crate. Zero dependencies on any other `nyx-*` crate. Every other crate in the workspace depends on this.

```
Nun/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── id.rs              # UUIDv7 generation, NyxId newtype
│   ├── time.rs            # Timestamp helpers, UTC enforcement
│   ├── error.rs           # NyxError enum, HTTP status mapping, error response body
│   ├── config.rs          # Config loading trait (env + TOML), service config structs
│   ├── pagination.rs      # Cursor-based pagination types (CursorRequest, CursorResponse)
│   ├── validation.rs      # Re-exports of validator, custom validators (phone, email)
│   ├── types/
│   │   ├── mod.rs
│   │   ├── app.rs         # NyxApp enum (Uzume, Anteros, Themis, ...), app-scoped alias types
│   │   ├── media.rs       # MediaType, MediaVariant, upload metadata
│   │   └── user.rs        # Shared user types (NyxIdentityId, AppAlias)
│   └── testing.rs         # Test utilities (random ID generators, test config)
└── tests/
```

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
