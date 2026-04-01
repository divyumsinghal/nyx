# Unified Test Crate & Comprehensive TDD Strategy

## TL;DR
> **Summary**: Create workspace-wide unified test harness at `@tests/` with all 22 Seshat TESTING.md dependencies, migrate all 33 integration tests via big bang approach, establish comprehensive security sweep (privacy-focused, edge cases, user data safety), TDD enforcement (CI-gating with full TESTING.md validation), and E2E tests using real sandboxed infrastructure.
>
> **Deliverables**: Unified `@tests/` crate with all dependencies migrated, 33 tests migrated, security sweep suite, E2E sandbox infrastructure, CI TDD gates, cleanup of old tests, documentation.
>
> **Effort**: XL (complex, large scope)
> **Parallel**: YES - 6 waves with 5-8 tasks per wave
> **Critical Path**: Wave 1 (foundation) → Wave 2 (migration) → Wave 3 (security) → Wave 5 (CI) → Final Verification

## Context
### Original Request
User wants to unify testing across Nyx monorepo by creating a unified test crate at `@tests/` that adopts all 22 dependencies from Seshat/TESTING.md. Goals include: component tests, E2E tests, integration tests, cross-component tests, comprehensive security testing (privacy-focused, edge cases, user data safety), TDD enforcement, and migration of all 33 existing integration tests via big bang approach.

### Interview Summary
- **Location**: `@tests/` at workspace root (already created with contracts/ subdirectory)
- **Dependencies**: Full Seshat TESTING.md stack (22 crates: proptest, criterion, assert_cmd, predicates, insta, rstest, test-case, serial_test, mockall, loom, quickcheck, arbitrary, libfuzzer-sys, tracing, tracing-subscriber, anyhow, thiserror, tempfile, pretty_assertions)
- **Testing philosophy**: "Go hard" - comprehensive coverage with multiple strategies
- **Migration**: Big bang - all 33 tests at once (user: "You are smart, you can take care of it")
- **Security**: Full security sweep - privacy-focused, comprehensive edge case testing. Open source site dealing with user data (don't want to get sued).
- **Infrastructure**: Real infrastructure as much as possible - like not servers, sandbox. Start actual services in sandbox for E2E tests.
- **TDD enforcement**: CI-gating for PRs to main - enforce everything in TESTING.md, check edge cases. No nasty surprises, especially with user data.
- **Dev-dependency cycle**: Resolved by moving `integration_privacy_matrix.rs` to `@tests/integration/heka_link_policy.rs` (no Heka dev-dep on uzume_profiles)

### Metis Review (gaps addressed)
Metis identified critical risks (dev-dependency cycles, big bang migration, security scope ambiguity, E2E infrastructure overload, TDD enforcement ambiguity) which are all resolved through user decisions:
1. **Dev-dependency cycle**: Fixed by moving cross-component test to `@tests/integration/`
2. **Big bang**: User explicitly requested, accepting risk with coordinated wave orchestration
3. **Security scope**: Full sweep - privacy-focused, edge cases, user data safety
4. **E2E infrastructure**: Real sandbox services, no production servers
5. **TDD enforcement**: Full TESTING.md validation (edge cases, no surprises)

## Work Objectives
### Core Objective
Establish a unified test crate ecosystem at workspace level (`@tests/`) that consolidates all testing patterns, fixtures, mocks, property-based strategies, security testing, and E2E sandbox infrastructure. Migrate all existing tests and enforce TDD via CI gating.

### Deliverables
- `@tests/` crate with all 22 TESTING.md dependencies and workspace-level structure
- Migrated 33 integration tests from scattered locations into `@tests/` with organized hierarchy
- Comprehensive security test suite (privacy-focused, edge cases, user data safety)
- E2E sandbox infrastructure (real services: postgres, NATS, DragonflyDB, MinIO, Kratos, Matrix, Meilisearch)
- CI integration with TDD gates (fmt, clippy, tests, security, coverage)
- Documentation and onboarding guide
- Cleanup of old test files after verification

### Definition of Done (verifiable conditions with commands)
```bash
# 1. Unified test crate builds successfully
cargo build -p tests

# 2. All 33 tests migrated
find @tests/ -name "*_test.rs" | wc -l  # Returns 33+
find . -path @tests -prune -o -name "*_test.rs" -print | grep -v @tests | wc -l  # Returns 0

# 3. No dependency cycles
cargo tree --workspace --duplicate  # No duplicates/cycles

# 4. Security tests pass
cargo nextest run --workspace security

# 5. E2E tests with sandbox infrastructure pass
cargo nextest run --workspace e2e

# 6. CI TDD gates work
just ci  # All steps pass: fmt, clippy, test, security, etc.

# 7. Documentation exists
ls @tests/README.md  # Comprehensive testing guide
```

### Must Have
- All 22 TESTING.md dependencies integrated and functional
- All 33 integration tests migrated and passing
- Security sweep covering SQL injection, XSS, authz bypass, PII leaks, privacy violations, edge cases
- E2E tests with real sandbox infrastructure (no mocks for core services)
- CI gates enforcing full TESTING.md validation
- Zero dependency cycles
- Old tests kept during migration, deleted after 7-day green verification

### Must NOT Have (guardrails, AI slop patterns, scope boundaries)
- **No production code changes** (only test infrastructure)
- **No circular dependencies** (Monad crates depending on Apps)
- **No mocked infrastructure for E2E** (use real sandbox services)
- **No partial test migration** (all 33 tests moved together)
- **No TDD gaps** (all TESTING.md criteria enforced in CI)
- **No edge case gaps** (comprehensive sweep for privacy/security)
- **No AI slop** (vague acceptance criteria, "we'll see", manual steps)

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
>
> Test decision: TDD-first with CI gating; migration approach: big bang; infrastructure: real sandbox services; security: full sweep.
>
> QA policy: Every task has agent-executed scenarios with specific commands, selectors, and expected results.
>
> Evidence: Store evidence under `.sisyphus/evidence/task-{N}-{slug}.{ext}`

## Execution Strategy
### Parallel Execution Waves
> Target: 5-8 tasks per wave. <3 per wave (except final) = under-splitting.
> Extract shared dependencies as Wave-1 tasks for max parallelism.

**Wave 1: Foundation** (5 tasks)
- Bootstrap `@tests/` crate with Cargo.toml, workspace integration, all 22 dependencies
- Create test harness structure (lib.rs, modules: common, fixtures, mocks, generators, builders, asserts)
- Extract Nun::testing utilities and extend with property-based helpers, mock factories
- Fix dev-dependency cycle (move integration_privacy_matrix.rs, remove Heka dev-dep)
- Set up justfile recipes (test, test-security, test-e2e, test-property, test-mock, test-all)

**Wave 2: Big Bang Migration** (8 tasks, parallelizable by crate)
- Migrate Uzume-profiles tests (7 files)
- Migrate Uzume-feed tests (4 files)
- Migrate Heimdall tests (5 files)
- Migrate xtask tests (5 files)
- Migrate Heka tests (3 files, excluding integration_privacy_matrix.rs which moves to cross-component)
- Migrate Uzume-stories, Uzume-reels, Uzume-discover tests (3 files total)
- Migrate Monad library tests (Lethe, Akash, api, events, Mnemosyne - 5 files total)
- Move cross-component test to @tests/integration/heka_link_policy.rs

**Wave 3: Security Sweep** (6 tasks)
- Implement SQL injection tests (payloads, multiple entry points, verification of proper sanitization)
- Implement XSS tests (payloads, script tags, URL manipulation, HTML injection)
- Implement authz bypass tests (user A accessing user B's resources, privilege escalation, token manipulation)
- Implement privacy violation tests (PII exposure, alias leakage, identity mapping leaks)
- Implement edge case tests (boundary conditions, empty inputs, oversized inputs, concurrent access)
- Implement fuzzing harness (libfuzzer-sys integration for critical paths)

**Wave 4: E2E Sandbox Infrastructure** (5 tasks)
- Create sandbox orchestration module (SandboxManager with service lifecycle)
- Implement postgres sandbox (testcontainers with schema migrations)
- Implement NATS sandbox (testcontainers with JetStream)
- Implement DragonflyDB sandbox (testcontainers with cache isolation)
- Implement remaining service sandboxes (MinIO, Kratos, Matrix, Meilisearch - testcontainers)

**Wave 5: E2E Full Flow Tests** (4 tasks)
- Create user registration & authentication flow (signup → Kratos → alias creation → profile setup)
- Create content creation & interaction flow (create post → feed → like → comment → timeline update)
- Create cross-app flow (Uzume-profiles → Uzume-feed → Uzume-stories, alias-based linking)
- Create security regression flows (attempted SQLi during signup → blocked, XSS in bio → sanitized, authz bypass → 403)

**Wave 6: CI TDD Integration** (4 tasks)
- Update justfile with comprehensive test recipes (test-unit, test-integration, test-e2e, test-security, test-property, test-all)
- Create CI workflow steps (fmt, clippy, audit, deny, test-all, security, e2e, property)
- Add CI gate enforcement (fail fast on any step, artifact collection)
- Add test reporting (JUnit XML, coverage reports, security scan results)

**Wave 7: Documentation & Cleanup** (3 tasks)
- Write comprehensive testing guide (@tests/README.md)
- Write onboarding guide for contributors (how to add tests, run tests, debug)
- Remove old test files after 7-day verification period

### Dependency Matrix (full, all tasks)
- **Workspace dependencies added**: `tests` crate to `[workspace]` members in root `Cargo.toml`
- **Dev-dependencies** (22 crates): proptest, criterion, assert_cmd, predicates, insta, rstest, test-case, serial_test, mockall, loom, quickcheck, arbitrary, libfuzzer-sys, tracing, tracing-subscriber, anyhow, thiserror, tempfile, pretty_assertions
- **Removed**: Heka's dev-dependency on uzume_profiles (line 26 from Monad/Heka/Cargo.toml)
- **Testcontainers modules**: postgres, nats, dragonfly, minio, redis, kratos, matrix, meilisearch (already in workspace dev-deps)

### Agent Dispatch Summary (wave → task count → categories)
- Wave 1 → 5 tasks → quick (foundation setup)
- Wave 2 → 8 tasks → unspecified-high (migration, coordination)
- Wave 3 → 6 tasks → unspecified-high (security sweep)
- Wave 4 → 5 tasks → unspecified-high (E2E infrastructure)
- Wave 5 → 4 tasks → unspecified-high (full flow tests)
- Wave 6 → 4 tasks → quick (CI integration)
- Wave 7 → 3 tasks → writing (documentation)

## TODOs
> Implementation + Test = ONE task. Never separate.
> EVERY task MUST have: Agent Profile + Parallelization + QA Scenarios.

### Wave 1: Foundation (Tasks 1-5) — DETAILED ABOVE
