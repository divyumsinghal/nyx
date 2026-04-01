# Unified Test Crate

This crate is the single test source of truth for the Nyx monorepo.

## Purpose

- Keep all Rust tests centralized under the root tests crate.
- Enforce strong security and privacy coverage.
- Run integration and E2E validation against real sandboxed infrastructure.
- Support TDD and CI gating with deterministic entry points.

## Test Categories

- Unit: pure helpers and utility behavior in `src/**`.
- Integration: cross-crate and service boundary behavior in `tests/integration/**`.
- Security: SQLi, XSS, authz, privacy, and edge-case regressions in `tests/security/**`.
- Property: property-based strategies in `tests/property/**`.
- E2E: sandbox-backed flows in `tests/e2e/**`.
- Migrated mirror: historical copies under `tests/migrated/**` for auditability.

## Folder Layout

- `src/lib.rs`: unified test harness re-exports.
- `src/common.rs`: tracing and generic helpers.
- `src/fixtures.rs`: deterministic fixture builders.
- `src/mocks.rs`: HTTP/mock service behavior.
- `src/generators.rs`: proptest generators.
- `src/asserts.rs`: assertion helpers.
- `src/security.rs`: security payload corpora and privacy checks.
- `src/sandbox.rs`: containerized service bootstrap helpers.
- `tests/integration.rs`: integration test index.
- `tests/security.rs`: security test index.
- `tests/property.rs`: property test index.
- `tests/e2e.rs`: e2e test index.

## Tooling Coverage

The test crate actively uses the TESTING.md stack across test and benchmark targets:

- `proptest`, `quickcheck`, `arbitrary`: property/fuzz-style input generation.
- `criterion`: benchmark harness in `benches/`.
- `assert_cmd`, `predicates`: command-level assertions.
- `insta`: snapshot assertions.
- `rstest`, `test-case`: parameterized test patterns.
- `serial_test`: serialized execution for shared resources.
- `mockall`: trait-based mock contracts.
- `loom`: deterministic concurrency modeling.
- `libfuzzer-sys`: fuzz target entry points under `fuzz/`.
- `tracing`, `tracing-subscriber`: trace-aware test diagnostics.
- `tempfile`, `pretty_assertions`, `anyhow`, `thiserror`: ergonomic test support.

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
just test-snapshot
just fuzz-check
```

Crate-only runs:

```bash
cargo test -p tests
cargo nextest run -p tests
```

## E2E Infrastructure

E2E tests are designed for real services in sandbox containers.

- PostgreSQL: schema and query-level behavior.
- Redis/Dragonfly protocol: cache and session behavior.
- MinIO: object storage integration behavior.
- NATS JetStream: async event transport checks.

When Docker socket is unavailable, `just test-e2e` skips intentionally rather than failing local dev flows.

## Security Posture

Security suites are explicit and separated for auditability:

- `sql_injection.rs`
- `xss.rs`
- `authz_bypass.rs`
- `privacy_violations.rs`
- `edge_cases.rs`
- `feed_security.rs`
- `payload_sweep.rs`

Every suite is expected to assert concrete attack vectors and fail-closed behavior.

## CI Expectations

CI should run quality and test gates with artifacts:

- format and lint checks
- security checks
- unified test gates
- JUnit test output
- coverage output

The root tests crate must remain green independently of legacy crate-level test folders.

## Migration Policy

- Legacy crate-level tests were migrated and removed from app/platform crates.
- Root tests crate now owns active execution.
- `tests/tests/migrated/**` remains available as historical reference during hardening.

## Writing New Tests

- Place new tests in the category that matches behavior intent.
- Prefer real data and real service contracts over synthetic no-op assertions.
- Use parameterized and property-based coverage for input-heavy logic.
- For security tests, include both blocked vectors and safe control cases.

## Troubleshooting

- If a security suite fails, isolate with `just test-security`.
- If an E2E suite fails locally, verify Docker availability first.
- If snapshots fail, inspect changes before updating snapshots.
- If concurrency tests fail, run targeted suites with single-thread execution.
