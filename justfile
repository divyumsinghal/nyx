# Development
dev-infra:          docker compose -f Prithvi/compose/infra.yml up -d
dev-ogma:           cargo run -p ogma
dev-aengus:         cargo run -p aengus
dev-themis:         cargo run -p themis
dev-gateway:        cargo run -p Heimdall

# Testing
test:               cargo nextest run --workspace
test-unit:          cargo nextest run --workspace --lib
test-integration:   cargo nextest run --workspace --test '*'
lint:               cargo clippy --workspace --all-targets --all-features -- -D warnings
fmt-check:          cargo fmt --all -- --check
check:              cargo deny check

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