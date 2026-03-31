# Decisions

- Global Identity (IdentityId) is internal-only; Aliases are the primary public identifiers.
- Default-private isolation is enforced at the platform level.
- Cross-app linking requires explicit opt-in stored in Platform DB (nyx.app_links).
- Every request must resolve to an app_id for proper isolation.
- Restored just/.gitkeep to maintain intended repository structure.
- Event backbone provider selection is officially deferred to Step-2+.
- Added contract modules under Nun src/types (app, feed_mode, link_policy) and re-exported from lib.rs as the canonical shared contract surface.
- Chose tagged LinkPolicy enum with explicit variants (one_way, two_way, app_selective, revoked) plus LinkDirection to keep identity-link semantics unambiguous and extensible.
- For LinkPolicy::AppSelective, validation rejects duplicate app entries to enforce deterministic and unambiguous privacy scope per app.
- For LinkPolicy::AppSelective, validation rejects duplicate app entries to enforce deterministic and unambiguous privacy scope per app.
- For LinkPolicy::AppSelective, validation rejects duplicate app entries to enforce deterministic and unambiguous privacy scope per app.
- Replaced placeholder CI workflow with a single deterministic Rust gate job that installs rustfmt/clippy/deny/audit, attempts nextest best-effort, and runs security checks as required gates.
- Standardized justfile around CI-focused recipes (`fmt-check`, `lint`, `security`, `test`, `ci`) and added fallback logic in `test*` recipes to preserve execution when nextest is unavailable.
- Task 2 decision: enforce strict JSON contract on LinkPolicy by rejecting unknown fields during serde deserialization.

- Task 2 decision: LinkPolicy now enforces strict unknown-field rejection with serde deny_unknown_fields.

- Task 2 decision: implemented Default for LinkPolicy as Revoked to encode privacy-first explicit-linking semantics in the contract layer.

- Task 2 rerun decision: no contract code change was applied because NyxApp, FeedMode, LinkPolicy, exports, serde strictness, and validation semantics already match required outcome.
- Task 4 decision: CI now executes a single `just ci` parity gate instead of duplicating individual steps in workflow YAML, ensuring the exact same command graph is used locally and in CI with deterministic nextest fallback kept in `just test`.

- Task 3 decision: modeled `nyx.app_aliases` with app-scoped alias uniqueness (`UNIQUE (app, alias)`) and `nyx.app_links` with composite source/target FK references to `(nyx_identity_id, app)` in `nyx.app_aliases` to enforce explicit app-boundary link integrity at the database layer.
- Task 3 decision: constrained `Uzume.profiles` to `app = uzume` and added composite FK `(nyx_identity_id, app, alias) -> nyx.app_aliases` so Uzume-facing aliases cannot drift from platform alias records.

- Task 5 decision: introduced `Seshat/SECURITY-BASELINE.md` as the actionable security-first baseline artifact, including threat categories, controls, abuse-case expectations, and mandatory local/CI gate mapping.
- Task 5 decision: wired `security-secret-scan` and `gate-cross-app-unauthorized` as mandatory just recipes and included the unauthorized-access gate in `just ci` so it is blocking by default.
- Task 5 decision: updated `.github/workflows/ci.yml` to run `just security` and `just gate-cross-app-unauthorized` explicitly before `just ci` to make security gating visible and non-optional in CI execution logs.

- Task 6 decision: implemented Heka Kratos client core behind a provider trait (`KratosProvider`) with a reqwest-backed adapter (`ReqwestKratosProvider`) so unit tests can validate deterministic Nun error mapping without leaking raw provider payloads beyond adapter boundary.
- Task 6 decision: `KratosClient` exposes typed methods for step-1 core (`validate_session`, `get_identity`) and maps provider failures into Nun-standardized `NyxError` categories using stable machine-readable codes.
- Task 6 decision: implement Heka Kratos core through provider trait (`KratosProvider`) plus reqwest adapter (`ReqwestKratosProvider`) so deterministic mapping and session/identity logic are testable without exposing provider internals.
- Task 6 decision: keep service-facing identity contract minimal (`NyxIdentity { id }`) and enforce strict serde boundaries (`deny_unknown_fields`) on raw Kratos response structs.
- Task 6 hardening decision: keep `NyxIdentity` and app-facing contracts in `types.rs` only, while making Kratos boundary payload models private implementation details in `client.rs` to avoid accidental public coupling to provider schema.
- Task 6 hardening decision: preserve deterministic Nun mapping codes and statuses unchanged while tightening malformed payload handling via internal parse step (`serde_json::Value` -> private typed structs with `deny_unknown_fields`).
- Task 7 decision: link policy precedence is deterministic with revoked deny taking priority and default private fallback when no valid rule exists.

- Task 7 decision: introduce a dedicated in-memory `LinkPolicyEngine` in Heka to make policy evaluation explicit, privacy-first, and testable before storage integration.
- Task 7 decision: introduce in-memory LinkPolicyEngine in Heka for explicit privacy-first alias/link evaluation with deterministic precedence.
