## .github/

```
.github/
├── workflows/
│   ├── ci.yml                 # PR checks: lint, test, build
│   ├── release.yml            # Tag-triggered: build Docker images, push to GHCR
│   └── deploy.yml             # Post-release: SSH deploy to Oracle Cloud
├── CODEOWNERS
├── PULL_REQUEST_TEMPLATE.md
└── ISSUE_TEMPLATE/
    ├── bug_report.yml
    └── feature_request.yml
```

## Workflows

### workflows/ci.yml

Runs on every PR and push to `main`. Steps:

1. **Rust checks**: `cargo fmt --check`, `cargo clippy`, `cargo deny check`, `cargo nextest run`
2. **Frontend checks**: `pnpm lint`, `pnpm typecheck`, `pnpm build`
3. **Change detection**: Uses `dorny/paths-filter` to determine which crates changed. Only runs tests for affected crates + their dependents (Cargo workspace makes this deterministic via the dependency graph).
4. **Integration tests**: Spins up PostgreSQL, DragonflyDB, NATS, Meilisearch via `testcontainers` in Rust tests. No external service required.

**Tool choice — CI test runner:**
- **cargo-nextest (chosen)**: 3-6x faster than `cargo test` due to parallel execution per-test (not per-crate). Better output formatting, retries, JUnit XML for CI reporting.
- Alternative — cargo test: Built-in, zero setup. Slower because tests within a crate run serially by default.

```yaml
- name: Check formatting
  run: just fmt-check

- name: Run Clippy
  run: just lint

- name: Check dependencies
  run: just check
```

### workflows/release.yml

Triggered by Git tags (`v*`). Builds multi-platform Docker images (amd64 + arm64) and pushes to GitHub Container Registry (`ghcr.io/nyx-Monad/*`). Images use multi-stage builds: Rust builder → `gcr.io/distroless/cc` runtime (no shell, minimal attack surface).

### workflows/deploy.yml

Post-release: SSHes into Oracle Cloud VMs, pulls new images, runs `docker compose up -d` with zero-downtime rolling strategy.

---