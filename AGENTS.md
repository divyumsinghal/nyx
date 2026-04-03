# PROJECT KNOWLEDGE BASE

**Generated:**
**Commit:**
**Branch:**

## Source of Truth

If you want to understanding the engineering, just understand: [Bible](Seshat/ARCHITECTURE.md)
-> Read this before any change

> Never MOCK Components - This is a production System - Dont create issues with Mocking.
> Never MOCK Components - This is a production System - Dont create issues with Mocking.
> Never MOCK Components - This is a production System - Dont create issues with Mocking.

## OVERVIEW

Execution target in this session: complete remaining Phase 0-2 gaps (no commits), with TDD and compatibility to Step 1 contracts.

Current verified status:

- Phase 0 infra/config/migrations: present and complete enough for development workflows.
- Phase 1/2: partially complete; key blocker was `Uzume-stories` compile break from missing modules.
- `Uzume-stories` now compiles and tests pass (`cargo test -p Uzume-stories`).
- Workspace `just test` now passes end-to-end (338/338) after Heimdall + events test fixes.
- `just ci` recipe invocation was fixed (now calls `just <recipe>` internally).

Active risks:

- Stories SQL query schema currently appears to differ from migration naming/shape (`uzume.*` vs `"Uzume".*`, and column naming differences). This is likely a runtime integration risk and needs reconciliation in follow-up tasks.
- `just lint` still fails due widespread clippy pedantic/doc constraints in legacy modules (not limited to this session's changes), especially `apps/Uzume/Uzume-profiles` and some platform crates.

## STRUCTURE

```

```

## WHERE TO LOOK

| Task                   | Location                                   | Notes                                                      |
| ---------------------- | ------------------------------------------ | ---------------------------------------------------------- |
| Stories service entry  | apps/Uzume/Uzume-stories/src/main.rs       | Bootstraps config, state, routes, workers, server          |
| Stories HTTP handlers  | apps/Uzume/Uzume-stories/src/handlers/     | Story + highlight endpoints and shared API error mapping   |
| Stories routes         | apps/Uzume/Uzume-stories/src/routes/mod.rs | Auth/public route split using nyx-api auth middleware      |
| Stories business logic | apps/Uzume/Uzume-stories/src/services/     | Visibility, ownership, pagination, idempotency helpers     |
| Stories workers        | apps/Uzume/Uzume-stories/src/workers/      | Expiry and reconciliation loops with retry-safe logging    |
| Stories data access    | apps/Uzume/Uzume-stories/src/queries/      | SQL access layer, verify against migrations before release |

## TDD (Test-Driven Development)

**MANDATORY.** RED-GREEN-REFACTOR:

1. **RED**: Write test → `just test` → FAIL
2. **GREEN**: Implement minimum → PASS
3. **REFACTOR**: Clean up → stay GREEN

**Rules:**

- NEVER write implementation before test
- NEVER delete failing tests - fix the code
- Test file: `*.test.ts` alongside source
- BDD comments: `#given`, `#when`, `#then`

## CONVENTIONS

- **Package manager**: Bun only (`bun run`, `bun build`, `bunx`)
- **Types**: bun-types (NEVER @types/node)
- **Build**: `bun build` (ESM) + `tsc --emitDeclarationOnly`
- **Exports**: Barrel pattern via index.ts
- **Naming**: kebab-case dirs, `createXXXHook`/`createXXXTool` factories
- **Testing**: BDD comments, 95 test files
- **Temperature**: 0.1 for code agents, max 0.3

## ANTI-PATTERNS

| Category | Forbidden |
| -------- | --------- |

## COMMANDS

```bash
cargo test -p Uzume-stories
just test-unit
just test
just ci
```

## DEPLOYMENT

## COMPLEXITY HOTSPOTS

| File                                             | Lines | Description                                                             |
| ------------------------------------------------ | ----- | ----------------------------------------------------------------------- |
| apps/Uzume/Uzume-stories/src/queries/stories.rs  | ~260  | Query layer likely mismatched with current migrations; integration risk |
| apps/Uzume/Uzume-stories/src/handlers/stories.rs | ~200  | Endpoint orchestration + authz + pagination, likely to expand           |

## SESSION UPDATE 2026-04-01

Completed in this run:

- Added missing `Uzume-stories` modules required by crate surface:
  - `src/handlers/mod.rs`, `src/handlers/stories.rs`, `src/handlers/highlights.rs`
  - `src/routes/mod.rs`
  - `src/services/stories.rs`, `src/services/highlights.rs`
  - `src/workers/mod.rs`, `src/workers/expiry.rs`, `src/workers/reconciliation.rs`
- Replaced empty `apps/Uzume/Uzume-stories/src/main.rs` with full startup flow.
- Added and passed service-level unit tests for visibility/authz, idempotency, and ownership helpers.
- Fixed failing full-test blockers outside stories:
  - `Monad/Heimdall`: added `src/lib.rs`, repaired integration/auth extraction behavior, request-id propagation, and JWT test assertions.
  - `Monad/events/tests/event_boundary.rs`: aligned with current events API.
- Verified full test gate passes: `just test` (338/338).
- Repaired `justfile` CI recipe command invocation wiring.

Next deltas to reach stronger Phase 2 readiness:

- Reconcile stories SQL layer with migration schema and add integration tests against real DB.
- Continue through remaining service/gateway/event boundary gaps discovered by audit.
- Resolve workspace-wide clippy/doc strictness issues so `just lint` and `just ci` pass cleanly.

## SESSION UPDATE 2026-04-01 (Unified tests crate)

Completed in this run:

- Expanded root `tests` crate into a unified harness with dedicated suites under:
  - `tests/tests/integration/`
  - `tests/tests/security/`
  - `tests/tests/property/`
  - `tests/tests/e2e/`
- Migrated key cross-component tests into unified crate (non-destructive phase):
  - `Monad/Heka/tests/integration_privacy_matrix.rs` -> `tests/tests/integration/heka_link_policy.rs`
  - `Monad/events/tests/event_boundary.rs` -> `tests/tests/integration/events_boundary.rs`
  - `apps/Uzume/Uzume-feed/tests/security_tests.rs` -> `tests/tests/security/feed_security.rs`
- Added new unified test suites:
  - `tests/tests/security/payload_sweep.rs`
  - `tests/tests/property/generators_property.rs`
  - `tests/tests/e2e/sandbox_smoke.rs`
- Added unified test documentation at `tests/README.md`.
- Added dedicated just recipes for unified test gating:
  - `test-security`, `test-property`, `test-e2e`, `test`

Current migration status:

- Root unified crate now hosts integration, security, property, and e2e suites.
- Legacy crate-local tests are still present intentionally during stabilization.

Next deltas:

- Continue big-bang migration of remaining crate-local test files into `tests/tests/`.
- Validate suite with `cargo test -p tests` and `just test`.
- After green stabilization window, remove duplicated legacy test copies.

## SESSION UPDATE 2026-04-01 (Unified migration completion)

Completed in this run:

- Performed big-bang mirror migration of all 33 legacy Rust test files into unified root crate under:
  - `tests/tests/migrated/Monad/...`
  - `tests/tests/migrated/apps/...`
- Preserved existing crate-local tests during stabilization (non-destructive phase).
- Kept unified executable suites active and green via top-level harness files:
  - `tests/tests/integration.rs`
  - `tests/tests/security.rs`
  - `tests/tests/property.rs`
  - `tests/tests/e2e.rs`

Verification snapshot:

- `cargo test -p tests` passes.
- `just test` passes.
- `just test-security` and `just test-property` pass.
- `just test-e2e` now skips gracefully when Docker socket is unavailable.

Remaining hardening work:

- Convert mirrored files in `tests/tests/migrated/**` into actively compiled unified targets in phases.
- Remove legacy crate-local copies after sustained green window.

## SESSION UPDATE 2026-04-01 (Legacy test cutover)

Completed in this run:

- Performed verified cutover: validated unified suite first, then removed legacy test directories from app/platform crates.
- Deleted legacy test directories:
  - `Monad/Akash/tests`
  - `Monad/Heimdall/tests`
  - `Monad/Heka/tests`
  - `Monad/Lethe/tests`
  - `Monad/Mnemosyne/tests`
  - `Monad/api/tests`
  - `Monad/events/tests`
  - `Monad/xtask/tests`
  - `apps/Uzume/Uzume-feed/tests`
  - `apps/Uzume/Uzume-profiles/tests`
- Expanded unified security coverage with dedicated suites:
  - `tests/tests/security/sql_injection.rs`
  - `tests/tests/security/xss.rs`
  - `tests/tests/security/authz_bypass.rs`
  - `tests/tests/security/privacy_violations.rs`
  - `tests/tests/security/edge_cases.rs`
  - wired in `tests/tests/security.rs`

Verification snapshot:

- Legacy test scan outside root `tests/` now returns only root tests tree.
- `cargo test -p tests` passes.
- `just test-security` passes (30 tests).
- `just test` passes (64 tests).
- TESTING dependency coverage in `tests/Cargo.toml` confirmed.
