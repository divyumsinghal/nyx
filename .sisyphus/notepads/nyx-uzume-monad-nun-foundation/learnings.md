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
