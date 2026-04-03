# Unified Tests Onboarding

This guide helps contributors add, run, and debug tests in the root tests crate.

## 1) Quick Start

From repository root:

```bash
just test
```

For focused categories:

```bash
just test-security
just test-property
just test-integration
just test-e2e
```

## 2) Where to Add a Test

- Integration behavior: `tests/tests/integration/`
- Security regressions: `tests/tests/security/`
- Property-based checks: `tests/tests/property/`
- E2E flows: `tests/tests/e2e/`
- Shared test helpers: `tests/src/`

Then register the file from the category index file:

- `tests/tests/integration.rs`
- `tests/tests/security.rs`
- `tests/tests/property.rs`
- `tests/tests/e2e.rs`

## 3) Writing Meaningful Tests

Use this checklist:

- Use real behavior assertions, not smoke-only checks.
- Include at least one positive and one negative vector.
- Prefer explicit setup and explicit expected outcomes.
- Add table/parameterized cases for parser or validation logic.
- Add property-based strategies for broad input spaces.

## 4) Security Test Expectations

Security suites should include attack vectors and safe controls:

- SQL injection payloads and no-leak behavior.
- XSS payloads and output sanitization behavior.
- Authorization bypass attempts and proper denial behavior.
- Privacy leakage checks (PII, identity mapping, cross-app alias leakage).
- Boundary/edge cases (oversized, malformed, unicode, null-byte).

Run with:

```bash
just test-security
```

## 5) E2E Sandbox Tests

E2E tests use real sandboxed services.

- PostgreSQL
- Redis/Dragonfly-compatible cache
- MinIO
- NATS JetStream (when enabled)

If Docker is unavailable, `just test-e2e` skips intentionally.

## 6) Snapshot and Property Tests

Snapshot tests:

```bash
just test-snapshot
```

Property tests:

```bash
just test-property
```

## 7) Fuzzing

Fuzz targets live under `tests/fuzz/`.

Build fuzz targets:

```bash
cargo fuzz build fuzz_target_token_validation
```

Run for a short smoke interval:

```bash
cargo fuzz run fuzz_target_token_validation -- -max_total_time=5
```

## 8) Debugging Failures

1. Re-run only the failing category.
2. Re-run a single test binary with `cargo test -p tests --test <name>`.
3. Enable logs with `RUST_LOG=debug`.
4. For Docker-backed failures, check socket and image pull access.

## 9) CI Expectations

Before opening a PR:

```bash
just test-security
just test-property
just test
```

If you touched sandbox or e2e paths, also run:

```bash
just test-e2e
```

## 10) Contributor Workflow Example

1. Add a new security test file under `tests/tests/security/`.
2. Register it in `tests/tests/security.rs`.
3. Run `just test-security`.
4. Run `just test`.
5. Update docs if test behavior introduces a new test pattern.
