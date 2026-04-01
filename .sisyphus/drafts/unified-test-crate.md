# Draft: Unified Test Crate & Comprehensive TDD Strategy

## Requirements (confirmed)
- **Location**: Unified test crate at `@tests/` (workspace root)
- **Adoption**: Full Seshat/TESTING.md dependency stack (all 22 crates)
- **Scope**: Full-blown testing infrastructure:
  - Component tests
  - End-to-end (E2E) tests
  - Integration tests
  - Cross-component tests
  - Security tests
  - TDD enforcement
- **Migration**: Move all existing tests into the unified @tests/ harness

## Technical Decisions (from user)
- Test crate location: `@tests/` (workspace root, next to crates/)
- Dependency set: Full Seshat/TESTING.md (proptest, criterion, assert_cmd, predicates, insta, rstest, test-case, serial_test, mockall, loom, quickcheck, arbitrary, libfuzzer-sys, tracing, tracing-subscriber, anyhow, thiserror, tempfile, pretty_assertions)
- Testing philosophy: Go hard - comprehensive coverage with multiple testing strategies

## Research Findings
### Current State
- **No unified test crate**: 33 integration test files scattered across crates
- **Only shared utility**: `Nun::testing` module (test_id, test_config, assertion helpers)
- **Missing**: Property-based testing, mocking frameworks, fuzzing, snapshot testing
- **Current execution**: `just test` → `cargo nextest run --workspace` or fallback to `cargo test`
- **Reverse dependency risk**: `Heka` depends on `uzume_profiles` in dev-deps (cycles)

### Existing Test Patterns
- **Unit tests**: Inline `#[cfg(test)]` modules
- **Integration tests**: Real infrastructure via testcontainers (PostgreSQL, NATS)
- **Hand-rolled mocks**: e.g., `TestAuthenticator` in Heka
- **Axum `ServiceExt::oneshot`**: HTTP router testing without network
- **Unsafe env vars**: `unsafe { std::env::set_var(...) }` pattern (fragile for parallel execution)

### Seshat TESTING.md Guidance
- **Property-based**: proptest, quickcheck, arbitrary
- **Snapshot**: insta
- **Parameterized**: rstest, test-case
- **Fuzzing**: libfuzzer-sys
- **Concurrency**: loom
- **Mocking**: mockall
- **CLI**: assert_cmd, predicates
- **Observability**: tracing, tracing-subscriber
- **Errors**: anyhow, thiserror
- **Utilities**: tempfile, pretty_assertions

## Decisions Made (User Answers)
- **TDD enforcement**: CI-gating for PRs to main - enforce passing tests plus all TESTING.md criteria (security, logic, etc.). Not just green tests - full validation.
- **Test migration**: Big bang - migrate all 33 integration test files at once to @tests/
- **Security testing**: Full security audit including penetration testing (SQL injection, XSS, authz bypass, etc.)
- **E2E infrastructure**: Real infrastructure only - use testcontainers with actual postgres, NATS, and all services
- **Backward compatibility**: Keep old crate-specific tests during migration, delete after verification

## Scope Boundaries
### INCLUDE
- Creating `@tests/` as a unified test crate with full Seshat/TESTING.md dependencies
- Migrating all existing integration tests into `@tests/`
- Establishing TDD workflow enforcement
- Component tests, E2E tests, integration tests, cross-component tests
- Security testing (full security audit including penetration testing: SQL injection, XSS, authz bypass, etc.)
- Property-based testing, snapshot testing, fuzzing, mocking
- CI integration to run unified test suite

### EXCLUDE
- Changes to runtime dependencies (only dev-dependencies in tests/)
- Production code refactoring unless directly related to testability
- Creating production crates (only test harness at @tests/)
- Test infrastructure not covered by Seshat/TESTING.md (e.g., custom test runners unless needed)

## Architecture Decision Record (ADR)
- **Decision**: Place unified test crate at `@tests/` in workspace root
- **Rationale**: Discoverability, workspace-level orchestration, follows monorepo patterns
- **Alternatives considered**: 
  1. `crates/nyx-tests` - rejected because tests are not production crates
  2. Per-crate test-utils - rejected (duplicates, harder to maintain)
- **Consequences**: All crates will have `tests/` as a dev-dependency; test discovery becomes workspace-level
