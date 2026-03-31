# Development
dev-infra:          docker compose -f Prithvi/compose/infra.yml up -d
dev-ogma:           cargo run -p ogma
dev-aengus:         cargo run -p aengus
dev-themis:         cargo run -p themis
dev-gateway:        cargo run -p Heimdall

# Quality gates (used by CI)
fmt-check:          cargo fmt --all -- --check
lint:               cargo clippy --workspace --all-targets --all-features -- -D warnings

# Security
security-deny:      cargo deny check
security-audit:     cargo audit
security:           security-deny security-audit
check:              security-deny

# Testing (nextest with deterministic fallback)
test:
    @if cargo nextest --version >/dev/null 2>&1; then \
      cargo nextest run --workspace; \
    else \
      cargo test --workspace --all-targets; \
    fi
test-unit:
    @if cargo nextest --version >/dev/null 2>&1; then \
      cargo nextest run --workspace --lib; \
    else \
      cargo test --workspace --lib; \
    fi
test-integration:
    @if cargo nextest --version >/dev/null 2>&1; then \
      cargo nextest run --workspace --test '*'; \
    else \
      cargo test --workspace --test '*'; \
    fi

# Migration + validation gates
migration-check:    cargo run -p nyx-xtask -- migrate
validation-check:   cargo check --workspace --all-targets --all-features

# Full CI-equivalent local gate
ci:                 fmt-check lint security migration-check validation-check test

# Database
db-migrate:         cargo run -p nyx-xtask -- migrate
db-reset:           cargo run -p nyx-xtask -- db-reset

# Build
build-release:      cargo build --release --workspace
docker-build:       ./tools/docker-build.sh

# Frontend
ui-dev:             cd clients && pnpm dev
ui-build:           cd clients && pnpm build
ui-lint:            cd clients && pnpm lint
