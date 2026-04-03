# =============================================================================
# justfile — Nyx task runner
# Install: cargo install just
# Usage:   just <recipe>  |  just --list
# =============================================================================

set shell := ["C:\\Program Files\\Git\\bin\\bash.exe", "-cu"]

# ── Compose stacks ─────────────────────────────────────────────────────────────
# To add a new app: add its overlay file here. Everything below picks it up.

_f_infra      := "Prithvi/compose/infra.yml"
_f_platform   := "Prithvi/compose/platform.yml"
_f_uzume      := "Prithvi/compose/uzume.yml"
_f_dev        := "Prithvi/compose/dev.yml"
_f_build      := "Prithvi/compose/build.yml"   # local source-build overlay (dev only)
_f_prod       := "Prithvi/compose/prod.yml"
_f_auth_test  := "Prithvi/compose/auth-test.yml"

_dc_infra     := "docker compose -f " + _f_infra
_dc_plat      := _dc_infra  + " -f " + _f_platform
_dc_uzume     := _dc_plat   + " -f " + _f_uzume
_dc_dev       := _dc_uzume  + " -f " + _f_dev
_dc_local     := _dc_dev    + " -f " + _f_build  # _dc_dev + build directives
_dc_prod      := _dc_uzume  + " -f " + _f_prod
_dc_auth_test := "docker compose -f " + _f_auth_test

# ── Auth-test env (exported to cargo test) ───────────────────────────────────
_auth_env := "KRATOS_PUBLIC_URL=http://localhost:4433 KRATOS_ADMIN_URL=http://localhost:4434 MAILPIT_API_URL=http://localhost:8025"

# ── Default ───────────────────────────────────────────────────────────────────

default:
    @just --list

# ── Toolchain ─────────────────────────────────────────────────────────────────

# Install required cargo extras (call once after cloning)
install-tools:
    cargo install cargo-nextest cargo-deny cargo-audit

# ── Bootstrap (run once, or after nuke) ───────────────────────────────────────

# Full one-time environment bootstrap: validate → build check →
# start infra → wait → migrate → NATS streams.
# To load development fixture data afterwards: just seed
setup:
    @just compose-validate
    @just build-check
    @just dev-infra
    @just _wait-postgres
    @just db-migrate
    @just nats-setup
    @echo ""
    @echo "Bootstrap complete."
    @echo "  Run 'just start'  — launch the full dev stack"
    @echo "  Run 'just seed'   — load development fixture data"

# ── Frontend ──────────────────────────────────────────────────────────────────

# Start Uzume social app dev server (http://localhost:8081)
web:
    cd Maya/uzume-web && pnpm dev

# Start Nyx account portal dev server (http://localhost:8082)
web-nyx:
    cd Maya/nyx-web && pnpm dev

# Start both frontends in parallel via Turbo
web-all:
    pnpm exec turbo run dev --parallel

# ── Runtime ───────────────────────────────────────────────────────────────────

# Start the full local dev stack (builds images on first run, reuses on subsequent)
start:
    {{_dc_local}} up -d
    @just _print-urls

# Force-rebuild all service images from source and restart
# Use after code changes or when images may be stale
rebuild:
    {{_dc_local}} up -d --build
    @just _print-urls

# Force-rebuild a single service (e.g.: just rebuild-service uzume-feed)
rebuild-service service:
    {{_dc_local}} up -d --build {{service}}

# Stop everything (volumes preserved)
stop:
    {{_dc_local}} down

# Restart a specific service without rebuilding (e.g.: just restart uzume-feed)
restart service:
    {{_dc_local}} restart {{service}}

# Hard-reset: stop + destroy ALL data volumes. Requires re-running setup.
nuke:
    @echo "WARNING: All local data will be destroyed. Ctrl+C within 5s to cancel."
    @sleep 5
    {{_dc_local}} down -v --remove-orphans

# ── Subsystem start/stop ──────────────────────────────────────────────────────

# Start infrastructure services only (postgres, dragonfly, nats, minio, kratos, …)
dev-infra:
    {{_dc_infra}} up -d

# Stop infrastructure services
dev-infra-down:
    {{_dc_infra}} down

# Start platform workers only (Heimdall, Oya, Ushas)
dev-platform:
    {{_dc_local}} up -d heimdall oya ushas

# Start Uzume microservices only
dev-uzume:
    {{_dc_local}} up -d uzume-profiles uzume-feed uzume-stories uzume-reels uzume-discover

# ── Database ──────────────────────────────────────────────────────────────────

# Run all pending migrations (Monad + Uzume schemas)
db-migrate:
    cargo run -p nyx-xtask -- migrate

# Drop all schemas and re-run from scratch (DESTRUCTIVE — dev only)
db-reset:
    cargo run -p nyx-xtask -- db-reset

# Interactive psql session — nyx database
db-shell:
    docker exec -it nyx-postgres psql -U postgres -d nyx

# Interactive psql session — kratos database
db-shell-kratos:
    docker exec -it nyx-postgres psql -U postgres -d kratos

# Load development fixture data: 10 users + 20 posts + sample content.
# Safe to re-run (idempotent insert-or-ignore).
seed:
    cargo run -p nyx-xtask -- seed

# ── NATS ─────────────────────────────────────────────────────────────────────

# Create / verify JetStream streams (NYX + UZUME)
nats-setup:
    cargo run -p nyx-xtask -- nats-setup

# Interactive NATS CLI shell (requires: brew install nats-io/nats-tools/nats)
nats-shell:
    nats --server nats://nyx:${NYX_NATS_PASSWORD:-changeme_nats}@localhost:4222

# ── Build ─────────────────────────────────────────────────────────────────────

# Check the full workspace compiles (fast — no codegen, no linking)
build-check:
    cargo check --workspace --all-targets --all-features

# Build release binaries for all crates
build:
    cargo build --release --workspace

# ── Code quality ──────────────────────────────────────────────────────────────

# Format all Rust source
fmt:
    cargo fmt --all

# Check formatting without modifying files (CI gate)
fmt-check:
    cargo fmt --all -- --check

# Run Clippy — all targets, all features, warnings as errors
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Apply Clippy auto-fixes where possible
lint-fix:
    cargo clippy --workspace --all-targets --all-features --fix -- -D warnings

# ── Security ──────────────────────────────────────────────────────────────────

# Check licenses and banned crates (requires: just install-tools)
security-deny:
    cargo deny check

# Check for known CVEs (requires: just install-tools)
security-audit:
    cargo audit

# Scan tracked files for credential material
security-secret-scan:
    @set -eu; \
    if git grep -nEI \
        '(AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|-----BEGIN (RSA|OPENSSH|EC|DSA|PGP) PRIVATE KEY-----|xox[baprs]-[A-Za-z0-9-]{10,}|AIza[0-9A-Za-z\\-_]{35})' \
        -- . ':(exclude).env.example'; then \
        echo "Secret scan FAILED: potential credential material in tracked files." >&2; \
        exit 1; \
    else \
        echo "Secret scan passed."; \
    fi

# Run all security checks
security: security-deny security-audit security-secret-scan

# ── Privacy gates ─────────────────────────────────────────────────────────────

# Verify cross-app boundary constraints are present in migrations
gate-cross-app:
    @set -eu; \
    file='migrations/Monad/0003_nyx_app_links.up.sql'; \
    grep -q "CHECK (source_app <> target_app)" "$$file" \
        || { echo "MISSING: cross-app boundary check (source_app <> target_app)" >&2; exit 1; }; \
    grep -q "CHECK (source_nyx_identity_id <> target_nyx_identity_id)" "$$file" \
        || { echo "MISSING: self-link prevention check" >&2; exit 1; }; \
    grep -q "DEFAULT '{\"type\":\"revoked\"}'::jsonb" "$$file" \
        || { echo "MISSING: fail-closed default policy (revoked)" >&2; exit 1; }; \
    echo "Cross-app privacy gate passed."

# Verify Step-1 API contract lock
gate-step1-compat:
    @bash tests/contracts/verify_step1_contract_lock.sh contracts/step1-compat.lock

# ── Auth integration tests ────────────────────────────────────────────────────

# Start the minimal auth stack (postgres + mailpit + kratos) and wait for health
auth-up:
    {{_dc_auth_test}} up -d --wait

# Stop the auth stack and remove volumes
auth-down:
    {{_dc_auth_test}} down -v --remove-orphans

# Run auth integration tests (auth stack must already be up)
auth-test-run:
    {{_auth_env}} cargo test --test auth_integration -- --test-threads=4 --nocapture

# Full auth test cycle: start → test → stop
auth-test:
    @just auth-up
    {{_auth_env}} cargo test --test auth_integration -- --test-threads=4 --nocapture || \
        (just auth-logs && just auth-down && exit 1)
    @just auth-down

# Rebuild the auth integration test binary without running (compile check)
auth-build:
    cargo test --test auth_integration --no-run

# Show logs for the auth stack
auth-logs:
    {{_dc_auth_test}} logs --tail=80

# Show auth stack container status
auth-ps:
    {{_dc_auth_test}} ps

# ── Tests ─────────────────────────────────────────────────────────────────────
# All test recipes require: just install-tools

# Run the full test suite (unit + integration, no live stack required)
test:
    cargo nextest run --workspace

# Unit tests only — no Docker, no I/O (fast)
test-unit:
    cargo nextest run --workspace --lib

# Integration tests only — requires Docker (testcontainers)
test-integration:
    cargo nextest run --workspace --test '*'

# Run all security-focused tests
test-security:
    cargo nextest run --workspace --test security

# Run property-based tests
test-property:
    cargo nextest run --workspace --test property

# Run e2e tests
test-e2e:
    cargo nextest run --workspace --test e2e

# Run tests for a single crate (e.g.: just test-crate uzume-feed)
test-crate crate:
    cargo nextest run -p {{crate}}

# ── Full CI gate ──────────────────────────────────────────────────────────────

# Alias for gate-cross-app used by CI workflow
gate-cross-app-unauthorized: gate-cross-app

# Mirrors CI exactly. Run before pushing.
ci:
    just fmt-check
    just lint
    just security
    just gate-cross-app
    just gate-step1-compat
    just build-check
    just test
    @echo ""
    @echo "All CI gates passed."

# ── Compose validation ────────────────────────────────────────────────────────

# Parse-validate all compose file combinations
compose-validate:
    {{_dc_infra}}  config --quiet
    {{_dc_plat}}   config --quiet
    {{_dc_uzume}}  config --quiet
    {{_dc_dev}}    config --quiet
    {{_dc_local}}  config --quiet
    {{_dc_prod}}   config --quiet
    @echo "All compose files valid."

# ── Docker image builds ───────────────────────────────────────────────────────

# Build a service image (e.g.: just docker-build uzume-profiles)
docker-build binary:
    docker build \
        --build-arg BINARY_NAME={{binary}} \
        -f Prithvi/docker/Dockerfile.service \
        -t ghcr.io/divyumsinghal/nyx/{{binary}}:latest \
        .

# Build Oya worker image (includes FFmpeg layer)
docker-build-oya:
    docker build \
        --build-arg BINARY_NAME=oya \
        --target runtime-ffmpeg \
        -f Prithvi/docker/Dockerfile.worker \
        -t ghcr.io/divyumsinghal/nyx/oya:latest \
        .

# Build Ushas worker image (distroless runtime)
docker-build-ushas:
    docker build \
        --build-arg BINARY_NAME=ushas \
        --target runtime-distroless \
        -f Prithvi/docker/Dockerfile.worker \
        -t ghcr.io/divyumsinghal/nyx/ushas:latest \
        .

# Build all service images
docker-build-all:
    just docker-build heimdall
    just docker-build uzume-profiles
    just docker-build uzume-feed
    just docker-build uzume-stories
    just docker-build uzume-reels
    just docker-build uzume-discover
    just docker-build-oya
    just docker-build-ushas

# ── Logs ──────────────────────────────────────────────────────────────────────

# Tail logs for a specific service (e.g.: just logs uzume-feed)
logs service:
    {{_dc_local}} logs -f --tail=100 {{service}}

# Tail all service logs
logs-all:
    {{_dc_local}} logs -f --tail=50

# ── Production ────────────────────────────────────────────────────────────────

# Start production stack
prod-up:
    {{_dc_prod}} up -d

# Stop production stack
prod-down:
    {{_dc_prod}} down

# Rolling restart of a production service (e.g.: just prod-restart uzume-feed)
prod-restart service:
    {{_dc_prod}} restart {{service}}

# ── Scaffold ──────────────────────────────────────────────────────────────────

# Scaffold a new Nyx app (e.g.: just new-app Anteros)
new-app app:
    cargo run -p nyx-xtask -- new-app {{app}}

# Create a new Nyx account interactively (requires: just auth-up)
account-create:
    cargo run -p nyx-xtask -- create-account

# Login to an existing Nyx account interactively (requires: just auth-up)
account-login:
    cargo run -p nyx-xtask -- login

# ── Private helpers ───────────────────────────────────────────────────────────

# Print service URLs
_print-urls:
    @echo ""
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
    @echo "  Mailpit:          http://localhost:8025"
    @echo "  MinIO console:    http://localhost:9001"

# Block until postgres inside its container reports ready (max 120s)
_wait-postgres:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Waiting for postgres to be healthy..."
    timeout=120
    elapsed=0
    until docker exec nyx-postgres pg_isready -U postgres -q 2>/dev/null; do
        if [ "$elapsed" -ge "$timeout" ]; then
            echo "ERROR: postgres failed to become healthy within ${timeout}s." >&2
            echo "Run: docker logs nyx-postgres" >&2
            exit 1
        fi
        sleep 3
        elapsed=$((elapsed + 3))
    done
    echo "postgres is ready."