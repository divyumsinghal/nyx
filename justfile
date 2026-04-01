# =============================================================================
# justfile — Nyx task runner
# Install: cargo install just  (or: brew install just)
# Usage:   just <recipe>
#          just --list          (show all recipes)
# =============================================================================

# ── Development: infrastructure ───────────────────────────────────────────────

# Start all infrastructure services (postgres, dragonfly, nats, minio, kratos, …)
dev-infra:
    docker compose -f Prithvi/compose/infra.yml -f Prithvi/compose/dev.yml up -d
    @echo "Infra started. Mailhog UI: http://localhost:8025 | Grafana: http://localhost:3030 | MinIO: http://localhost:9001"

# Stop all infrastructure services
dev-infra-down:
    docker compose -f Prithvi/compose/infra.yml -f Prithvi/compose/dev.yml down

# Start platform workers (Heimdall gateway + Oya + Ushas)
dev-platform:
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/dev.yml \
        up -d heimdall oya ushas

# Start all 5 Uzume microservices
dev-uzume:
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        -f Prithvi/compose/dev.yml \
        up -d

# Start the full dev stack (infra + platform + Uzume)
dev-up: dev-infra
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        -f Prithvi/compose/dev.yml \
        up -d
    @echo "Full dev stack started."
    @echo "  Gateway:          http://localhost:3000"
    @echo "  Uzume-profiles:   http://localhost:3001"
    @echo "  Uzume-feed:       http://localhost:3002"
    @echo "  Uzume-stories:    http://localhost:3003"
    @echo "  Uzume-reels:      http://localhost:3004"
    @echo "  Uzume-discover:   http://localhost:3005"
    @echo "  Kratos (public):  http://localhost:4433"
    @echo "  Matrix:           http://localhost:8008"
    @echo "  Meilisearch:      http://localhost:7700"
    @echo "  Grafana:          http://localhost:3030"
    @echo "  Mailhog:          http://localhost:8025"

# Tear down the full dev stack (preserves volumes)
dev-down:
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        -f Prithvi/compose/dev.yml \
        down

# Tear down and wipe all volumes (full reset — DESTROYS ALL DATA)
dev-nuke:
    @echo "⚠ This will destroy all local data (postgres, minio, nats, etc.). Press Ctrl+C to cancel."
    @sleep 3
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        down -v --remove-orphans

# ── Development: individual service runners (cargo) ───────────────────────────

dev-gateway:
    cargo run -p heimdall
dev-oya:
    cargo run -p oya
dev-ushas:
    cargo run -p ushas
dev-profiles:
    cargo run -p uzume-profiles
dev-feed:
    cargo run -p uzume-feed
dev-stories:
    cargo run -p uzume-stories
dev-reels:
    cargo run -p uzume-reels
dev-discover:
    cargo run -p uzume-discover

# ── Production ────────────────────────────────────────────────────────────────

# Start the full production stack
prod-up:
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        -f Prithvi/compose/prod.yml \
        up -d

# Rolling restart of a specific service (e.g.: just prod-restart uzume-feed)
prod-restart service:
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        -f Prithvi/compose/prod.yml \
        restart {{service}}

# ── Database ──────────────────────────────────────────────────────────────────

# Run all pending migrations (Monad + Uzume schemas)
db-migrate:
    cargo run -p nyx-xtask -- migrate

# Drop all schemas and re-run migrations from scratch (DESTROYS DATA — dev only)
db-reset:
    cargo run -p nyx-xtask -- db-reset

# Load seed data (10 users + 20 posts) into a running dev database
seed:
    cargo run -p nyx-xtask -- seed

# Open a psql session to the nyx database
db-shell:
    docker exec -it nyx-postgres psql -U postgres -d nyx

# Open a psql session to the kratos database
db-shell-kratos:
    docker exec -it nyx-postgres psql -U postgres -d kratos

# ── NATS ─────────────────────────────────────────────────────────────────────

# Create/verify NATS JetStream streams (NYX + UZUME)
nats-setup:
    cargo run -p nyx-xtask -- nats-setup

# Open NATS CLI shell (requires nats CLI: cargo install nats-cli)
nats-shell:
    nats --server nats://nyx:${NYX_NATS_PASSWORD:-changeme_nats}@localhost:4222

# ── Quality gates (used by CI) ────────────────────────────────────────────────

fmt-check:
    cargo fmt --all -- --check
fmt:
    cargo fmt --all
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings
lint-fix:
    cargo clippy --workspace --all-targets --all-features --fix -- -D warnings

# ── Security ──────────────────────────────────────────────────────────────────

security-deny:
    cargo deny check
security-audit:
    cargo audit

security-secret-scan:
	@set -eu; \
	if git grep -nEI '(AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|-----BEGIN (RSA|OPENSSH|EC|DSA|PGP) PRIVATE KEY-----|xox[baprs]-[A-Za-z0-9-]{10,}|AIza[0-9A-Za-z\\-_]{35})' -- .; then \
	  echo 'Secret scan failed: potential credential material detected in tracked files.' >&2; \
	  exit 1; \
	else \
	  echo 'Secret scan passed: no high-confidence secret signatures detected.'; \
	fi

security:           security-deny security-audit security-secret-scan

# Privacy-isolation gate: verify cross-app boundary constraints exist in migrations
gate-cross-app-unauthorized:
	@set -eu; \
	file='migrations/Monad/0003_nyx_app_links.up.sql'; \
	grep -q "CHECK (source_app <> target_app)" "$$file" || { echo 'Missing cross-app boundary check: source_app <> target_app' >&2; exit 1; }; \
	grep -q "CHECK (source_nyx_identity_id <> target_nyx_identity_id)" "$$file" || { echo 'Missing self-link prevention check' >&2; exit 1; }; \
	grep -q "DEFAULT '{\"type\":\"revoked\"}'::jsonb" "$$file" || { echo 'Missing fail-closed default policy (revoked)' >&2; exit 1; }; \
	echo 'Cross-app unauthorized-access gate passed: required constraints present.'

gate-step1-compat:
	@set -eu; \
	bash tests/contracts/verify_step1_contract_lock.sh contracts/step1-compat.lock

check: security-deny

# ── Testing ───────────────────────────────────────────────────────────────────

# Run all tests (nextest if available, else cargo test)
test:
    @if cargo nextest --version >/dev/null 2>&1; then \
      cargo nextest run --workspace; \
    else \
      cargo test --workspace --all-targets; \
    fi

# Unit tests only (no integration tests — fast, no Docker required)
test-unit:
    @if cargo nextest --version >/dev/null 2>&1; then \
      cargo nextest run --workspace --lib; \
    else \
      cargo test --workspace --lib; \
    fi

# Integration tests only (requires Docker for testcontainers)
test-integration:
    @if cargo nextest --version >/dev/null 2>&1; then \
      cargo nextest run --workspace --test '*'; \
    else \
      cargo test --workspace --test '*'; \
    fi

# Run tests for a specific crate (e.g.: just test-crate uzume-feed)
test-crate crate:
    @if cargo nextest --version >/dev/null 2>&1; then \
      cargo nextest run -p {{crate}}; \
    else \
      cargo test -p {{crate}}; \
    fi

# ── Migrations validation gate ────────────────────────────────────────────────

migration-check:
    cargo run -p nyx-xtask -- migrate
validation-check:
    cargo check --workspace --all-targets --all-features

# ── Full CI-equivalent local gate ─────────────────────────────────────────────

# Mirrors exactly what .github/workflows/ci.yml runs — run this before pushing.
ci:
    fmt-check
    lint
    security
    gate-cross-app-unauthorized
    validation-check
    test
    @echo "All CI gates passed locally."

# ── Docker builds ─────────────────────────────────────────────────────────────

# Validate all compose files parse correctly (no-op dry run)
compose-validate:
    docker compose -f Prithvi/compose/infra.yml config --quiet
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        config --quiet
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        config --quiet
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        -f Prithvi/compose/dev.yml \
        config --quiet
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        -f Prithvi/compose/prod.yml \
        config --quiet
    @echo "All compose files are valid."

# Build a service Docker image (e.g.: just docker-build uzume-profiles)
docker-build binary:
    docker build \
        --build-arg BINARY_NAME={{binary}} \
        -f Prithvi/docker/Dockerfile.service \
        -t ghcr.io/divyumsinghal/nyx/{{binary}}:latest \
        .

# Build Oya (media worker — includes FFmpeg)
docker-build-oya:
    docker build \
        --build-arg BINARY_NAME=oya \
        --target runtime-ffmpeg \
        -f Prithvi/docker/Dockerfile.worker \
        -t ghcr.io/divyumsinghal/nyx/oya:latest \
        .

# Build Ushas (notify worker — distroless)
docker-build-ushas:
    docker build \
        --build-arg BINARY_NAME=ushas \
        --target runtime-distroless \
        -f Prithvi/docker/Dockerfile.worker \
        -t ghcr.io/divyumsinghal/nyx/ushas:latest \
        .

# Build a web frontend image (e.g.: just docker-build-web Uzume-web)
docker-build-web app:
    docker build \
        --build-arg APP_DIR={{app}} \
        --build-arg PUBLIC_API_BASE_URL=http://localhost:3000/api \
        -f Prithvi/docker/Dockerfile.web \
        -t ghcr.io/divyumsinghal/nyx/{{app}}:latest \
        .

# Build all service images
docker-build-all:
    just docker-build uzume-profiles
    just docker-build uzume-feed
    just docker-build uzume-stories
    just docker-build uzume-reels
    just docker-build uzume-discover
    just docker-build heimdall
    just docker-build-oya
    just docker-build-ushas

# ── Build ─────────────────────────────────────────────────────────────────────

build-release:
    cargo build --release --workspace
build-check:
    cargo check --workspace --all-targets --all-features

# ── Frontend ──────────────────────────────────────────────────────────────────

ui-install:
    cd Maya && pnpm install
ui-dev-uzume:
    cd Maya/Uzume-web && pnpm dev
ui-dev-nyx:
    cd Maya/nyx-web && pnpm dev
ui-build:
    cd Maya && pnpm build
ui-lint:
    cd Maya && pnpm lint

# ── Logs ──────────────────────────────────────────────────────────────────────

# Tail logs for a specific service (e.g.: just logs uzume-feed)
logs service:
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        logs -f --tail=100 {{service}}

# Tail all logs
logs-all:
    docker compose \
        -f Prithvi/compose/infra.yml \
        -f Prithvi/compose/platform.yml \
        -f Prithvi/compose/uzume.yml \
        logs -f --tail=50

# ── Scaffold ──────────────────────────────────────────────────────────────────

# Scaffold a new Nyx app (e.g.: just new-app Anteros)
new-app app:
    cargo run -p nyx-xtask -- new-app {{app}}
