# Heka: The Auth + Identity

> In Egyptian cosmology, magic (heka) worked through one mechanism: knowing the true name of a thing gave you power over it. Isis used this to extract Ra's secret name and gain dominion over him. Every binding spell, every protective charm, every act of divine power ran through Heka, the god who made names real.

This is the auth + identity service for Nyx. It provides a clean interface to Ory Kratos, and implements the app-scoped alias system that allows users to link their identities across apps without sharing personally identifiable information (PII).


Ory Kratos HTTP client (`KratosClient`). Session validation, identity CRUD, and the **app-scoped alias system** — the core of privacy isolation:

```rust
pub async fn validate_session(&self, token: &str) -> Result<NyxIdentity>;
pub async fn resolve_alias(&self, id: NyxId, app: NyxApp) -> Result<AppAlias>;
pub async fn identity_from_alias(&self, alias: &str, app: NyxApp) -> Result<NyxIdentity>;
pub async fn are_linked(&self, id_a: NyxId, id_b: NyxId, from: NyxApp, to: NyxApp) -> Result<bool>;
```

Aliases stored in `nyx.app_aliases` table. Kratos holds canonical identity (phone, email, password hash). Nyx layer adds per-app alias mapping.


Ory Kratos HTTP client. Depends on `common`.

```
Heka/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs          # KratosClient: wraps reqwest, typed responses
│   ├── session.rs         # Session validation (GET /sessions/whoami)
│   ├── identity.rs        # Identity CRUD (admin API)
│   ├── types.rs           # Kratos response types (Session, Identity, Traits)
│   └── alias.rs           # App-scoped alias management (create, resolve, validate uniqueness)
└── tests/
```

**API surface exposed to app services:**

```rust
/// Validate a Kratos session cookie/token, return the authenticated identity.
pub async fn validate_session(&self, session_token: &str) -> Result<NyxIdentity>;

/// Get or create an app-scoped alias for a given identity + app.
pub async fn resolve_alias(&self, identity_id: NyxId, app: NyxApp) -> Result<AppAlias>;

/// Lookup a Nyx identity from an app-scoped alias (reverse resolution).
pub async fn identity_from_alias(&self, alias: &str, app: NyxApp) -> Result<NyxIdentity>;

/// Check whether two identities have a cross-app link (user consented).
pub async fn are_linked(&self, id_a: NyxId, id_b: NyxId, from_app: NyxApp, to_app: NyxApp) -> Result<bool>;
```

The alias system is the core of the privacy isolation model. Aliases are stored in the `nyx` PostgreSQL schema, not in Kratos itself. Kratos holds the canonical identity (phone, email, password hash). The Nyx layer adds the per-app alias mapping.
