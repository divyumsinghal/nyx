# Unified Test Crate & Comprehensive TDD Strategy

## TL;DR
> **Summary**: Create workspace-wide unified test harness at `@tests/` with all 22 Seshat TESTING.md dependencies, migrate all 33 integration tests via big bang approach, establish comprehensive security sweep (privacy-focused, edge cases, user data safety), TDD enforcement (CI-gating with full TESTING.md validation), and E2E tests using real sandboxed infrastructure.
>
> **Deliverables**: Unified `@tests/` crate with all dependencies migrated, 33 tests migrated, security sweep suite, E2E sandbox infrastructure, CI TDD gates, cleanup of old tests, documentation.
>
> **Effort**: XL (complex, large scope)
> **Parallel**: YES - 7 waves with 5-8 tasks per wave
> **Critical Path**: Wave 1 (foundation) → Wave 2 (migration) → Wave 3 (security) → Wave 4 (E2E infra) → Wave 5 (E2E flows) → Wave 6 (CI) → Final Verification

## Context
### Original Request
Unify testing across Nyx monorepo by creating a unified test crate at `@tests/` that adopts all 22 dependencies from Seshat/TESTING.md. Big bang migration of all 33 integration tests. Full security sweep (privacy-focused, edge cases, user data safety). TDD enforcement via CI gating. E2E tests with real sandboxed infrastructure (not production servers). "Go hard" - comprehensive coverage with multiple strategies. No nasty surprises with user data.

### Interview Summary
- **Location**: `@tests/` at workspace root (already created with contracts/ subdirectory)
- **Dependencies**: Full Seshat TESTING.md stack (22 crates: proptest, criterion, assert_cmd, predicates, insta, rstest, test-case, serial_test, mockall, loom, quickcheck, arbitrary, libfuzzer-sys, tracing, tracing-subscriber, anyhow, thiserror, tempfile, pretty_assertions)
- **Testing philosophy**: "Go hard" - comprehensive coverage with multiple strategies
- **Migration**: Big bang - all 33 tests at once (user: "You are smart, you can take care of it")
- **Security**: Full security sweep - privacy-focused, comprehensive edge case testing. Open source site dealing with user data (don't want to get sued).
- **Infrastructure**: Real infrastructure as much as possible - like not servers, sandbox. Start actual services in sandbox for E2E tests. Tests that are focused vs full flows, but everything in test environment.
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
- Set up justfile recipes (test, test-security, test-e2e, test-property, test-mock, test)

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
- Update justfile with comprehensive test recipes (test-unit, test-integration, test-e2e, test-security, test-property, test)
- Create CI workflow steps (fmt, clippy, audit, deny, test, security, e2e, property)
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
>
> **A task WITHOUT QA Scenarios is INCOMPLETE. No exceptions.**

### Wave 1: Foundation (Tasks 1-5)

- [ ] 1. Bootstrap @tests/ crate with full dependencies

  **What to do**: Create `@tests/Cargo.toml` with all 22 Seshat TESTING.md dependencies, add to root `Cargo.toml` workspace members, create basic `@tests/src/lib.rs` structure.

  **Must NOT do**: Do not create production dependencies, only dev-dependencies for testing. Do not modify existing crates.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Foundation setup is straightforward crate creation
  - Skills: [] — No special skills needed
  - Omitted: All - setup task

  **Parallelization**: Can Run In Parallel: NO | Wave 1 | Blocks: [2,3,4,5] | Blocked By: []

  **References**:
  - `Seshat/TESTING.md:1-23` — All 22 dependencies listed
  - `Cargo.toml:1` — Root workspace definition, add `tests` to members
  - `apps/Uzume/Uzume-profiles/Cargo.toml:1` — Reference Cargo.toml structure

  **Acceptance Criteria**:
  - [ ] `@tests/Cargo.toml` exists with all 22 dev-dependencies from Seshat/TESTING.md
  - [ ] `@tests/src/lib.rs` exists and compiles
  - [ ] Root `Cargo.toml` includes `tests` in `[workspace]` members
  - [ ] `cargo build -p tests` succeeds without errors

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Cargo.toml has all 22 dependencies
    Tool: Bash
    Steps: cat @tests/Cargo.toml | grep -cE "proptest|criterion|assert_cmd|predicates|insta|rstest|test-case|serial_test|mockall|loom|quickcheck|arbitrary|libfuzzer|tracing|anyhow|thiserror|tempfile|pretty_assertions"
    Expected: Output is 22 or greater (all dependencies present)
    Evidence: .sisyphus/evidence/task-1-dependencies.txt

  Scenario: Workspace integration successful
    Tool: Bash
    Steps: cargo build --workspace && cargo tree -p tests
    Expected: No workspace errors, tests crate visible in dependency tree
    Evidence: .sisyphus/evidence/task-1-workspace-integration.txt

  Scenario: Crate compiles
    Tool: Bash
    Steps: cargo build -p tests
    Expected: Clean build with no errors
    Evidence: .sisyphus/evidence/task-1-build.txt
  ```

  **Commit**: YES | Message: `feat(tests): Bootstrap unified test crate with all Seshat TESTING.md dependencies` | Files: [Cargo.toml, @tests/Cargo.toml, @tests/src/lib.rs]

- [ ] 2. Create test harness structure

  **What to do**: Create modular test harness structure in `@tests/src/` with modules: common/, fixtures/, mocks/, generators/, builders/, asserts/.

  **Must NOT do**: Do not create production code or modify other crates.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Directory structure creation, straightforward
  - Skills: [] — No special skills needed
  - Omitted: All - structure setup

  **Parallelization**: Can Run In Parallel: NO | Wave 1 | Blocks: [3,5] | Blocked By: [1]

  **References**:
  - `Monad/Nun/src/testing.rs:1` — Existing testing utilities structure
  - `apps/Uzume/Uzume-profiles/tests/` — Test organization patterns

  **Acceptance Criteria**:
  - [ ] `@tests/src/common/` directory exists (fixtures, mocks, generators, builders, asserts subdirs)
  - [ ] `@tests/src/lib.rs` re-exports all common modules
  - [ ] All modules have mod.rs files and compile

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Directory structure created
    Tool: Bash
    Steps: ls -la @tests/src/common/
    Expected: Subdirectories fixtures/, mocks/, generators/, builders/, asserts/ present
    Evidence: .sisyphus/evidence/task-2-structure.txt

  Scenario: Modules compile
    Tool: Bash
    Steps: cargo build -p tests
    Expected: No compilation errors in common modules
    Evidence: .sisyphus/evidence/task-2-compile.txt
  ```

  **Commit**: YES | Message: `feat(tests): Create modular test harness structure (common, fixtures, mocks, generators, builders, asserts)` | Files: [@tests/src/common/mod.rs, @tests/src/lib.rs]

- [ ] 3. Extract Nun::testing utilities and extend

  **What to do**: Extract all testing utilities from `Monad/Nun/src/testing.rs` into `@tests/src/common/asserts.rs`, extend with property-based helpers and mock factories.

  **Must NOT do**: Do not modify Nun::testing.rs (keep as-is), do not break existing test patterns.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Code extraction and extension
  - Skills: [] — No special skills needed
  - Omitted: All - straightforward refactoring

  **Parallelization**: Can Run In Parallel: NO | Wave 1 | Blocks: [4,5] | Blocked By: [1,2]

  **References**:
  - `Monad/Nun/src/testing.rs:1` — All testing utilities
  - `Monad/Nun/src/lib.rs:11` — Nun's public API

  **Acceptance Criteria**:
  - [ ] All Nun::testing utilities extracted to `@tests/src/common/asserts.rs`
  - [ ] New property-based helpers added (e.g., `prop_test_id()`, `prop_safe_email()`)
  - [ ] New mock factories added (e.g., `MockAuthenticatorFactory::default()`)
  - [ ] `@tests/src/lib.rs` re-exports all utilities

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All Nun utilities present
    Tool: Bash
    Steps: grep -r "test_id\|test_config\|assert_error" @tests/src/common/asserts.rs
    Expected: All Nun testing functions present (test_id, test_config, assert_error_kind, assert_error_code, assert_ok)
    Evidence: .sisyphus/evidence/task-3-nun-utils.txt

  Scenario: Property-based helpers compile
    Tool: Bash
    Steps: cargo test -p tests --lib common
    Expected: New property functions compile and pass basic sanity tests
    Evidence: .sisyphus/evidence/task-3-property-utils.txt

  Scenario: Re-exports work
    Tool: Bash
    Steps: cargo doc -p tests --no-deps --open
    Expected: Documentation shows test_id, test_config, property helpers, mock factories as public API
    Evidence: .sisyphus/evidence/task-3-reexports.txt
  ```

  **Commit**: YES | Message: `feat(tests): Extract Nun::testing utilities and add property-based helpers, mock factories` | Files: [@tests/src/common/asserts.rs, @tests/src/lib.rs]

- [ ] 4. Fix dev-dependency cycle (move integration_privacy_matrix.rs)

  **What to do**: Move `Monad/Heka/tests/integration_privacy_matrix.rs` to `@tests/integration/heka_link_policy.rs`, remove Heka's dev-dependency on uzume_profiles from `Monad/Heka/Cargo.toml`.

  **Must NOT do**: Do not modify test logic, only move file and remove dependency.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: File move and Cargo.toml edit
  - Skills: [] — No special skills needed
  - Omitted: All - straightforward task

  **Parallelization**: Can Run In Parallel: NO | Wave 1 | Blocks: [5] | Blocked By: [1,2,3]

  **References**:
  - `Monad/Heka/tests/integration_privacy_matrix.rs:1` — Test to move
  - `Monad/Heka/Cargo.toml:26` — Remove uzume_profiles dev-dep
  - `@tests/integration/heka_link_policy.rs` — New location

  **Acceptance Criteria**:
  - [ ] `integration_privacy_matrix.rs` removed from `Monad/Heka/tests/`
  - [ ] `@tests/integration/heka_link_policy.rs` exists with identical content
  - [ ] `uzume_profiles` dev-dependency removed from `Monad/Heka/Cargo.toml`
  - [ ] No dev-dependency cycles detected: `cargo tree --workspace --duplicate`

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Test file moved correctly
    Tool: Bash
    Steps: diff Monad/Heka/tests/integration_privacy_matrix.rs @tests/integration/heka_link_policy.rs
    Expected: Files are identical (diff shows no changes)
    Evidence: .sisyphus/evidence/task-4-file-move.txt

  Scenario: Dev-dependency removed
    Tool: Bash
    Steps: grep uzume_profiles Monad/Heka/Cargo.toml
    Expected: No matches (dependency removed)
    Evidence: .sisyphus/evidence/task-4-removal.txt

  Scenario: No dependency cycles
    Tool: Bash
    Steps: cargo tree --workspace --duplicate
    Expected: No duplicate or cyclic dependencies reported
    Evidence: .sisyphus/evidence/task-4-no-cycles.txt
  ```

  **Commit**: YES | Message: `fix(tests): Break Heka dev-dependency cycle by moving integration_privacy_matrix.rs to @tests/` | Files: [Monad/Heka/Cargo.toml, @tests/integration/heka_link_policy.rs]

- [ ] 5. Set up justfile recipes

  **What to do**: Add comprehensive test recipes to justfile: test, test-unit, test-integration, test-e2e, test-security, test-property, test-mock, test.

  **Must NOT do**: Do not break existing justfile recipes, only add new ones.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: justfile edit
  - Skills: [] — No special skills needed
  - Omitted: All - straightforward task

  **Parallelization**: Can Run In Parallel: NO | Wave 1 | Blocks: [Wave 2] | Blocked By: [1,2,3,4]

  **References**:
  - `justfile:177` — Current test recipes (lines 177-209)
  - `justfile:1` — Just command format

  **Acceptance Criteria**:
  - [ ] `just test` → `cargo nextest run --workspace` (existing, preserve)
  - [ ] `just test-security` → `cargo nextest run --workspace security` (new)
  - [ ] `just test-e2e` → `cargo nextest run --workspace e2e` (new)
  - [ ] `just test-property` → `cargo nextest run --workspace property` (new)
  - [ ] `just test` → runs all test categories sequentially (new)
  - [ ] All recipes work when invoked

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All recipes defined
    Tool: Bash
    Steps: grep -E "test-security|test-e2e|test-property|test" justfile
    Expected: All new recipes present
    Evidence: .sisyphus/evidence/task-5-recipes.txt

  Scenario: Recipes execute correctly
    Tool: Bash
    Steps: just test-security; just test-e2e; just test-property
    Expected: Each recipe runs cargo nextest with appropriate filters
    Evidence: .sisyphus/evidence/task-5-execution.txt

  Scenario: test runs all categories
    Tool: Bash
    Steps: just test
    Expected: security, e2e, property tests all execute (verify in output)
    Evidence: .sisyphus/evidence/task-5-all-tests.txt
  ```

  **Commit**: YES | Message: `feat(tests): Add comprehensive justfile recipes (test-security, test-e2e, test-property, test)` | Files: [justfile]

### Wave 2: Big Bang Migration (Tasks 6-13)

- [ ] 6. Migrate Uzume-profiles tests (7 files)

  **What to do**: Migrate all 7 test files from `apps/Uzume/Uzume-profiles/tests/` to `@tests/uzume-profiles/`. Update imports to use `@tests` common utilities. Verify all tests compile and pass.

  **Must NOT do**: Do not modify test logic or assertions. Do not change production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Migration requires careful import updates, dependency management
  - Skills: [] — Standard migration
  - Omitted: All - straightforward migration

  **Parallelization**: Can Run In Parallel: YES | Wave 2 (with Tasks 7-13) | Blocks: [Wave 3] | Blocked By: [Wave 1]

  **References**:
  - `apps/Uzume/Uzume-profiles/tests/` — Source test files (7 files: api_tests.rs, service_tests.rs, step1_profiles.rs, step1_profiles_endpoints.rs, task11_event_boundary.rs, api/, services/)
  - `@tests/src/lib.rs` — Common utilities to import
  - `apps/Uzume/Uzume-profiles/Cargo.toml:1` — Current dev-dependencies

  **Acceptance Criteria**:
  - [ ] All 7 test files moved to `@tests/uzume-profiles/`
  - [ ] Old test files removed from `apps/Uzume/Uzume-profiles/tests/`
  - [ ] All tests compile with updated imports
  - [ ] `cargo nextest run --workspace uzume-profiles` passes

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All tests migrated
    Tool: Bash
    Steps: ls @tests/uzume-profiles/ | wc -l
    Expected: 7 test files present
    Evidence: .sisyphus/evidence/task-6-migration.txt

  Scenario: Old tests removed
    Tool: Bash
    Steps: ls apps/Uzume/Uzume-profiles/tests/ | wc -l
    Expected: 0 test files remain
    Evidence: .sisyphus/evidence/task-6-cleanup.txt

  Scenario: Tests pass
    Tool: Bash
    Steps: cargo nextest run --workspace uzume-profiles
    Expected: All 7 tests pass (0 failures)
    Evidence: .sisyphus/evidence/task-6-pass.txt
  ```

  **Commit**: YES | Message: `feat(tests): Migrate Uzume-profiles tests (7 files) to unified @tests/` | Files: [@tests/uzume-profiles/*]

- [ ] 7. Migrate Uzume-feed tests (4 files)

  **What to do**: Migrate all 4 test files from `apps/Uzume/Uzume-feed/tests/` to `@tests/uzume-feed/`. Update imports to use `@tests` common utilities. Verify all tests compile and pass.

  **Must NOT do**: Do not modify test logic or assertions. Do not change production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Migration requires careful import updates
  - Skills: [] — Standard migration
  - Omitted: All - straightforward migration

  **Parallelization**: Can Run In Parallel: YES | Wave 2 (with Tasks 6,8-13) | Blocks: [Wave 3] | Blocked By: [Wave 1]

  **References**:
  - `apps/Uzume/Uzume-feed/tests/` — Source test files (4 files: security_tests.rs, step1_feed_chronological.rs, step1_feed_mode_handling.rs, task11_event_boundary.rs)
  - `@tests/src/lib.rs` — Common utilities to import
  - `apps/Uzume/Uzume-feed/Cargo.toml:1` — Current dev-dependencies

  **Acceptance Criteria**:
  - [ ] All 4 test files moved to `@tests/uzume-feed/`
  - [ ] Old test files removed from `apps/Uzume/Uzume-feed/tests/`
  - [ ] All tests compile with updated imports
  - [ ] `cargo nextest run --workspace uzume-feed` passes

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All tests migrated
    Tool: Bash
    Steps: ls @tests/uzume-feed/ | wc -l
    Expected: 4 test files present
    Evidence: .sisyphus/evidence/task-7-migration.txt

  Scenario: Old tests removed
    Tool: Bash
    Steps: ls apps/Uzume/Uzume-feed/tests/ | wc -l
    Expected: 0 test files remain
    Evidence: .sisyphus/evidence/task-7-cleanup.txt

  Scenario: Tests pass
    Tool: Bash
    Steps: cargo nextest run --workspace uzume-feed
    Expected: All 4 tests pass (0 failures)
    Evidence: .sisyphus/evidence/task-7-pass.txt
  ```

  **Commit**: YES | Message: `feat(tests): Migrate Uzume-feed tests (4 files) to unified @tests/` | Files: [@tests/uzume-feed/*]

- [ ] 8. Migrate Heimdall tests (5 files)

  **What to do**: Migrate all 5 test files from `Monad/Heimdall/tests/` to `@tests/heimdall/`. Update imports to use `@tests` common utilities. Verify all tests compile and pass.

  **Must NOT do**: Do not modify test logic or assertions. Do not change production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Migration requires careful import updates
  - Skills: [] — Standard migration
  - Omitted: All - straightforward migration

  **Parallelization**: Can Run In Parallel: YES | Wave 2 (with Tasks 6-7,9-13) | Blocks: [Wave 3] | Blocked By: [Wave 1]

  **References**:
  - `Monad/Heimdall/tests/` — Source test files (5 files: integration_test.rs, proxy_test.rs, health_test.rs, jwt_test.rs, auth_layer_test.rs, config_test.rs)
  - `@tests/src/lib.rs` — Common utilities to import
  - `Monad/Heimdall/Cargo.toml:1` — Current dev-dependencies

  **Acceptance Criteria**:
  - [ ] All 5 test files moved to `@tests/heimdall/`
  - [ ] Old test files removed from `Monad/Heimdall/tests/`
  - [ ] All tests compile with updated imports
  - [ ] `cargo nextest run --workspace heimdall` passes

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All tests migrated
    Tool: Bash
    Steps: ls @tests/heimdall/ | wc -l
    Expected: 5 test files present
    Evidence: .sisyphus/evidence/task-8-migration.txt

  Scenario: Old tests removed
    Tool: Bash
    Steps: ls Monad/Heimdall/tests/ | wc -l
    Expected: 0 test files remain
    Evidence: .sisyphus/evidence/task-8-cleanup.txt

  Scenario: Tests pass
    Tool: Bash
    Steps: cargo nextest run --workspace heimdall
    Expected: All 5 tests pass (0 failures)
    Evidence: .sisyphus/evidence/task-8-pass.txt
  ```

  **Commit**: YES | Message: `feat(tests): Migrate Heimdall tests (5 files) to unified @tests/` | Files: [@tests/heimdall/*]

- [ ] 9. Migrate xtask tests (5 files)

  **What to do**: Migrate all 5 test files from `Monad/xtask/tests/` to `@tests/xtask/`. Update imports to use `@tests` common utilities. Verify all tests compile and pass.

  **Must NOT do**: Do not modify test logic or assertions. Do not change production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Migration requires careful import updates
  - Skills: [] — Standard migration
  - Omitted: All - straightforward migration

  **Parallelization**: Can Run In Parallel: YES | Wave 2 (with Tasks 6-8,10-13) | Blocks: [Wave 3] | Blocked By: [Wave 1]

  **References**:
  - `Monad/xtask/tests/` — Source test files (5 files: migrate_test.rs, seed_test.rs, db_reset_test.rs, nats_setup_test.rs, new_app_test.rs)
  - `@tests/src/lib.rs` — Common utilities to import
  - `Monad/xtask/Cargo.toml:1` — Current dev-dependencies

  **Acceptance Criteria**:
  - [ ] All 5 test files moved to `@tests/xtask/`
  - [ ] Old test files removed from `Monad/xtask/tests/`
  - [ ] All tests compile with updated imports
  - [ ] `cargo nextest run --workspace xtask` passes

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All tests migrated
    Tool: Bash
    Steps: ls @tests/xtask/ | wc -l
    Expected: 5 test files present
    Evidence: .sisyphus/evidence/task-9-migration.txt

  Scenario: Old tests removed
    Tool: Bash
    Steps: ls Monad/xtask/tests/ | wc -l
    Expected: 0 test files remain
    Evidence: .sisyphus/evidence/task-9-cleanup.txt

  Scenario: Tests pass
    Tool: Bash
    Steps: cargo nextest run --workspace xtask
    Expected: All 5 tests pass (0 failures)
    Evidence: .sisyphus/evidence/task-9-pass.txt
  ```

  **Commit**: YES | Message: `feat(tests): Migrate xtask tests (5 files) to unified @tests/` | Files: [@tests/xtask/*]

- [ ] 10. Migrate Heka tests (3 files, excluding integration_privacy_matrix.rs)

  **What to do**: Migrate 3 test files from `Monad/Heka/tests/` to `@tests/heka/` (excluding integration_privacy_matrix.rs which moved to cross-component in Task 4). Update imports to use `@tests` common utilities. Verify all tests compile and pass.

  **Must NOT do**: Do not modify test logic or assertions. Do not change production code. Do not include integration_privacy_matrix.rs (already moved).

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Migration requires careful import updates
  - Skills: [] — Standard migration
  - Omitted: All - straightforward migration

  **Parallelization**: Can Run In Parallel: YES | Wave 2 (with Tasks 6-9,11-13) | Blocks: [Wave 3] | Blocked By: [Wave 1, 4]

  **References**:
  - `Monad/Heka/tests/` — Source test files (3 files: kratos_client_core.rs, link_policy_engine.rs)
  - `@tests/src/lib.rs` — Common utilities to import
  - `Monad/Heka/Cargo.toml:1` — Current dev-dependencies

  **Acceptance Criteria**:
  - [ ] All 3 test files moved to `@tests/heka/`
  - [ ] Old test files removed from `Monad/Heka/tests/`
  - [ ] All tests compile with updated imports
  - [ ] `cargo nextest run --workspace heka` passes

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All tests migrated
    Tool: Bash
    Steps: ls @tests/heka/ | wc -l
    Expected: 3 test files present
    Evidence: .sisyphus/evidence/task-10-migration.txt

  Scenario: Old tests removed
    Tool: Bash
    Steps: ls Monad/Heka/tests/ | wc -l
    Expected: 0 test files remain
    Evidence: .sisyphus/evidence/task-10-cleanup.txt

  Scenario: Tests pass
    Tool: Bash
    Steps: cargo nextest run --workspace heka
    Expected: All 3 tests pass (0 failures)
    Evidence: .sisyphus/evidence/task-10-pass.txt
  ```

  **Commit**: YES | Message: `feat(tests): Migrate Heka tests (3 files) to unified @tests/` | Files: [@tests/heka/*]

- [ ] 11. Migrate Uzume-stories, Uzume-reels, Uzume-discover tests (3 files total)

  **What to do**: Migrate test files from `apps/Uzume/Uzume-stories/tests/`, `apps/Uzume/Uzume-reels/tests/`, `apps/Uzume/Uzume-discover/tests/` to `@tests/uzume-stories/`, `@tests/uzume-reels/`, `@tests/uzume-discover/`. Update imports to use `@tests` common utilities. Verify all tests compile and pass.

  **Must NOT do**: Do not modify test logic or assertions. Do not change production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Migration requires careful import updates
  - Skills: [] — Standard migration
  - Omitted: All - straightforward migration

  **Parallelization**: Can Run In Parallel: YES | Wave 2 (with Tasks 6-10,12-13) | Blocks: [Wave 3] | Blocked By: [Wave 1]

  **References**:
  - `apps/Uzume/Uzume-stories/tests/` — Source test files
  - `apps/Uzume/Uzume-reels/tests/` — Source test files
  - `apps/Uzume/Uzume-discover/tests/` — Source test files
  - `@tests/src/lib.rs` — Common utilities to import

  **Acceptance Criteria**:
  - [ ] All test files moved to `@tests/uzume-stories/`, `@tests/uzume-reels/`, `@tests/uzume-discover/`
  - [ ] Old test files removed from source locations
  - [ ] All tests compile with updated imports
  - [ ] `cargo nextest run --workspace uzume-stories uzume-reels uzume-discover` passes

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All tests migrated
    Tool: Bash
    Steps: ls @tests/uzume-stories/ @tests/uzume-reels/ @tests/uzume-discover/ | wc -l
    Expected: 3 test files present total
    Evidence: .sisyphus/evidence/task-11-migration.txt

  Scenario: Old tests removed
    Tool: Bash
    Steps: ls apps/Uzume/Uzume-stories/tests/ apps/Uzume/Uzume-reels/tests/ apps/Uzume/Uzume-discover/tests/ | wc -l
    Expected: 0 test files remain
    Evidence: .sisyphus/evidence/task-11-cleanup.txt

  Scenario: Tests pass
    Tool: Bash
    Steps: cargo nextest run --workspace uzume-stories uzume-reels uzume-discover
    Expected: All 3 tests pass (0 failures)
    Evidence: .sisyphus/evidence/task-11-pass.txt
  ```

  **Commit**: YES | Message: `feat(tests): Migrate Uzume-stories, Uzume-reels, Uzume-discover tests (3 files) to unified @tests/` | Files: [@tests/uzume-stories/*, @tests/uzume-reels/*, @tests/uzume-discover/*]

- [ ] 12. Migrate Monad library tests (Lethe, Akash, api, events, Mnemosyne - 5 files total)

  **What to do**: Migrate test files from `Monad/Lethe/tests/`, `Monad/Akash/tests/`, `Monad/api/tests/`, `Monad/events/tests/`, `Monad/Mnemosyne/tests/` to `@tests/lethe/`, `@tests/akash/`, `@tests/api/`, `@tests/events/`, `@tests/mnemosyne/`. Update imports to use `@tests` common utilities. Verify all tests compile and pass.

  **Must NOT do**: Do not modify test logic or assertions. Do not change production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Migration requires careful import updates
  - Skills: [] — Standard migration
  - Omitted: All - straightforward migration

  **Parallelization**: Can Run In Parallel: YES | Wave 2 (with Tasks 6-11,13) | Blocks: [Wave 3] | Blocked By: [Wave 1]

  **References**:
  - `Monad/Lethe/tests/` — stories_helpers_test.rs
  - `Monad/Akash/tests/` — stories_presign.rs
  - `Monad/api/tests/` — contracts_test.rs
  - `Monad/events/tests/` — event_boundary.rs
  - `Monad/Mnemosyne/tests/` — helpers_test.rs
  - `@tests/src/lib.rs` — Common utilities to import

  **Acceptance Criteria**:
  - [ ] All 5 test files moved to `@tests/lethe/`, `@tests/akash/`, `@tests/api/`, `@tests/events/`, `@tests/mnemosyne/`
  - [ ] Old test files removed from source locations
  - [ ] All tests compile with updated imports
  - [ ] `cargo nextest run --workspace lethe akash api events mnemosyne` passes

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All tests migrated
    Tool: Bash
    Steps: ls @tests/lethe/ @tests/akash/ @tests/api/ @tests/events/ @tests/mnemosyne/ | wc -l
    Expected: 5 test files present total
    Evidence: .sisyphus/evidence/task-12-migration.txt

  Scenario: Old tests removed
    Tool: Bash
    Steps: ls Monad/Lethe/tests/ Monad/Akash/tests/ Monad/api/tests/ Monad/events/tests/ Monad/Mnemosyne/tests/ | wc -l
    Expected: 0 test files remain
    Evidence: .sisyphus/evidence/task-12-cleanup.txt

  Scenario: Tests pass
    Tool: Bash
    Steps: cargo nextest run --workspace lethe akash api events mnemosyne
    Expected: All 5 tests pass (0 failures)
    Evidence: .sisyphus/evidence/task-12-pass.txt
  ```

  **Commit**: YES | Message: `feat(tests): Migrate Monad library tests (5 files) to unified @tests/` | Files: [@tests/lethe/*, @tests/akash/*, @tests/api/*, @tests/events/*, @tests/mnemosyne/*]

- [ ] 13. Move cross-component test to @tests/integration/heka_link_policy.rs

  **What to do**: Move `integration_privacy_matrix.rs` from `Monad/Heka/tests/` to `@tests/integration/heka_link_policy.rs`. Update imports to use `@tests` common utilities and workspace crates. Verify test compiles and passes.

  **Must NOT do**: Do not modify test logic or assertions. Do not change production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Cross-component test requires careful dependency management
  - Skills: [] — Standard migration
  - Omitted: All - straightforward migration

  **Parallelization**: Can Run In Parallel: YES | Wave 2 (with Tasks 6-12) | Blocks: [Wave 3] | Blocked By: [Wave 1, 4]

  **References**:
  - `Monad/Heka/tests/integration_privacy_matrix.rs:1` — Source test (311 lines)
  - `@tests/src/lib.rs` — Common utilities to import
  - `apps/Uzume/Uzume-profiles/Cargo.toml:1` — uzume_profiles crate
  - `Monad/Heka/Cargo.toml:1` — heka crate

  **Acceptance Criteria**:
  - [ ] `integration_privacy_matrix.rs` moved to `@tests/integration/heka_link_policy.rs`
  - [ ] Old test file removed from `Monad/Heka/tests/`
  - [ ] Test compiles with updated imports
  - [ ] `cargo nextest run --workspace heka_link_policy` passes

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Test file moved correctly
    Tool: Bash
    Steps: ls @tests/integration/heka_link_policy.rs
    Expected: File exists
    Evidence: .sisyphus/evidence/task-13-move.txt

  Scenario: Old test removed
    Tool: Bash
    Steps: ls Monad/Heka/tests/integration_privacy_matrix.rs
    Expected: File does not exist (error expected)
    Evidence: .sisyphus/evidence/task-13-cleanup.txt

  Scenario: Test passes
    Tool: Bash
    Steps: cargo nextest run --workspace heka_link_policy
    Expected: All 8 tests pass (0 failures)
    Evidence: .sisyphus/evidence/task-13-pass.txt
  ```

  **Commit**: YES | Message: `feat(tests): Move cross-component test to @tests/integration/heka_link_policy.rs` | Files: [@tests/integration/heka_link_policy.rs]

### Wave 3: Security Sweep (Tasks 14-19)

- [ ] 14. Implement SQL injection tests

  **What to do**: Create comprehensive SQL injection test suite covering all database entry points. Test payloads: `' OR '1'='1`, `'; DROP TABLE users; --`, `1 UNION SELECT * FROM users--`, null bytes, encoded payloads. Test all API endpoints that accept user input (profiles, posts, comments, search, etc.). Verify proper sanitization and parameterized queries.

  **Must NOT do**: Do not modify production code. Do not create actual SQL injection vulnerabilities.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Security testing requires careful edge case coverage
  - Skills: [] — Security testing
  - Omitted: All - security-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 3 (with Tasks 15-19) | Blocks: [Wave 5] | Blocked By: [Wave 1, 2]

  **References**:
  - `Monad/Mnemosyne/src/` — Database layer, check for parameterized queries
  - `apps/Uzume/Uzume-profiles/src/queries/` — Profile queries
  - `apps/Uzume/Uzume-feed/src/queries/` — Feed queries
  - `@tests/src/common/` — Common test utilities

  **Acceptance Criteria**:
  - [ ] SQL injection test suite created in `@tests/security/sql_injection.rs`
  - [ ] Tests cover all API endpoints accepting user input
  - [ ] Tests verify parameterized query usage
  - [ ] Tests verify proper error handling for malicious input
  - [ ] All tests pass (expected: injection attempts blocked)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: SQL injection blocked at profile endpoint
    Tool: Bash
    Steps: Create test with payload "' OR '1'='1" in profile alias field, execute against test DB
    Expected: Request returns 400 Bad Request or 422 Unprocessable Entity, no data leakage
    Evidence: .sisyphus/evidence/task-14-profile-sqli.txt

  Scenario: SQL injection blocked at feed endpoint
    Tool: Bash
    Steps: Create test with payload "'; DROP TABLE users; --" in post caption, execute against test DB
    Expected: Request returns 400/422, no table dropped, database intact
    Evidence: .sisyphus/evidence/task-14-feed-sqli.txt

  Scenario: SQL injection blocked at search endpoint
    Tool: Bash
    Steps: Create test with payload "1 UNION SELECT * FROM users--" in search query, execute against test DB
    Expected: Request returns 400/422, no data leakage
    Evidence: .sisyphus/evidence/task-14-search-sqli.txt
  ```

  **Commit**: YES | Message: `feat(tests): Add SQL injection test suite covering all API endpoints` | Files: [@tests/security/sql_injection.rs]

- [ ] 15. Implement XSS tests

  **What to do**: Create comprehensive XSS test suite covering all user input fields. Test payloads: `<script>alert('xss')</script>`, `<img src=x onerror=alert(1)>`, `javascript:alert(1)`, encoded variants, SVG injection, CSS injection. Test profile fields (bio, display name), post content, comments, search queries. Verify proper sanitization and output encoding.

  **Must NOT do**: Do not modify production code. Do not create actual XSS vulnerabilities.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Security testing requires careful edge case coverage
  - Skills: [] — Security testing
  - Omitted: All - security-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 3 (with Tasks 14,16-19) | Blocks: [Wave 5] | Blocked By: [Wave 1, 2]

  **References**:
  - `apps/Uzume/Uzume-profiles/src/` — Profile fields (bio, display_name)
  - `apps/Uzume/Uzume-feed/src/` — Post content, comments
  - `@tests/src/common/` — Common test utilities

  **Acceptance Criteria**:
  - [ ] XSS test suite created in `@tests/security/xss.rs`
  - [ ] Tests cover all user input fields (bio, display name, post content, comments, search)
  - [ ] Tests verify proper sanitization and output encoding
  - [ ] Tests verify no script execution in rendered output
  - [ ] All tests pass (expected: XSS attempts blocked/sanitized)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: XSS blocked in profile bio
    Tool: Bash
    Steps: Create test with payload "<script>alert('xss')</script>" in bio field, retrieve profile
    Expected: Bio is sanitized (script tags escaped or removed), no script execution
    Evidence: .sisyphus/evidence/task-15-bio-xss.txt

  Scenario: XSS blocked in post content
    Tool: Bash
    Steps: Create test with payload "<img src=x onerror=alert(1)>" in post caption, retrieve post
    Expected: Caption is sanitized, no script execution
    Evidence: .sisyphus/evidence/task-15-post-xss.txt

  Scenario: XSS blocked in comments
    Tool: Bash
    Steps: Create test with payload "javascript:alert(1)" in comment text, retrieve comment
    Expected: Comment text is sanitized, no script execution
    Evidence: .sisyphus/evidence/task-15-comment-xss.txt
  ```

  **Commit**: YES | Message: `feat(tests): Add XSS test suite covering all user input fields` | Files: [@tests/security/xss.rs]

- [ ] 16. Implement authz bypass tests

  **What to do**: Create comprehensive authorization bypass test suite. Test scenarios: user A accessing user B's resources, privilege escalation, token manipulation, session fixation, cross-app access control, forged app context. Test all protected endpoints (profile updates, post deletion, follow/unfollow, block/unblock). Verify proper authorization checks at every level.

  **Must NOT do**: Do not modify production code. Do not create actual authz bypass vulnerabilities.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Security testing requires careful edge case coverage
  - Skills: [] — Security testing
  - Omitted: All - security-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 3 (with Tasks 14-15,17-19) | Blocks: [Wave 5] | Blocked By: [Wave 1, 2]

  **References**:
  - `Monad/Heka/src/link_policy.rs:1` — Link policy engine
  - `apps/Uzume/Uzume-profiles/src/services/` — Profile services
  - `apps/Uzume/Uzume-feed/src/services/` — Feed services
  - `@tests/integration/heka_link_policy.rs:1` — Existing cross-component test

  **Acceptance Criteria**:
  - [ ] Authz bypass test suite created in `@tests/security/authz_bypass.rs`
  - [ ] Tests cover all protected endpoints
  - [ ] Tests verify proper authorization checks
  - [ ] Tests verify no cross-app access without proper links
  - [ ] All tests pass (expected: unauthorized access blocked)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: User A cannot access user B's private profile
    Tool: Bash
    Steps: Create user A and user B, set B's profile to private, attempt A accessing B's profile with A's token
    Expected: Request returns 403 Forbidden
    Evidence: .sisyphus/evidence/task-16-private-profile.txt

  Scenario: User A cannot delete user B's post
    Tool: Bash
    Steps: Create user A and user B, B creates post, attempt A deleting B's post with A's token
    Expected: Request returns 403 Forbidden
    Evidence: .sisyphus/evidence/task-16-delete-post.txt

  Scenario: Token manipulation does not grant elevated access
    Tool: Bash
    Steps: Create test with modified/expired token, attempt to access protected endpoint
    Expected: Request returns 401 Unauthorized
    Evidence: .sisyphus/evidence/task-16-token-manipulation.txt
  ```

  **Commit**: YES | Message: `feat(tests): Add authorization bypass test suite covering all protected endpoints` | Files: [@tests/security/authz_bypass.rs]

- [ ] 17. Implement privacy violation tests

  **What to do**: Create comprehensive privacy violation test suite. Test scenarios: PII exposure (phone, email in responses), alias leakage (global identity exposed), identity mapping leaks, cross-app data leakage, follow graph exposure, block/mute bypass. Verify all API responses contain only permitted data.

  **Must NOT do**: Do not modify production code. Do not create actual privacy violations.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Privacy testing requires careful data flow analysis
  - Skills: [] — Privacy testing
  - Omitted: All - privacy-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 3 (with Tasks 14-16,18-19) | Blocks: [Wave 5] | Blocked By: [Wave 1, 2]

  **References**:
  - `Monad/Heka/src/types.rs:1` — Identity types
  - `apps/Uzume/Uzume-profiles/src/models/` — Profile models (check for PII fields)
  - `Monad/Heka/src/alias.rs:1` — Alias system
  - `@tests/src/common/` — Common test utilities

  **Acceptance Criteria**:
  - [ ] Privacy violation test suite created in `@tests/security/privacy_violations.rs`
  - [ ] Tests cover all API responses for PII leakage
  - [ ] Tests verify alias isolation (no global identity exposure)
  - [ ] Tests verify cross-app data isolation
  - [ ] All tests pass (expected: no privacy violations)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Profile response does not expose global identity
    Tool: Bash
    Steps: Create profile, retrieve via API, check response fields
    Expected: Response contains alias, display_name, but NOT nyx_identity_id or email
    Evidence: .sisyphus/evidence/task-17-identity-leak.txt

  Scenario: Follow graph does not expose blocked users
    Tool: Bash
    Steps: User A blocks user B, user C queries A's followers
    Expected: User B not visible in any response
    Evidence: .sisyphus/evidence/task-17-block-exposure.txt

  Scenario: Cross-app alias isolation maintained
    Tool: Bash
    Steps: Create alias in Uzume, attempt to resolve in Anteros
    Expected: Alias not resolvable in different app context
    Evidence: .sisyphus/evidence/task-17-cross-app-leak.txt
  ```

  **Commit**: YES | Message: `feat(tests): Add privacy violation test suite covering PII exposure and alias leakage` | Files: [@tests/security/privacy_violations.rs]

- [ ] 18. Implement edge case tests

  **What to do**: Create comprehensive edge case test suite. Test scenarios: boundary conditions (max/min values, empty strings, oversized inputs), concurrent access (race conditions, simultaneous updates), malformed input (invalid JSON, missing fields, wrong types), network failures (timeout, connection refused), database failures (constraint violations, deadlocks). Verify graceful error handling in all scenarios.

  **Must NOT do**: Do not modify production code. Do not create actual edge case vulnerabilities.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Edge case testing requires comprehensive scenario coverage
  - Skills: [] — Edge case testing
  - Omitted: All - edge case-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 3 (with Tasks 14-17,19) | Blocks: [Wave 5] | Blocked By: [Wave 1, 2]

  **References**:
  - `Monad/Nun/src/testing.rs:1` — Existing test utilities
  - `apps/Uzume/Uzume-profiles/src/` — Profile services
  - `apps/Uzume/Uzume-feed/src/` — Feed services
  - `@tests/src/common/` — Common test utilities

  **Acceptance Criteria**:
  - [ ] Edge case test suite created in `@tests/security/edge_cases.rs`
  - [ ] Tests cover boundary conditions, concurrent access, malformed input, network failures
  - [ ] Tests verify graceful error handling
  - [ ] All tests pass (expected: edge cases handled gracefully)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Empty string input handled gracefully
    Tool: Bash
    Steps: Create profile with empty display_name, attempt to retrieve
    Expected: Request returns 422 Unprocessable Entity or validation error
    Evidence: .sisyphus/evidence/task-18-empty-input.txt

  Scenario: Oversized input rejected
    Tool: Bash
    Steps: Create post with 100KB caption, attempt to create
    Expected: Request returns 413 Payload Too Large or 422 Unprocessable Entity
    Evidence: .sisyphus/evidence/task-18-oversized-input.txt

  Scenario: Concurrent profile updates handled safely
    Tool: Bash
    Steps: Create two simultaneous requests to update same profile field
    Expected: One succeeds, other returns conflict error or both handled safely
    Evidence: .sisyphus/evidence/task-18-concurrent-update.txt
  ```

  **Commit**: YES | Message: `feat(tests): Add edge case test suite covering boundary conditions and concurrent access` | Files: [@tests/security/edge_cases.rs]

- [ ] 19. Implement fuzzing harness

  **What to do**: Create fuzzing harness using libfuzzer-sys for critical paths. Target: input parsing functions, authentication logic, authorization checks, database query builders. Set up cargo-fuzz integration. Create initial fuzz targets for high-risk areas (user input processing, token validation, alias resolution).

  **Must NOT do**: Do not modify production code. Do not create actual fuzzing vulnerabilities.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Fuzzing requires specialized setup and target identification
  - Skills: [] — Fuzzing
  - Omitted: All - fuzzing-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 3 (with Tasks 14-18) | Blocks: [Wave 5] | Blocked By: [Wave 1, 2]

  **References**:
  - `Monad/Heka/src/client.rs:1` — Kratos client (token validation)
  - `Monad/Heka/src/alias.rs:1` — Alias resolver
  - `Monad/Nun/src/` — Input validation
  - `@tests/src/common/` — Common test utilities

  **Acceptance Criteria**:
  - [ ] Fuzzing harness created in `@tests/fuzz/`
  - [ ] cargo-fuzz integration configured
  - [ ] Initial fuzz targets for high-risk areas (token validation, alias resolution, input parsing)
  - [ ] Fuzz targets compile and run

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Fuzz harness compiles
    Tool: Bash
    Steps: cargo fuzz build --target fuzz_target_token_validation
    Expected: Build succeeds with no errors
    Evidence: .sisyphus/evidence/task-19-fuzz-build.txt

  Scenario: Fuzz target runs
    Tool: Bash
    Steps: cargo fuzz run fuzz_target_token_validation -- -max_total_time=5
    Expected: Fuzz target runs for 5 seconds without crashes
    Evidence: .sisyphus/evidence/task-19-fuzz-run.txt
  ```

  **Commit**: YES | Message: `feat(tests): Add fuzzing harness with initial targets for critical paths` | Files: [@tests/fuzz/*]

### Wave 4: E2E Sandbox Infrastructure (Tasks 20-24)

- [ ] 20. Create sandbox orchestration module

  **What to do**: Create `@tests/src/sandbox/mod.rs` with SandboxManager struct that manages service lifecycle (start, stop, cleanup). Implement service discovery, health checks, port allocation, container lifecycle management. Support parallel test execution with isolated sandboxes.

  **Must NOT do**: Do not create production infrastructure. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Sandbox orchestration requires complex lifecycle management
  - Skills: [] — Infrastructure
  - Omitted: All - infrastructure-focused

  **Parallelization**: Can Run In Parallel: NO | Wave 4 | Blocks: [21,22,23,24] | Blocked By: [Wave 1, 2]

  **References**:
  - `testcontainers` — Workspace dependency for container management
  - `Monad/Mnemosyne/src/` — Database layer patterns
  - `Monad/events/src/` — NATS patterns
  - `@tests/src/common/` — Common test utilities

  **Acceptance Criteria**:
  - [ ] SandboxManager struct created with start/stop/cleanup methods
  - [ ] Service discovery mechanism implemented
  - [ ] Health checks for all services
  - [ ] Port allocation for parallel execution
  - [ ] Container lifecycle management (start, stop, cleanup)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: SandboxManager starts successfully
    Tool: Bash
    Steps: Create SandboxManager instance, call start(), verify services running
    Expected: All services start, health checks pass
    Evidence: .sisyphus/evidence/task-20-start.txt

  Scenario: SandboxManager cleans up properly
    Tool: Bash
    Steps: Create SandboxManager, start, stop, verify no orphaned containers
    Expected: All containers stopped, no orphaned resources
    Evidence: .sisyphus/evidence/task-20-cleanup.txt
  ```

  **Commit**: YES | Message: `feat(tests): Create sandbox orchestration module with service lifecycle management` | Files: [@tests/src/sandbox/mod.rs]

- [ ] 21. Implement postgres sandbox

  **What to do**: Create postgres sandbox using testcontainers with schema migrations. Implement database isolation per test, migration application, cleanup. Support parallel test execution with isolated databases.

  **Must NOT do**: Do not create production database. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Database sandbox requires migration and isolation handling
  - Skills: [] — Infrastructure
  - Omitted: All - infrastructure-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 4 (with Tasks 22-24) | Blocks: [Wave 5] | Blocked By: [20]

  **References**:
  - `testcontainers-modules` — Pre-built postgres container
  - `Monad/Mnemosyne/src/` — Database layer, migration patterns
  - `Monad/xtask/src/` — Migration commands

  **Acceptance Criteria**:
  - [ ] Postgres sandbox created with testcontainers
  - [ ] Schema migrations applied automatically
  - [ ] Database isolation per test
  - [ ] Cleanup after test completion

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Postgres sandbox starts with migrations
    Tool: Bash
    Steps: Create PostgresSandbox, start, verify migrations applied
    Expected: All migrations applied, database ready
    Evidence: .sisyphus/evidence/task-21-start.txt

  Scenario: Database isolation works
    Tool: Bash
    Steps: Create two test databases, insert different data, verify isolation
    Expected: Data in database A not visible in database B
    Evidence: .sisyphus/evidence/task-21-isolation.txt
  ```

  **Commit**: YES | Message: `feat(tests): Implement postgres sandbox with testcontainers and migrations` | Files: [@tests/src/sandbox/postgres.rs]

- [ ] 22. Implement NATS sandbox

  **What to do**: Create NATS sandbox using testcontainers with JetStream enabled. Implement stream creation, consumer setup, message publishing/subscribing for tests. Support parallel test execution with isolated streams.

  **Must NOT do**: Do not create production NATS. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: NATS sandbox requires JetStream and stream management
  - Skills: [] — Infrastructure
  - Omitted: All - infrastructure-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 4 (with Tasks 21,23-24) | Blocks: [Wave 5] | Blocked By: [20]

  **References**:
  - `testcontainers-modules` — Pre-built NATS container
  - `Monad/events/src/` — NATS patterns, JetStream usage
  - `Monad/events/Cargo.toml:1` — Events crate dependencies

  **Acceptance Criteria**:
  - [ ] NATS sandbox created with testcontainers
  - [ ] JetStream enabled
  - [ ] Stream creation and consumer setup
  - [ ] Message publishing/subscribing for tests

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: NATS sandbox starts with JetStream
    Tool: Bash
    Steps: Create NatsSandbox, start, verify JetStream enabled
    Expected: JetStream enabled, streams can be created
    Evidence: .sisyphus/evidence/task-22-start.txt

  Scenario: Message publishing and subscribing works
    Tool: Bash
    Steps: Publish message to stream, subscribe, verify message received
    Expected: Message received with correct content
    Evidence: .sisyphus/evidence/task-22-messaging.txt
  ```

  **Commit**: YES | Message: `feat(tests): Implement NATS sandbox with JetStream support` | Files: [@tests/src/sandbox/nats.rs]

- [ ] 23. Implement DragonflyDB sandbox

  **What to do**: Create DragonflyDB sandbox using testcontainers with cache isolation. Implement key isolation per test, TTL management, cleanup. Support parallel test execution with isolated caches.

  **Must NOT do**: Do not create production cache. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Cache sandbox requires isolation and TTL management
  - Skills: [] — Infrastructure
  - Omitted: All - infrastructure-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 4 (with Tasks 21-22,24) | Blocks: [Wave 5] | Blocked By: [20]

  **References**:
  - `testcontainers-modules` — Pre-built redis/dragonfly container
  - `Monad/Lethe/src/` — Cache patterns, DragonflyDB usage
  - `Monad/Lethe/Cargo.toml:1` — Lethe crate dependencies

  **Acceptance Criteria**:
  - [ ] DragonflyDB sandbox created with testcontainers
  - [ ] Key isolation per test
  - [ ] TTL management
  - [ ] Cleanup after test completion

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: DragonflyDB sandbox starts
    Tool: Bash
    Steps: Create DragonflySandbox, start, verify connection
    Expected: Connection successful, cache ready
    Evidence: .sisyphus/evidence/task-23-start.txt

  Scenario: Key isolation works
    Tool: Bash
    Steps: Create two test caches, set different keys, verify isolation
    Expected: Keys in cache A not visible in cache B
    Evidence: .sisyphus/evidence/task-23-isolation.txt
  ```

  **Commit**: YES | Message: `feat(tests): Implement DragonflyDB sandbox with cache isolation` | Files: [@tests/src/sandbox/dragonfly.rs]

- [ ] 24. Implement remaining service sandboxes

  **What to do**: Create sandboxes for MinIO (S3), Kratos (identity), Matrix (messaging), Meilisearch (search) using testcontainers. Implement service-specific setup, health checks, cleanup. Support parallel test execution with isolated services.

  **Must NOT do**: Do not create production services. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Multiple service sandboxes require diverse configuration
  - Skills: [] — Infrastructure
  - Omitted: All - infrastructure-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 4 (with Tasks 21-23) | Blocks: [Wave 5] | Blocked By: [20]

  **References**:
  - `Monad/Akash/src/` — MinIO/S3 patterns
  - `Monad/Brizo/src/` — Meilisearch patterns
  - `Monad/Ogma/src/` — Matrix patterns
  - `Monad/Heka/src/` — Kratos patterns

  **Acceptance Criteria**:
  - [ ] MinIO sandbox created with testcontainers
  - [ ] Kratos sandbox created with testcontainers
  - [ ] Matrix sandbox created with testcontainers
  - [ ] Meilisearch sandbox created with testcontainers
  - [ ] All sandboxes support parallel execution with isolation

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All service sandboxes start
    Tool: Bash
    Steps: Create all sandboxes, start, verify health checks
    Expected: All services start, health checks pass
    Evidence: .sisyphus/evidence/task-24-start.txt

  Scenario: All service sandboxes clean up
    Tool: Bash
    Steps: Start all sandboxes, stop, verify no orphaned containers
    Expected: All containers stopped, no orphaned resources
    Evidence: .sisyphus/evidence/task-24-cleanup.txt
  ```

  **Commit**: YES | Message: `feat(tests): Implement remaining service sandboxes (MinIO, Kratos, Matrix, Meilisearch)` | Files: [@tests/src/sandbox/minio.rs, @tests/src/sandbox/kratos.rs, @tests/src/sandbox/matrix.rs, @tests/src/sandbox/meilisearch.rs]

### Wave 5: E2E Full Flow Tests (Tasks 25-28)

- [ ] 25. Create user registration & authentication flow

  **What to do**: Create E2E test for full user registration and authentication flow: signup → Kratos identity creation → alias generation → profile setup → session validation. Use real sandbox infrastructure (postgres, NATS, Kratos). Verify all steps complete successfully.

  **Must NOT do**: Do not modify production code. Do not use mocked services.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: E2E flow requires real infrastructure and complex orchestration
  - Skills: [] — E2E testing
  - Omitted: All - E2E-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 5 (with Tasks 26-28) | Blocks: [Final Verification] | Blocked By: [Wave 1, 2, 3, 4]

  **References**:
  - `Monad/Heka/src/` — Kratos client, identity management
  - `apps/Uzume/Uzume-profiles/src/` — Profile creation
  - `@tests/src/sandbox/` — Sandbox infrastructure

  **Acceptance Criteria**:
  - [ ] E2E registration flow test created in `@tests/e2e/registration.rs`
  - [ ] Test uses real sandbox infrastructure (postgres, NATS, Kratos)
  - [ ] Test verifies complete flow: signup → identity → alias → profile → session
  - [ ] Test passes with real services

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Complete registration flow succeeds
    Tool: Bash
    Steps: Run E2E registration test with real sandbox services
    Expected: All steps complete successfully, user registered with profile
    Evidence: .sisyphus/evidence/task-25-registration.txt

  Scenario: Registration with invalid data fails gracefully
    Tool: Bash
    Steps: Run registration test with invalid email/phone, verify error handling
    Expected: Request returns 422 Unprocessable Entity with validation errors
    Evidence: .sisyphus/evidence/task-25-invalid-registration.txt
  ```

  **Commit**: YES | Message: `feat(tests): Create user registration & authentication E2E flow test` | Files: [@tests/e2e/registration.rs]

- [ ] 26. Create content creation & interaction flow

  **What to do**: Create E2E test for full content creation and interaction flow: create post → feed generation → like → comment → timeline update. Use real sandbox infrastructure (postgres, NATS, DragonflyDB). Verify all steps complete successfully.

  **Must NOT do**: Do not modify production code. Do not use mocked services.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: E2E flow requires real infrastructure and complex orchestration
  - Skills: [] — E2E testing
  - Omitted: All - E2E-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 5 (with Tasks 25,27-28) | Blocks: [Final Verification] | Blocked By: [Wave 1, 2, 3, 4]

  **References**:
  - `apps/Uzume/Uzume-feed/src/` — Post creation, feed generation
  - `Monad/events/src/` — Event publishing/subscribing
  - `@tests/src/sandbox/` — Sandbox infrastructure

  **Acceptance Criteria**:
  - [ ] E2E content flow test created in `@tests/e2e/content.rs`
  - [ ] Test uses real sandbox infrastructure (postgres, NATS, DragonflyDB)
  - [ ] Test verifies complete flow: post → feed → like → comment → timeline
  - [ ] Test passes with real services

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Complete content flow succeeds
    Tool: Bash
    Steps: Run E2E content test with real sandbox services
    Expected: All steps complete successfully, post created and visible in feed
    Evidence: .sisyphus/evidence/task-26-content.txt

  Scenario: Content interaction updates timeline
    Tool: Bash
    Steps: Create post, like it, verify timeline updated
    Expected: Timeline contains post with updated engagement metrics
    Evidence: .sisyphus/evidence/task-26-timeline.txt
  ```

  **Commit**: YES | Message: `feat(tests): Create content creation & interaction E2E flow test` | Files: [@tests/e2e/content.rs]

- [ ] 27. Create cross-app flow

  **What to do**: Create E2E test for cross-app flow: Uzume-profiles → Uzume-feed → Uzume-stories, alias-based linking. Verify cross-app data consistency, alias resolution, privacy isolation. Use real sandbox infrastructure.

  **Must NOT do**: Do not modify production code. Do not use mocked services.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Cross-app E2E flow requires complex service orchestration
  - Skills: [] — E2E testing
  - Omitted: All - E2E-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 5 (with Tasks 25-26,28) | Blocks: [Final Verification] | Blocked By: [Wave 1, 2, 3, 4]

  **References**:
  - `apps/Uzume/Uzume-profiles/src/` — Profile services
  - `apps/Uzume/Uzume-feed/src/` — Feed services
  - `apps/Uzume/Uzume-stories/src/` — Story services
  - `Monad/Heka/src/alias.rs:1` — Alias system

  **Acceptance Criteria**:
  - [ ] E2E cross-app flow test created in `@tests/e2e/cross_app.rs`
  - [ ] Test uses real sandbox infrastructure
  - [ ] Test verifies alias-based linking across apps
  - [ ] Test verifies cross-app data consistency
  - [ ] Test passes with real services

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Cross-app alias resolution works
    Tool: Bash
    Steps: Create user in Uzume-profiles, resolve alias in Uzume-feed
    Expected: Alias resolves correctly, user data consistent
    Evidence: .sisyphus/evidence/task-27-cross-app.txt

  Scenario: Cross-app privacy isolation maintained
    Tool: Bash
    Steps: Create private profile in Uzume-profiles, attempt access from Uzume-feed without link
    Expected: Access denied, privacy maintained
    Evidence: .sisyphus/evidence/task-27-privacy.txt
  ```

  **Commit**: YES | Message: `feat(tests): Create cross-app E2E flow test with alias-based linking` | Files: [@tests/e2e/cross_app.rs]

- [ ] 28. Create security regression flows

  **What to do**: Create E2E security regression flow tests: attempted SQLi during signup → blocked, XSS in bio → sanitized, authz bypass → 403. Verify security measures work end-to-end with real sandbox infrastructure.

  **Must NOT do**: Do not modify production code. Do not create actual security vulnerabilities.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Security E2E flows require careful attack simulation
  - Skills: [] — Security testing
  - Omitted: All - security-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 5 (with Tasks 25-27) | Blocks: [Final Verification] | Blocked By: [Wave 1, 2, 3, 4]

  **References**:
  - `@tests/security/sql_injection.rs` — SQL injection tests
  - `@tests/security/xss.rs` — XSS tests
  - `@tests/security/authz_bypass.rs` — Authz bypass tests
  - `@tests/src/sandbox/` — Sandbox infrastructure

  **Acceptance Criteria**:
  - [ ] E2E security regression tests created in `@tests/e2e/security_regression.rs`
  - [ ] Tests cover SQLi, XSS, authz bypass scenarios
  - [ ] Tests verify security measures work end-to-end
  - [ ] All tests pass (expected: attacks blocked/sanitized)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: SQLi during signup blocked
    Tool: Bash
    Steps: Attempt registration with SQLi payload in email field
    Expected: Request returns 400/422, no data leakage, user not created
    Evidence: .sisyphus/evidence/task-28-sqli.txt

  Scenario: XSS in bio sanitized
    Tool: Bash
    Steps: Create profile with XSS payload in bio, retrieve profile
    Expected: Bio sanitized, no script execution
    Evidence: .sisyphus/evidence/task-28-xss.txt

  Scenario: Authz bypass blocked
    Tool: Bash
    Steps: Attempt to access another user's private profile with forged token
    Expected: Request returns 403 Forbidden
    Evidence: .sisyphus/evidence/task-28-authz.txt
  ```

  **Commit**: YES | Message: `feat(tests): Create security regression E2E flow tests` | Files: [@tests/e2e/security_regression.rs]

### Wave 6: CI TDD Integration (Tasks 29-32)

- [ ] 29. Update justfile with comprehensive test recipes

  **What to do**: Update justfile with comprehensive test recipes: test-unit, test-integration, test-e2e, test-security, test-property, test. Ensure all recipes work correctly and can be invoked independently.

  **Must NOT do**: Do not break existing justfile recipes. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: justfile updates are straightforward
  - Skills: [] — CI/CD
  - Omitted: All - CI-focused

  **Parallelization**: Can Run In Parallel: NO | Wave 6 | Blocks: [30,31,32] | Blocked By: [Wave 1, 2, 3, 4, 5]

  **References**:
  - `justfile:177` — Current test recipes
  - `justfile:1` — Just command format

  **Acceptance Criteria**:
  - [ ] `just test-unit` → runs unit tests
  - [ ] `just test-integration` → runs integration tests
  - [ ] `just test-e2e` → runs E2E tests
  - [ ] `just test-security` → runs security tests
  - [ ] `just test-property` → runs property tests
  - [ ] `just test` → runs all test categories

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All recipes defined
    Tool: Bash
    Steps: grep -E "test-unit|test-integration|test-e2e|test-security|test-property|test" justfile
    Expected: All recipes present
    Evidence: .sisyphus/evidence/task-29-recipes.txt

  Scenario: Recipes execute correctly
    Tool: Bash
    Steps: just test-unit; just test-security; just test-property
    Expected: Each recipe runs successfully
    Evidence: .sisyphus/evidence/task-29-execution.txt
  ```

  **Commit**: YES | Message: `feat(tests): Update justfile with comprehensive test recipes` | Files: [justfile]

- [ ] 30. Create CI workflow steps

  **What to do**: Create CI workflow steps in `.github/workflows/ci.yml` for: fmt, clippy, audit, deny, test, security, e2e, property. Ensure all steps run in correct order with proper failure handling.

  **Must NOT do**: Do not break existing CI workflow. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: CI workflow updates are straightforward
  - Skills: [] — CI/CD
  - Omitted: All - CI-focused

  **Parallelization**: Can Run In Parallel: NO | Wave 6 | Blocks: [31,32] | Blocked By: [29]

  **References**:
  - `.github/workflows/ci.yml:1` — Current CI workflow
  - `justfile:1` — Just command format

  **Acceptance Criteria**:
  - [ ] CI workflow includes fmt step
  - [ ] CI workflow includes clippy step
  - [ ] CI workflow includes audit step
  - [ ] CI workflow includes deny step
  - [ ] CI workflow includes test step
  - [ ] CI workflow includes security step
  - [ ] CI workflow includes e2e step
  - [ ] CI workflow includes property step

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: CI workflow includes all steps
    Tool: Bash
    Steps: grep -E "fmt|clippy|audit|deny|test|security|e2e|property" .github/workflows/ci.yml
    Expected: All steps present
    Evidence: .sisyphus/evidence/task-30-steps.txt

  Scenario: CI workflow runs successfully
    Tool: Bash
    Steps: just ci
    Expected: All CI steps pass
    Evidence: .sisyphus/evidence/task-30-run.txt
  ```

  **Commit**: YES | Message: `feat(tests): Create CI workflow steps for all test categories` | Files: [.github/workflows/ci.yml]

- [ ] 31. Add CI gate enforcement

  **What to do**: Add CI gate enforcement to fail fast on any step failure. Configure artifact collection for test results, security scan results, coverage reports. Ensure proper error reporting.

  **Must NOT do**: Do not break existing CI workflow. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: CI gate enforcement is straightforward
  - Skills: [] — CI/CD
  - Omitted: All - CI-focused

  **Parallelization**: Can Run In Parallel: NO | Wave 6 | Blocks: [32] | Blocked By: [30]

  **References**:
  - `.github/workflows/ci.yml:1` — Current CI workflow
  - `justfile:1` — Just command format

  **Acceptance Criteria**:
  - [ ] CI fails fast on any step failure
  - [ ] Artifact collection configured for test results
  - [ ] Artifact collection configured for security scan results
  - [ ] Artifact collection configured for coverage reports
  - [ ] Proper error reporting configured

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: CI fails fast on test failure
    Tool: Bash
    Steps: Introduce failing test, run CI
    Expected: CI fails immediately, error reported
    Evidence: .sisyphus/evidence/task-31-fail-fast.txt

  Scenario: Artifacts collected
    Tool: Bash
    Steps: Run CI, verify artifacts created
    Expected: Test results, security scan results, coverage reports collected
    Evidence: .sisyphus/evidence/task-31-artifacts.txt
  ```

  **Commit**: YES | Message: `feat(tests): Add CI gate enforcement with fail-fast and artifact collection` | Files: [.github/workflows/ci.yml]

- [ ] 32. Add test reporting

  **What to do**: Add test reporting to CI: JUnit XML output, coverage reports, security scan results. Configure reporting tools and artifact storage. Ensure reports are accessible and actionable.

  **Must NOT do**: Do not break existing CI workflow. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Test reporting is straightforward
  - Skills: [] — CI/CD
  - Omitted: All - CI-focused

  **Parallelization**: Can Run In Parallel: NO | Wave 6 | Blocks: [Final Verification] | Blocked By: [31]

  **References**:
  - `.github/workflows/ci.yml:1` — Current CI workflow
  - `justfile:1` — Just command format

  **Acceptance Criteria**:
  - [ ] JUnit XML output configured
  - [ ] Coverage reports generated
  - [ ] Security scan results reported
  - [ ] Reports accessible and actionable

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: JUnit XML output generated
    Tool: Bash
    Steps: Run tests with JUnit output, verify file created
    Expected: JUnit XML file created with test results
    Evidence: .sisyphus/evidence/task-32-junit.txt

  Scenario: Coverage report generated
    Tool: Bash
    Steps: Run tests with coverage, verify report created
    Expected: Coverage report created with coverage data
    Evidence: .sisyphus/evidence/task-32-coverage.txt
  ```

  **Commit**: YES | Message: `feat(tests): Add test reporting with JUnit XML, coverage, and security scan results` | Files: [.github/workflows/ci.yml]

### Wave 7: Documentation & Cleanup (Tasks 33-35)

- [ ] 33. Write comprehensive testing guide

  **What to do**: Write `@tests/README.md` with comprehensive testing guide. Cover: test organization, how to run tests, how to add new tests, test categories, sandbox infrastructure, security testing, E2E testing, property-based testing, fuzzing, CI integration.

  **Must NOT do**: Do not modify production code. Do not create vague documentation.

  **Recommended Agent Profile**:
  - Category: `writing` — Reason: Documentation requires clear, comprehensive writing
  - Skills: [] — Documentation
  - Omitted: All - documentation-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 7 (with Tasks 34-35) | Blocks: [Final Verification] | Blocked By: [Wave 1, 2, 3, 4, 5, 6]

  **References**:
  - `@tests/src/` — All test modules
  - `@tests/security/` — Security tests
  - `@tests/e2e/` — E2E tests
  - `@tests/fuzz/` — Fuzzing harness

  **Acceptance Criteria**:
  - [ ] `@tests/README.md` exists and comprehensive
  - [ ] Guide covers all test categories
  - [ ] Guide includes examples for each category
  - [ ] Guide explains sandbox infrastructure
  - [ ] Guide explains CI integration

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Testing guide exists and is comprehensive
    Tool: Bash
    Steps: ls @tests/README.md; wc -l @tests/README.md
    Expected: File exists, 100+ lines
    Evidence: .sisyphus/evidence/task-33-exists.txt

  Scenario: Guide covers all categories
    Tool: Bash
    Steps: grep -E "unit|integration|e2e|security|property|fuzzing|CI" @tests/README.md
    Expected: All categories mentioned
    Evidence: .sisyphus/evidence/task-33-coverage.txt
  ```

  **Commit**: YES | Message: `docs(tests): Write comprehensive testing guide` | Files: [@tests/README.md]

- [ ] 34. Write onboarding guide for contributors

  **What to do**: Write onboarding guide for contributors: how to add tests, how to run tests, how to debug failing tests, how to use sandbox infrastructure, how to add new test categories. Include examples and troubleshooting.

  **Must NOT do**: Do not modify production code. Do not create vague documentation.

  **Recommended Agent Profile**:
  - Category: `writing` — Reason: Documentation requires clear, comprehensive writing
  - Skills: [] — Documentation
  - Omitted: All - documentation-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 7 (with Tasks 33,35) | Blocks: [Final Verification] | Blocked By: [Wave 1, 2, 3, 4, 5, 6]

  **References**:
  - `@tests/README.md` — Main testing guide
  - `@tests/src/` — All test modules
  - `justfile:1` — Just command format

  **Acceptance Criteria**:
  - [ ] Onboarding guide exists in `@tests/ONBOARDING.md`
  - [ ] Guide explains how to add tests
  - [ ] Guide explains how to run tests
  - [ ] Guide explains how to debug failing tests
  - [ ] Guide includes examples and troubleshooting

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Onboarding guide exists and is comprehensive
    Tool: Bash
    Steps: ls @tests/ONBOARDING.md; wc -l @tests/ONBOARDING.md
    Expected: File exists, 50+ lines
    Evidence: .sisyphus/evidence/task-34-exists.txt

  Scenario: Guide includes examples
    Tool: Bash
    Steps: grep -E "example|how to|step" @tests/ONBOARDING.md
    Expected: Examples and step-by-step instructions present
    Evidence: .sisyphus/evidence/task-34-examples.txt
  ```

  **Commit**: YES | Message: `docs(tests): Write onboarding guide for contributors` | Files: [@tests/ONBOARDING.md]

- [ ] 35. Remove old test files after 7-day verification period

  **What to do**: After 7-day verification period (all tests passing in new location, CI green), remove all old test files from original locations. Verify no tests remain outside `@tests/`. Update any remaining references to old test locations.

  **Must NOT do**: Do not remove tests before 7-day verification. Do not modify production code.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Cleanup is straightforward
  - Skills: [] — Cleanup
  - Omitted: All - cleanup-focused

  **Parallelization**: Can Run In Parallel: YES | Wave 7 (with Tasks 33-34) | Blocks: [Final Verification] | Blocked By: [Wave 1, 2, 3, 4, 5, 6]

  **References**:
  - `apps/Uzume/*/tests/` — Old test locations
  - `Monad/*/tests/` — Old test locations

  **Acceptance Criteria**:
  - [ ] All old test files removed from original locations
  - [ ] No tests remain outside `@tests/`
  - [ ] All references to old test locations updated
  - [ ] CI still passes after cleanup

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All old tests removed
    Tool: Bash
    Steps: find . -path @tests -prune -o -name "*_test.rs" -print | grep -v @tests | wc -l
    Expected: 0 test files remain outside @tests/
    Evidence: .sisyphus/evidence/task-35-cleanup.txt

  Scenario: CI still passes after cleanup
    Tool: Bash
    Steps: just ci
    Expected: All CI steps pass
    Evidence: .sisyphus/evidence/task-35-ci.txt
  ```

  **Commit**: YES | Message: `chore(tests): Remove old test files after 7-day verification period` | Files: [apps/Uzume/*/tests/*, Monad/*/tests/*]

## Final Verification Wave (MANDATORY — after ALL implementation tasks)
> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback → fix → re-run → present again → wait for okay.
- [ ] F1. Plan Compliance Audit — oracle
- [ ] F2. Code Quality Review — unspecified-high
- [ ] F3. Real Manual QA — unspecified-high (+ Playwright if UI)
- [ ] F4. Scope Fidelity Check — deep

## Commit Strategy
- **Wave-level commits**: Each wave → one commit with descriptive message (e.g., "feat(tests): Wave 1 - foundation with unified test crate")
- **Never commit failing tests**: All tests must pass in wave before commit
- **Atomic commits**: No wave spans across multiple commits unless explicitly blocked by issues
- **Commit messages**: Follow conventional commits with `feat(tests):`, `fix(tests):`, `refactor(tests):` prefixes

## Success Criteria
- [ ] Unified `@tests/` crate builds successfully: `cargo build -p tests`
- [ ] All 33 tests migrated: `find @tests/ -name "*_test.rs" | wc -l` returns 33+
- [ ] Zero old tests remain: `find . -path @tests -prune -o -name "*_test.rs" -print | grep -v @tests | wc -l` returns 0
- [ ] No dependency cycles: `cargo tree --workspace --duplicate` shows no duplicates/cycles
- [ ] Security tests pass: `cargo nextest run --workspace security`
- [ ] E2E tests pass: `cargo nextest run --workspace e2e`
- [ ] CI passes all gates: `just ci` (fmt, clippy, audit, deny, test, security, e2e)
- [ ] Documentation complete: `@tests/README.md` exists and comprehensive
- [ ] Property tests operational: `cargo nextest run --workspace property`
- [ ] Snapshot tests operational: `cargo nextest run --workspace snapshot`
- [ ] Fuzzing harness ready: `cargo fuzz ...` works for critical paths
- [ ] Sandbox services start/stop cleanly: No orphaned Docker containers after test runs

