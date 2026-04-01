# PROJECT KNOWLEDGE BASE

**Generated:**
**Commit:**
**Branch:**

## OVERVIEW

Execution target in this session: complete remaining Phase 0-2 gaps (no commits), with TDD and compatibility to Step 1 contracts.

Current verified status:
- Phase 0 infra/config/migrations: present and complete enough for development workflows.
- Phase 1/2: partially complete; key blocker was `Uzume-stories` compile break from missing modules.
- `Uzume-stories` now compiles and tests pass (`cargo test -p Uzume-stories`).

Active risks:
- Stories SQL query schema currently appears to differ from migration naming/shape (`uzume.*` vs `"Uzume".*`, and column naming differences). This is likely a runtime integration risk and needs reconciliation in follow-up tasks.


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

Next deltas to reach stronger Phase 2 readiness:
- Reconcile stories SQL layer with migration schema and add integration tests against real DB.
- Continue through remaining service/gateway/event boundary gaps discovered by audit.
