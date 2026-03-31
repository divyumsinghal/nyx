# Learnings
- Architectural tone in Seshat is authoritative and structured, focusing on "Monad as substrate."
- Nun's Id<T> pattern is the core mechanism for compile-time entity safety.
- Uzume services follow a strict separate-process, separate-db-schema pattern.
- Event provider neutrality is critical for architectural flexibility; avoided hard-coding NATS in ADR-003.
- Nun re-exported NyxApp from lib.rs without a backing src/app.rs; contracts are now normalized under src/types and re-exported from lib.rs.
- Nun re-exported NyxApp from lib.rs without a backing src/app.rs, so contracts were normalized under src/types and re-exported from lib.rs.
- FeedMode is modeled as a non-exhaustive enum with snake_case serde names and Chronological as Default to preserve Step-1 behavior while remaining future-ready.
- Link policy coverage needs both serde parse tests and semantic validation (app_selective requires at least one app).
- Link-policy privacy semantics are safer when app_selective.apps is unique; duplicate app entries can hide policy authoring mistakes and should be rejected during validation.
- Serde-hardening for tagged enums should use `deny_unknown_fields` so extra JSON keys cannot silently bypass strict contract expectations.
- Serde-hardening for tagged enums should use `deny_unknown_fields` so extra JSON keys cannot silently bypass strict contract expectations.
- Link-policy privacy semantics are safer when `app_selective.apps` is unique; duplicate app entries can hide policy authoring mistakes and should be rejected during validation.
- Link-policy privacy semantics are safer when `app_selective.apps` is unique; duplicate app entries can hide policy authoring mistakes and should be rejected during validation.
- Link-policy privacy semantics are safer when `app_selective.apps` is unique; duplicate app entries can hide policy authoring mistakes and should be rejected at validation time.
- Nun currently exposes `NyxApp` from `lib.rs` but had no backing `src/app.rs`; contracts were normalized under `src/types/*` and re-exported from `lib.rs`.
- `FeedMode` contract is best modeled as a non-exhaustive enum with `snake_case` serde names and `Chronological` as `Default` to preserve Step-1 semantics while staying future-ready.
- Link policy needs both syntactic serde tests and semantic validation (`app_selective.apps` non-empty) to cover parse + invalid value rejection.
- Nun currently exposes `NyxApp` from `lib.rs` but had no backing `src/app.rs`; contracts were normalized under `src/types/*` and re-exported from `lib.rs`.
- `FeedMode` contract is best modeled as a non-exhaustive enum with `snake_case` serde names and `Chronological` as `Default` to preserve Step-1 semantics while staying future-ready.
- Link policy needs both syntactic serde tests and semantic validation (`app_selective.apps` non-empty) to cover parse + invalid value rejection.
- Nun re-exported NyxApp from lib.rs without a backing src/app.rs; contracts are now normalized under src/types and re-exported from lib.rs.
- FeedMode is modeled as a non-exhaustive enum with snake_case serde names and Chronological as Default to preserve Step-1 behavior while remaining future-ready.
- Link policy coverage needs both serde parse tests and semantic validation (app_selective requires at least one app).
- tmp
- Link-policy privacy semantics are safer when app_selective.apps is unique; duplicate app entries can hide policy authoring mistakes and should be rejected during validation.
- CI/just gates should avoid hard dependency on cargo-nextest by probing availability and falling back to cargo test for deterministic execution across fresh environments.
- Placeholder migration/validation checks can be explicit no-op gates (clear TODO output) so CI wiring is stable while real checks are pending.
- Serde-hardening for tagged enums should use `deny_unknown_fields` so extra JSON keys cannot silently bypass strict contract expectations.
 - Serde-hardening for tagged enums should use `deny_unknown_fields` so extra JSON keys cannot silently bypass strict contract expectations.
 - Added during Task 2: enforce strict unknown-field rejection for LinkPolicy serde via deny_unknown_fields.

- Task 2 learning: strict serde contracts for tagged enums should reject unknown JSON fields to avoid silent schema drift.

- Task 2 learning (contract semantics): default LinkPolicy should be Revoked to preserve default-private identity isolation when no explicit link choice is present.

- Task 2 rerun learning: Nun contracts already satisfy required semantics for FeedMode default chronological and LinkPolicy default revoked with strict serde unknown-field rejection.
- Task 2 rerun learning: invalid or unknown LinkPolicy payloads are deterministically rejected via tagged enum parsing and app_selective semantic checks.

- Task 2 rerun learning: Nun contracts already satisfy required semantics for FeedMode default chronological and LinkPolicy default revoked with strict serde unknown-field rejection.
- Task 2 rerun learning: invalid or unknown LinkPolicy payloads are deterministically rejected via tagged enum parsing and app_selective semantic checks.

- Task 2 rerun learning: Nun contracts under `src/types/{app,feed_mode,link_policy}.rs` already satisfy required semantics (FeedMode default chronological, LinkPolicy default revoked, strict serde unknown-field rejection).
- Task 2 rerun learning: invalid/unknown LinkPolicy payloads are deterministically rejected through tagged enum deserialization + explicit app_selective semantic checks (non-empty, unique apps).

- Task 2 rerun learning: Nun contracts under `src/types/{app,feed_mode,link_policy}.rs` already satisfy required semantics (FeedMode default chronological, LinkPolicy default revoked, strict serde unknown-field rejection).
- Task 2 rerun learning: invalid/unknown LinkPolicy payloads are deterministically rejected through tagged enum deserialization + explicit app_selective semantic checks (non-empty, unique apps).

## Ory Kratos Implementation Learnings
*   **Serde Strictness**: Always use `#[serde(deny_unknown_fields)]` for Identity `traits` mapping to prevent silent data dropping when schemas change. Unknown enums must securely fail closed.
*   **Anti-Leak boundaries**: Raw Kratos types (`ory_kratos_client::models::Session`, `Identity`) must be explicitly mapped to domain types (`AppSession`) before being passed to application logic or API responses to avoid leaking PII and internal authentication metadata.
*   **Validation endpoints**: Use `FrontendApi::to_session` for standard session validation (handling both Cookies and `X-Session-Token`), reserving `IdentityApi::get_identity` (Admin API) strictly for backend-only lookup.

- Task 2 rerun learning: Nun contracts already satisfy required semantics for FeedMode default chronological and LinkPolicy default revoked with strict serde unknown-field rejection.
- Task 2 rerun learning: invalid or unknown LinkPolicy payloads are deterministically rejected via tagged enum parsing and app_selective semantic checks.
- Task 4 learning: replacing placeholder migration/validation recipes with concrete cargo gates (`cargo run -p nyx-xtask -- migrate`, `cargo check --workspace --all-targets --all-features`) makes local/CI checks explicit and reviewable.

- Task 3 learning: step-1 migration baseline can be structured as plain SQL up/down files under `migrations/Monad` and `migrations/Uzume`, with privacy-critical integrity encoded via DB constraints (`UNIQUE (app, alias)`, cross-app link checks, and composite FK ties from app links/profiles back to `nyx.app_aliases`).

- Task 5 learning: security baseline should map explicit threat categories to concrete deterministic gates (dependency policy, CVE audit, secret signatures, and cross-app unauthorized-access invariants) so local and CI enforcement stay aligned.
- Task 5 learning: cross-app unauthorized-access gating can be made deterministic by asserting critical migration constraints (`source_app <> target_app`, no self-link, revoked default policy) rather than relying on runtime integration setup.

- Task 6 learning: provider payload structs stay internal and are mapped immediately to domain types.

- Task 6 learning: keep provider payload structs internal to Heka and map immediately into domain identity types to enforce anti-leak boundaries.
- Task 6 learning: deterministic Kratos error mapping stays stable as 401 auth_session_invalid, 403 auth_session_forbidden, 404 auth_identity_not_found, 5xx auth_provider_unavailable, network auth_network_unreachable, decode auth_provider_invalid_response.

- Task 6 learning: Heka’s anti-leak boundary is safest when provider payload structs (`KratosSession`, `KratosIdentity`, `KratosIdentityTraits`) are internal-only and mapped immediately into domain-facing `NyxIdentity`, with strict serde `deny_unknown_fields` on boundary structs.
- Task 6 learning: deterministic Kratos error mapping is stable when 401->`auth_session_invalid` (Unauthorized), 403->`auth_session_forbidden` (Forbidden), 404->`auth_identity_not_found` (NotFound), 5xx->`auth_provider_unavailable` (ServiceUnavailable), network->`auth_network_unreachable` (ServiceUnavailable), and decode/malformed payloads->`auth_provider_invalid_response` (ServiceUnavailable).

- Task 6 learning: Heka’s anti-leak boundary is safest when provider payload structs (`KratosSession`, `KratosIdentity`, `KratosIdentityTraits`) are internal-only and mapped immediately into domain-facing `NyxIdentity`, with strict serde `deny_unknown_fields` on boundary structs.
- Task 6 learning: deterministic Kratos error mapping is stable when 401->`auth_session_invalid` (Unauthorized), 403->`auth_session_forbidden` (Forbidden), 404->`auth_identity_not_found` (NotFound), 5xx->`auth_provider_unavailable` (ServiceUnavailable), network->`auth_network_unreachable` (ServiceUnavailable), and decode/malformed payloads->`auth_provider_invalid_response` (ServiceUnavailable).
- Task 6 learning: provider payload structs stay internal to Heka and map immediately to domain identity types to preserve anti-leak boundaries.
- Task 6 learning: deterministic Kratos error mapping is fixed as 401 auth_session_invalid, 403 auth_session_forbidden, 404 auth_identity_not_found, 5xx auth_provider_unavailable, network auth_network_unreachable, decode auth_provider_invalid_response.
- Task 6 hardening learning: strict anti-leak boundaries are stronger when Kratos payload models are private to `client.rs` and `KratosProvider` returns raw JSON that is parsed internally with `deny_unknown_fields` into private boundary structs before mapping to `NyxIdentity`.
- Task 6 TDD learning: RED started by adding `validate_session_rejects_identity_with_unknown_fields` in `Monad/Heka/tests/kratos_client_core.rs` to force strict malformed-provider rejection, GREEN was achieved by internal JSON parsing (`parse_session`/`parse_identity`) plus private boundary structs in `client.rs`, and REFACTOR moved provider payload types out of `types.rs` to keep service API clean.

- Task 7 learning: privacy-safe alias/link policy evaluation is easiest to keep deterministic by evaluating direct tuple match first and reverse tuple fallback second, with default-deny when neither rule exists.
- Task 7 learning: revoke semantics are most reliable when modeled as explicit `LinkPolicy::Revoked` upsert on the same tuple so visibility reverts immediately without implicit state transitions.
- Task 7 learning: app-selective policy checks must validate the target app in the evaluated direction (`to_app` for direct, `from_app` for reverse) to avoid accidental cross-app leakage.


- Task 7 learning: privacy-safe alias/link policy evaluation is easiest to keep deterministic by evaluating direct tuple match first and reverse tuple fallback second, with default-deny when neither rule exists.
- Task 7 learning: revoke semantics are most reliable when modeled as explicit `LinkPolicy::Revoked` upsert on the same tuple so visibility reverts immediately without implicit state transitions.
- Task 7 learning: app-selective policy checks must validate the target app in the evaluated direction (`to_app` for direct, `from_app` for reverse) to avoid accidental cross-app leakage.

- Task 7 learning: deterministic policy evaluation is safer when direct tuple match is checked first, reverse tuple fallback second, and no-match falls back to deny.
- Task 7 learning: explicit `Revoked` as a stored policy state guarantees immediate privacy reversion without implicit transitions.
- Task 7 learning: app-selective direction checks must evaluate target app by direction (`to_app` direct, `from_app` reverse) to prevent cross-app leakage.

## Task 8 Learning: Uzume-profiles Step-1 Identity/Policy Integration Points

### Files Identified

**Monad/Heka (Identity + Auth):**
- `/home/sin/nyx/Monad/Heka/src/types.rs` — line 4-6: `NyxIdentity { id: IdentityId }` struct
- `/home/sin/nyx/Monad/Heka/src/client.rs` — line 68-83: `KratosClient::validate_session(session_token) -> Result<NyxIdentity>`
- `/home/sin/nyx/Monad/Heka/src/link_policy.rs` — line 76-84: `LinkPolicyEngine::is_visible(owner, viewer, from_app, to_app) -> bool`
- `/home/sin/nyx/Monad/Heka/src/lib.rs` — line 8-9: re-exports `KratosClient`, `NyxIdentity`, `AppAlias`

**Monad/Nun (Error Types + Policy Types):**
- `/home/sin/nyx/Monad/Nun/src/error.rs` — line 81-92: `NyxError::unauthorized(code, msg)` → 401; line 95-106: `NyxError::forbidden(code, msg)` → 403; line 506-526: `ErrorResponse` wire format
- `/home/sin/nyx/Monad/Nun/src/types/app.rs` — line 6-10: `NyxApp` enum (`Uzume`, `Anteros`, `Themis`) with `#[non_exhaustive]`
- `/home/sin/nyx/Monad/Nun/src/types/link_policy.rs` — line 14-24: `LinkPolicy` enum variants (`OneWay`, `TwoWay`, `AppSelective`, `Revoked`) with `#[serde(deny_unknown_fields)]`
- `/home/sin/nyx/Monad/Nun/src/validation.rs` — line 300-325: `link_policy(&LinkPolicy) -> Result<()>` validator
- `/home/sin/nyx/Monad/Nun/src/testing.rs` — line 27-29: `test_id::<T>()`, line 41-83: `test_config()`, line 102-114: `assert_error_kind()`, line 117-127: `assert_error_code()`

**apps/Uzume/Uzume-profiles:**
- `/home/sin/nyx/apps/Uzume/Uzume-profiles/src/lib.rs` — stub (line 1-3), needs step-1 implementation

### Call-Chain: Auth/Session Validation

```
HTTP Request (session token in Cookie or X-Session-Token)
    ↓
Handler extracts token
    ↓
heka::KratosClient::validate_session(&token)  [client.rs:68]
    ↓
KratosProvider::fetch_session() → /sessions/whoami
    ↓
map_kratos_identity() → NyxIdentity { id: IdentityId }  [client.rs:106]
    ↓
Result<NyxIdentity> returned to handler
```

### Call-Chain: Policy Enforcement (Post-Auth)

```
After NyxIdentity validated:
    ↓
Extract target identity + target NyxApp (e.g., Uzume)
    ↓
heka::LinkPolicyEngine::is_visible(owner, viewer, from_app, to_app)  [link_policy.rs:76]
    ↓
evaluate() → direct match first, reverse fallback, default-deny  [link_policy.rs:87-99]
    ↓
Returns bool (true=visible, false=hidden)
    ↓
If false → NyxError::forbidden("policy_denied", "Cross-app access denied")
```

### Error Response Matrix

| Scenario | HTTP Status | Code | Message |
|----------|-------------|------|---------|
| Missing/empty session token | 400 | `auth_session_token_missing` | "Session token is required" |
| Invalid/expired Kratos session | 401 | `auth_session_invalid` | "Session is invalid or expired" |
| Session lacks privileges | 403 | `auth_session_forbidden` | "Session does not have required privileges" |
| Identity not found (Kratos) | 404 | `auth_identity_not_found` | "Identity was not found" |
| Cross-app policy denies access | 403 | `policy_denied` | "Cross-app access denied" (app code) |
| Kratos 5xx response | 503 | `auth_provider_unavailable` | "Authentication provider is unavailable" |
| Network unreachable | 503 | `auth_network_unreachable` | "Authentication provider is unreachable" |

### Minimal Integration Surface for Uzume-profiles Step-1

1. **Dependency**: Add `Heka` to `Cargo.toml`:
   ```toml
   Heka = { path = "../../Monad/Heka" }
   ```

2. **Handler pattern** (pseudo-code):
   ```rust
   async fn get_profile(ctx: Context, alias: String) -> Result<Json<Profile>> {
       // 1. Validate session → get NyxIdentity
       let identity = ctx.heka.validate_session(ctx.session_token()).await?;
       
       // 2. Resolve alias → get AppAlias + IdentityId
       let app_alias = ctx.heka.resolve_alias(identity.id, NyxApp::Uzume).await?;
       
       // 3. Query profile by alias
       let profile = query_profile_by_alias(&alias, NyxApp::Uzume)?;
       
       // 4. Optional: Policy check if viewing other's profile
       if profile.owner_id != identity.id {
           let visible = ctx.link_policy.is_visible(
               profile.owner_id, 
               identity.id, 
               NyxApp::Uzume, 
               NyxApp::Uzume
           );
           if !visible {
               return Err(NyxError::forbidden("policy_denied", "Cross-app access denied"));
           }
       }
       
       Ok(Json(profile))
   }
   ```

3. **Error handling**: Use `NyxError::unauthorized()` / `NyxError::forbidden()` for auth failures. The wire format is standardized via `ErrorResponse` (line 512-526 in error.rs).

### Test Fixtures Available

- `nun::testing::test_id::<Post>()` — generate typed IDs
- `nun::testing::test_config()` — localhost config for tests
- `nun::testing::assert_error_kind(&result, ErrorKind::Unauthorized)` — assert 401
- `nun::testing::assert_error_code(&result, "auth_session_invalid")` — assert specific code

### What NOT to Do

- Do NOT expose `NyxIdentity` directly to API responses (anti-leak boundary)
- Do NOT call Kratos Admin API from handlers (reserved for backend-only lookups)
- Do NOT default-allow on missing policy — Task 7 established default-deny semantics
- Do NOT implement link policy upserts in step-1 (beyond read-only visibility checks)

## Uzume-profiles Step-1 Service: Rust Web-Service Patterns (Axum)

### 1. Standardized Error Handling via `IntoResponse`
**Reference**: [Axum Official Docs - Error Handling](https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html) / Reputable implementations like `tokio-rs/axum` examples.
**Pattern**: Wrap `anyhow::Error` or Heka-provided domain errors in an `AppError` tuple struct to ensure deterministic responses.
```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

pub struct AppError(pub StatusCode, pub String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = (self.0, self.1);
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
```
**Risk Note**: Returning raw strings or default 500s leaks internal state. Ensure all provider failures from Heka strictly map to this structure, converting non-auth domain issues explicitly to avoid non-deterministic auth errors.

### 2. App-Scoped Identity via `FromRequestParts` (Extractor)
**Reference**: [Axum Extractors - `FromRequestParts`](https://docs.rs/axum/latest/axum/extract/trait.FromRequestParts.html)
**Pattern**: Implement `FromRequestParts` for an app-scoped `User` or `ProfileIdentity` struct to enforce default-deny authentication on `GET /me` and `PATCH /me`.
```rust
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

pub struct ProfileIdentity {
    pub user_id: String,
    // Note: Do not expose global auth state, only app-scoped identifiers.
}

#[async_trait]
impl<S> FromRequestParts<S> for ProfileIdentity
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract token, validate via Heka/Nun foundation.
        // Return explicit 401 Unauthorized if invalid.
        Err(AppError(StatusCode::UNAUTHORIZED, "Invalid or missing auth token".into()))
    }
}
```
**Risk Note**: Do not leak global session IDs into the controller scope. The extractor must downcast the global token into a strictly scoped `ProfileIdentity` (Step-1 constraint: no global identity exposure). 

### 3. Handler Signatures for `/me` Endpoints
**Reference**: Standard REST patterns in Rust `axum` architectures (e.g., [GitHub OpenAPITools/openapi-generator Axum templates](https://github.com/OpenAPITools/openapi-generator/tree/master/samples/server/petstore/rust-axum)).
**Pattern**: Rely on the extractor to guarantee the user is authorized before the handler logic executes.
```rust
use axum::{Json, extract::State};

pub async fn get_me(
    identity: ProfileIdentity, // Extractor guarantees 401/403 if missing/invalid
    State(state): State<AppState>,
) -> Result<Json<ProfileDto>, AppError> {
    // Controller logic only sees the authorized app-scoped identity
    todo!()
}

pub async fn patch_me(
    identity: ProfileIdentity,
    State(state): State<AppState>,
    Json(payload): Json<UpdateProfileDto>,
) -> Result<Json<ProfileDto>, AppError> {
    todo!()
}
```
**Risk Note**: Avoid manual header parsing inside the handlers (`get_me`/`patch_me`). Doing so bypasses the centralized, deterministic error mapping of the extractor, potentially leading to inconsistent 403 vs 401 behaviors.

### 4. Public Profile Read (Explicit Scoping)
**Reference**: General Rust API security best practices.
**Pattern**: Differentiate explicitly between authenticated (`/me`) and public (`/:id`) endpoints by omitting the `ProfileIdentity` extractor for public reads, relying instead on data-layer authorization (e.g. `is_public` flags).
```rust
use axum::extract::Path;

pub async fn get_public_profile(
    Path(profile_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<PublicProfileDto>, AppError> {
    // No identity extractor -> purely public read logic.
    // Must return 404 NOT FOUND (not 403) if profile doesn't exist or is private, to prevent enumeration.
    todo!()
}
```
**Risk Note**: Returning 403 for private profiles on public endpoints creates an enumeration vulnerability (identity leakage). Always map access-denied on public routes to `404 Not Found`.
- Task 8 learning: keep Uzume-profiles response boundary identity-safe by storing `IdentityId` internally in the domain model while returning only alias/profile fields via a dedicated `PublicProfileResponse` DTO.
- Task 8 learning: protected `/me` behavior is deterministic when auth extraction is centralized (`require_identity`) and maps missing/blank token to `NyxError::unauthorized("auth_session_token_missing", ...)` before provider validation.
- Task 8 learning: private profile reads should default-deny and return `NyxError::forbidden("policy_denied", ...)` unless viewer is owner or `LinkPolicyEngine::is_visible(owner, viewer, NyxApp::Uzume, NyxApp::Uzume)` allows visibility.
- Task 8 learning: BDD test comments (`#given #when #then`) remain required and useful for enforcing TDD traceability in profile lifecycle and unauthorized/forbidden path tests.
- Task 8 endpoint wiring learning: a minimal callable endpoint layer can stay deterministic by routing `(method, path)` to service methods and mapping `NyxError` via `to_error_response(None)` into stable `{error, code}` JSON with matching status.
- Task 8 endpoint testing learning: endpoint-level lifecycle/auth/privacy coverage is preserved by BDD tests that assert concrete route behavior for `GET /me`, `PATCH /me`, and `GET /{alias}` including `auth_session_token_missing` and `policy_denied` codes.
