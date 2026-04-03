#!/usr/bin/env bash
# =============================================================================
# test-auth.sh — Run the Nyx auth integration tests against a live stack
#
# Usage:
#   ./tools/scripts/test-auth.sh            # start stack, test, tear down
#   ./tools/scripts/test-auth.sh --keep     # keep stack running after tests
#   ./tools/scripts/test-auth.sh --no-start # assume stack is already up
#   ./tools/scripts/test-auth.sh --filter email_password  # run specific tests
#
# Manual Google OAuth test instructions are printed at the end.
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

COMPOSE_FILE="${ROOT}/Prithvi/compose/auth-test.yml"

KEEP=false
NO_START=false
FILTER=""

for arg in "$@"; do
  case "$arg" in
    --keep)     KEEP=true ;;
    --no-start) NO_START=true ;;
    --filter)   shift; FILTER="$1" ;;
    --filter=*) FILTER="${arg#--filter=}" ;;
  esac
done

# ── Colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GRN='\033[0;32m'
YLW='\033[1;33m'
BLU='\033[0;34m'
RST='\033[0m'

info()    { echo -e "${BLU}▶ $*${RST}"; }
success() { echo -e "${GRN}✔ $*${RST}"; }
warn()    { echo -e "${YLW}⚠ $*${RST}"; }
error()   { echo -e "${RED}✗ $*${RST}" >&2; }

# ── Tear-down trap ────────────────────────────────────────────────────────────
STACK_STARTED=false
cleanup() {
  if $STACK_STARTED && ! $KEEP; then
    info "Tearing down auth stack..."
    docker compose -f "$COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
    success "Stack removed."
  elif $KEEP; then
    warn "Stack left running (--keep). Stop it with:"
    echo "  docker compose -f Prithvi/compose/auth-test.yml down -v"
  fi
}
trap cleanup EXIT

# ── Start stack ───────────────────────────────────────────────────────────────
if ! $NO_START; then
  info "Starting auth stack (Postgres + Mailpit + Kratos)..."
  docker compose -f "$COMPOSE_FILE" pull --quiet 2>/dev/null || true
  docker compose -f "$COMPOSE_FILE" up -d --wait
  STACK_STARTED=true
  success "Auth stack is healthy."
else
  info "Skipping stack start (--no-start)."
fi

# ── Verify services are reachable ─────────────────────────────────────────────
info "Verifying service connectivity..."

if ! curl -sf http://localhost:4433/health/ready >/dev/null; then
  error "Kratos is not reachable at http://localhost:4433"
  error "Make sure the auth stack is running:"
  error "  docker compose -f Prithvi/compose/auth-test.yml up -d --wait"
  exit 1
fi

if ! curl -sf http://localhost:8025/api/v1/info >/dev/null; then
  error "Mailpit is not reachable at http://localhost:8025"
  exit 1
fi

success "All services reachable."

# ── Build + run tests ─────────────────────────────────────────────────────────
info "Building auth integration tests..."
cd "$ROOT"
cargo test --test auth_integration --no-run

info "Running auth integration tests..."
export KRATOS_PUBLIC_URL="http://localhost:4433"
export KRATOS_ADMIN_URL="http://localhost:4434"
export MAILPIT_API_URL="http://localhost:8025"
export RUST_LOG="info"
export RUST_BACKTRACE="1"

TEST_ARGS="--test-threads=4 --nocapture"
if [ -n "$FILTER" ]; then
  TEST_ARGS="$TEST_ARGS $FILTER"
fi

cargo test --test auth_integration -- $TEST_ARGS

success "All auth integration tests passed!"

# ── Manual Google OAuth instructions ─────────────────────────────────────────
echo ""
echo -e "${YLW}════════════════════════════════════════════════════════════════${RST}"
echo -e "${YLW}  MANUAL: Google OAuth2 Test (not automated — requires browser)${RST}"
echo -e "${YLW}════════════════════════════════════════════════════════════════${RST}"
echo ""
echo "To test Google sign-in manually:"
echo ""
echo "  1. Create a Google OAuth2 project:"
echo "     https://console.cloud.google.com/apis/credentials"
echo "     → Create credentials → OAuth 2.0 Client ID → Web application"
echo "     → Authorised redirect URIs: http://localhost:4433/self-service/methods/oidc/callback/google"
echo ""
echo "  2. Set environment variables:"
echo "     export GOOGLE_CLIENT_ID=<your-client-id>"
echo "     export GOOGLE_CLIENT_SECRET=<your-client-secret>"
echo ""
echo "  3. Restart Kratos with the new credentials:"
echo "     docker compose -f Prithvi/compose/auth-test.yml restart kratos"
echo ""
echo "  4. Initiate a registration flow and follow the OIDC URL:"
echo "     FLOW=\$(curl -s http://localhost:4433/self-service/registration/api | jq -r .id)"
echo "     GOOGLE_URL=\$(curl -s \"http://localhost:4433/self-service/registration?flow=\$FLOW\" \\"
echo "       -H 'Content-Type: application/json' \\"
echo "       -d '{\"method\":\"oidc\",\"provider\":\"google\"}' | jq -r '.redirect_browser_to')"
echo "     echo \"Open in browser: \$GOOGLE_URL\""
echo ""
echo "  5. Complete Google consent in browser. After redirect, Kratos will:"
echo "     a. Map your verified Google email to the 'email' trait"
echo "     b. Return a 422 asking you to choose a nyx_id (handle)"
echo "     c. Submit your chosen nyx_id to complete registration"
echo ""
echo "  6. Verify in the Kratos admin API:"
echo "     curl -s http://localhost:4434/admin/identities | jq '.[].traits'"
echo ""
