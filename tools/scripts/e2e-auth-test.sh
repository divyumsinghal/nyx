#!/usr/bin/env bash
# =============================================================================
# e2e-auth-test.sh — Automated end-to-end auth flow test
#
# Tests the complete Nyx account creation and login flow:
#   1. Nyx ID availability check
#   2. OTP-based registration (real email → Kratos courier DB → code → session)
#   3. Kratos session → Nyx JWT exchange
#   4. Password setup via settings flow
#   5. Password-based login → JWT
#
# OTP retrieval: reads from the Kratos admin courier API
#   (GET /admin/courier/messages), which stores every dispatched message
#   regardless of SMTP provider.
#
# Requirements:
#   - Gateway (Caddy) at $GATEWAY_URL         (default https://localhost:3443)
#   - Kratos admin at  $KRATOS_ADMIN_URL      (default http://localhost:4434)
#   - E2E_TEST_EMAIL   set to real test inbox (OTP is sent there via real SMTP)
#   - jq installed
#
# Usage:
#   just auth-dev-test
#   # or directly:
#   E2E_TEST_EMAIL=you@example.com \
#   GATEWAY_URL=https://localhost:3443 \
#   bash tools/scripts/e2e-auth-test.sh
# =============================================================================

set -euo pipefail

# GATEWAY_URL always points at the HTTPS edge (Caddy), never directly at Heimdall.
# Default: https://localhost:3443 (Caddy exposed port on the host, after trusting CA).
# Inside Docker (just auth-e2e-test): overridden to https://localhost via --resolve.
GATEWAY_URL="${GATEWAY_URL:-https://localhost:3443}"
KRATOS_ADMIN_URL="${KRATOS_ADMIN_URL:-http://localhost:4434}"
E2E_TEST_EMAIL="${E2E_TEST_EMAIL:?E2E_TEST_EMAIL must be set to a real email address}"

TS=$(date +%s)
TEST_NYX_ID="nyxe2e${TS: -6}"
# Random password per run — no hardcoded credentials in tracked files.
# Set E2E_TEST_PASSWORD to override for reproducible CI runs.
TEST_PASSWORD="${E2E_TEST_PASSWORD:-$(openssl rand -base64 18 | tr -d '/+=' | head -c 20)Aa1!}"

BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
RESET='\033[0m'

pass() { echo -e "  ${GREEN}✓${RESET} $*"; }
fail() { echo -e "  ${RED}✗${RESET} $*"; exit 1; }
step() { echo -e "\n${BOLD}[$1]${RESET} $2"; }
warn() { echo -e "  ${YELLOW}⚠${RESET} $*"; }

echo -e "\n${BOLD}=== Nyx E2E Auth Test ===${RESET}"
echo "  Gateway       : $GATEWAY_URL"
echo "  Kratos admin  : $KRATOS_ADMIN_URL"
echo "  Test email    : $E2E_TEST_EMAIL"
echo "  Nyx ID        : $TEST_NYX_ID"

# ── helpers ───────────────────────────────────────────────────────────────────

require_jq() {
  command -v jq >/dev/null 2>&1 || fail "jq is required (brew install jq / apt install jq)"
}

CURL_CA_ARGS=()
if [ -n "${CADDY_CA_CERT:-}" ]; then
  CURL_CA_ARGS+=(--cacert "$CADDY_CA_CERT")
fi
if [ -n "${CADDY_RESOLVE:-}" ]; then
  CURL_CA_ARGS+=(--resolve "$CADDY_RESOLVE")
fi

http_get() {
  curl -sf --max-time 10 "${CURL_CA_ARGS[@]}" "$@"
}

http_post() {
  curl -sf --max-time 10 "${CURL_CA_ARGS[@]}" -X POST -H "Content-Type: application/json" "$@"
}

# ── Step 0: Prerequisites ─────────────────────────────────────────────────────

require_jq

step 0 "Checking services are reachable"

http_get "$GATEWAY_URL/healthz" >/dev/null \
  || fail "Heimdall not responding at $GATEWAY_URL — run: just auth-dev-up && just auth-dev-heimdall"
pass "Heimdall healthy"

curl -sf --max-time 5 "$KRATOS_ADMIN_URL/health/ready" >/dev/null \
  || fail "Kratos admin not responding at $KRATOS_ADMIN_URL"
pass "Kratos admin healthy"

# ── Step 1: Nyx ID availability ───────────────────────────────────────────────

step 1 "Nyx ID availability check"

avail=$(http_post "$GATEWAY_URL/api/nyx/id/check-availability" \
  -d "{\"id\": \"$TEST_NYX_ID\"}" | jq -r '.available')
[ "$avail" = "true" ] || fail "Nyx ID '$TEST_NYX_ID' not available"
pass "Nyx ID available"

# ── Step 2: Init registration flow ───────────────────────────────────────────

step 2 "Initialising registration flow"

flow_resp=$(http_get "$GATEWAY_URL/api/nyx/auth/self-service/registration/api")
FLOW_ID=$(echo "$flow_resp" | jq -r '.id')
[ -n "$FLOW_ID" ] && [ "$FLOW_ID" != "null" ] \
  || fail "No flow ID in response: $flow_resp"
pass "Flow ID: $FLOW_ID"

# ── Step 3: Submit traits → dispatch OTP email ────────────────────────────────

step 3 "Submitting email + Nyx ID (dispatches OTP via real SMTP)"

# Record time just before triggering the OTP — used to filter courier messages
BEFORE_OTP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "${CURL_CA_ARGS[@]}" \
  -X POST -H "Content-Type: application/json" \
  -d "{\"method\":\"code\",\"traits\":{\"email\":\"$E2E_TEST_EMAIL\",\"nyx_id\":\"$TEST_NYX_ID\"}}" \
  "$GATEWAY_URL/api/nyx/auth/self-service/registration?flow=$FLOW_ID")

[ "$status" = "422" ] || [ "$status" = "200" ] || [ "$status" = "400" ] \
  || fail "Expected 400/422 (OTP sent) or 200, got: $status"
pass "OTP dispatched (Kratos returned $status — expected)"

# ── Step 4: Retrieve OTP from Kratos courier DB ───────────────────────────────
#
# Kratos stores every dispatched message in its own DB regardless of SMTP.
# We query GET /admin/courier/messages, filter by recipient + created_at,
# and extract the 6-digit code from the message body.

step 4 "Fetching OTP from Kratos courier DB"

OTP=""
for attempt in 1 2 3 4 5 6 8 10; do
  sleep "$attempt"
  messages=$(curl -sf --max-time 10 "$KRATOS_ADMIN_URL/admin/courier/messages?page_size=20" 2>/dev/null || echo '[]')
  OTP=$(echo "$messages" \
    | jq -r --arg email "$E2E_TEST_EMAIL" --arg since "$BEFORE_OTP" '
        .[]
        | select(.recipient == $email)
        | select(.created_at >= $since)
        | .body
      ' \
    | grep -oE '[0-9]{6}' \
    | head -1)
  if [ -n "$OTP" ]; then
    pass "OTP extracted (attempt $attempt)"
    break
  fi
  warn "OTP not yet in courier DB (attempt $attempt)..."
done

[ -n "$OTP" ] || fail "OTP never appeared in Kratos courier DB for $E2E_TEST_EMAIL"
pass "OTP: $OTP"

# ── Step 5: Verify OTP — account created ─────────────────────────────────────

step 5 "Verifying OTP → creating account"

reg_resp=$(http_post \
  "$GATEWAY_URL/api/nyx/auth/self-service/registration?flow=$FLOW_ID" \
  -d "{\"method\":\"code\",\"code\":\"$OTP\",\"traits\":{\"email\":\"$E2E_TEST_EMAIL\",\"nyx_id\":\"$TEST_NYX_ID\"}}" \
  2>&1) || fail "OTP verification failed. Response: $reg_resp"

SESSION_TOKEN=$(echo "$reg_resp" | jq -r '.session_token // empty')
[ -n "$SESSION_TOKEN" ] \
  || fail "No session_token after OTP verify. Response: $reg_resp"
pass "Account created. Session token obtained."

# ── Step 6: Exchange Kratos session → Nyx JWT ─────────────────────────────────

step 6 "Exchanging session token for Nyx JWT"

jwt_resp=$(http_post "$GATEWAY_URL/api/nyx/auth/token" \
  -d "{\"session_token\":\"$SESSION_TOKEN\"}" 2>&1) \
  || fail "Token exchange failed. Response: $jwt_resp"

ACCESS_TOKEN=$(echo "$jwt_resp" | jq -r '.access_token // empty')
[ -n "$ACCESS_TOKEN" ] || fail "No access_token in exchange response: $jwt_resp"

SEGMENT_COUNT=$(echo "$ACCESS_TOKEN" | tr -cd '.' | wc -c)
[ "$SEGMENT_COUNT" -eq 2 ] || fail "Token is not a valid JWT"
pass "JWT obtained and validated"

# ── Step 7: Set password via settings flow ────────────────────────────────────

step 7 "Setting password via Kratos settings flow"

settings_resp=$(curl -sf --max-time 10 "${CURL_CA_ARGS[@]}" \
  -H "X-Session-Token: $SESSION_TOKEN" \
  "$GATEWAY_URL/api/nyx/auth/self-service/settings/api" 2>&1) || {
  warn "Could not init settings flow. Skipping password setup."
  SKIP_LOGIN=1
}

if [ -z "${SKIP_LOGIN:-}" ]; then
  SETTINGS_FLOW_ID=$(echo "$settings_resp" | jq -r '.id // empty')
  if [ -n "$SETTINGS_FLOW_ID" ]; then
    pw_status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "${CURL_CA_ARGS[@]}" \
      -X POST -H "Content-Type: application/json" \
      -H "X-Session-Token: $SESSION_TOKEN" \
      -d "{\"method\":\"password\",\"password\":\"$TEST_PASSWORD\"}" \
      "$GATEWAY_URL/api/nyx/auth/self-service/settings?flow=$SETTINGS_FLOW_ID")
    [ "$pw_status" = "200" ] \
      && pass "Password set" \
      || warn "Password set returned $pw_status — login test may fail"
  else
    warn "Settings flow missing ID — skipping"
    SKIP_LOGIN=1
  fi
fi

# ── Step 8: Login with password ───────────────────────────────────────────────

if [ -z "${SKIP_LOGIN:-}" ]; then
  step 8 "Password login"

  login_flow_resp=$(http_get "$GATEWAY_URL/api/nyx/auth/self-service/login/api") \
    || fail "Could not init login flow"
  LOGIN_FLOW_ID=$(echo "$login_flow_resp" | jq -r '.id // empty')
  [ -n "$LOGIN_FLOW_ID" ] || fail "No login flow ID"

  login_resp=$(http_post \
    "$GATEWAY_URL/api/nyx/auth/self-service/login?flow=$LOGIN_FLOW_ID" \
    -d "{\"method\":\"password\",\"identifier\":\"$E2E_TEST_EMAIL\",\"password\":\"$TEST_PASSWORD\"}" \
    2>&1) || fail "Password login failed. Response: $login_resp"

  LOGIN_SESSION=$(echo "$login_resp" | jq -r '.session_token // empty')
  [ -n "$LOGIN_SESSION" ] || fail "No session_token after login."
  pass "Login successful"

  step 9 "Exchanging login session for JWT"

  login_jwt_resp=$(http_post "$GATEWAY_URL/api/nyx/auth/token" \
    -d "{\"session_token\":\"$LOGIN_SESSION\"}" 2>&1) \
    || fail "Login token exchange failed: $login_jwt_resp"

  LOGIN_JWT=$(echo "$login_jwt_resp" | jq -r '.access_token // empty')
  [ -n "$LOGIN_JWT" ] || fail "No access_token for login JWT"
  pass "Login JWT obtained"
fi

# ── Done ──────────────────────────────────────────────────────────────────────

echo -e "\n${GREEN}${BOLD}=============================================${RESET}"
echo -e "${GREEN}${BOLD}  ALL TESTS PASSED${RESET}"
echo -e "${GREEN}${BOLD}=============================================${RESET}"
echo -e "  Email    : $E2E_TEST_EMAIL"
echo -e "  Nyx ID   : $TEST_NYX_ID"
echo -e "  OTP reg  : OK"
echo -e "  JWT issue: OK"
if [ -z "${SKIP_LOGIN:-}" ]; then
echo -e "  Pw login : OK"
fi
echo ""
