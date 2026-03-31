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
security-secret-scan:
	@set -eu; \
	if git grep -nEI '(AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|-----BEGIN (RSA|OPENSSH|EC|DSA|PGP) PRIVATE KEY-----|xox[baprs]-[A-Za-z0-9-]{10,}|AIza[0-9A-Za-z\\-_]{35})' -- .; then \
	  echo 'Secret scan failed: potential credential material detected in tracked files.' >&2; \
	  exit 1; \
	else \
	  echo 'Secret scan passed: no high-confidence secret signatures detected.'; \
	fi
security:           security-deny security-audit security-secret-scan

gate-cross-app-unauthorized:
	@set -eu; \
	file='migrations/Monad/0003_nyx_app_links.up.sql'; \
	grep -q "CHECK (source_app <> target_app)" "$file" || { echo 'Missing cross-app boundary check: source_app <> target_app' >&2; exit 1; }; \
	grep -q "CHECK (source_nyx_identity_id <> target_nyx_identity_id)" "$file" || { echo 'Missing self-link prevention check' >&2; exit 1; }; \
	grep -q "DEFAULT '{\"type\":\"revoked\"}'::jsonb" "$file" || { echo 'Missing fail-closed default policy (revoked)' >&2; exit 1; }; \
	echo 'Cross-app unauthorized-access gate passed: required constraints present.'
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
ci:                 fmt-check lint security gate-cross-app-unauthorized migration-check validation-check test

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
