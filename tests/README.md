# Unified Test Crate

This crate centralizes workspace testing for Nyx and Uzume.

## Goals

- Keep cross-crate integration tests in one place.
- Provide reusable fixtures, mocks, generators, and security payloads.
- Run security and E2E sandbox tests from one crate.
- Enable CI-friendly test gate recipes.

## Structure

- src/lib.rs: shared test harness exports
- src/common.rs: tracing and common helpers
- src/fixtures.rs: deterministic fixture data
- src/mocks.rs: mock server utilities
- src/generators.rs: property-based generators
- src/asserts.rs: custom assertions
- src/security.rs: security payload sets
- src/sandbox.rs: Docker-based sandbox infrastructure
- tests/integration: cross-component and boundary tests
- tests/security: security and privacy tests
- tests/property: property-based tests
- tests/e2e: sandbox-backed end-to-end smoke tests
- tests/migrated: mirrored legacy test files (33 files) for big-bang consolidation

## Commands

From repository root:

```bash
just test
just test-unit
just test-integration
just test-security
just test-property
just test-e2e
just test-all
```

Run only unified crate tests:

```bash
cargo test -p tests
cargo nextest run -p tests
```

## Migration Notes

- Existing crate-local tests are being migrated into this crate in a non-destructive phase.
- A full mirror of 33 legacy test files now exists under `tests/tests/migrated/**`.
- Once unified coverage is verified stable, legacy copies can be removed.
