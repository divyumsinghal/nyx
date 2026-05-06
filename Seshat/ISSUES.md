# Nyx Auth Flow — Security & Architecture Audit

> Audit date: 2026-04-04
> Scope: Nyx + Uzume account creation, auth, login — everything that touches Heimdall, Heka, Kratos, xtask.

---

## CRITICAL — Must fix before any use

### C-001 · Password collected but never sent to Kratos
**File:** `Monad/xtask/src/commands/create_account.rs`
The registration flow collects a password, runs it through zxcvbn strength scoring and the HaveIBeenPwned API, then submits via `method: "code"` (passwordless OTP). The password string is silently discarded — it is never included in any HTTP payload sent to Kratos. Users create accounts with zero password credential set.

**Risk:** Users believe they have a password. `login.rs` uses `method: "password"` which fails for every account created by `create_account.rs`. The entire create-then-login flow is broken end-to-end.

**Fix:** After OTP verification creates the account, use the returned `session_token` to call the Kratos settings flow and set the password.

---

### C-002 · Login/registration method mismatch — flow completely broken
**Files:** `Monad/xtask/src/commands/create_account.rs`, `Monad/xtask/src/commands/login.rs`
Registration uses `method: "code"` (OTP, no password stored). Login uses `method: "password"` (requires a password credential). Every account created by xtask will fail every xtask login attempt.

**Fix:** Covered by C-001 fix (set password via settings after OTP registration).

---

### C-003 · Hardcoded test secrets in kratos.yml (production config)
**File:** `Prithvi/config/kratos/kratos.yml:235-236`
```yaml
secrets:
  cookie:
    - "test-cookie-secret-exactly-32byt"
  cipher:
    - "test-cipher-secret-exactly-32byt"
```
These are literal test strings in the **main** configuration file. Any deployment mounting `kratos.yml` uses these known secrets. Kratos session cookies and CSRF tokens are signed with `test-cookie-secret-exactly-32byt` — publicly known.

**Risk:** Anyone can forge Kratos session cookies and CSRF tokens for any account. CVSSv3: 9.8 Critical.

**Fix:** Replace with `${KRATOS_COOKIE_SECRET}` and `${KRATOS_CIPHER_SECRET}` env vars.

---

### C-004 · Hardcoded database password in kratos.yml DSN
**File:** `Prithvi/config/kratos/kratos.yml:41`
```yaml
dsn: "postgres://kratos_app:changeme_kratos_db@postgres:5432/kratos?sslmode=disable"
```
The `kratos_app` database password is embedded in cleartext in a committed config file.

**Risk:** Repository read access = database credential access.

**Fix:** `postgres://kratos_app:${KRATOS_DB_PASSWORD}@${POSTGRES_HOST:-postgres}:5432/kratos?...`

---

### C-005 · Missing token exchange — Kratos sessions cannot reach protected APIs
**Files:** `Monad/Heimdall/src/routes.rs`, `Monad/xtask/src/commands/login.rs`
Kratos returns opaque session tokens (`ory_st_xxxxx`). Heimdall's `auth_middleware` validates `Authorization: Bearer <JWT>`. There is no endpoint to exchange a Kratos session token for a Nyx JWT. After successful login or registration, no protected API endpoint (`/api/uzume/*`, `/api/nyx/account/*`) can be called.

**Risk:** The entire authenticated API surface is permanently unreachable. Login effectively does nothing.

**Fix:** Add `POST /api/nyx/auth/token` to Heimdall. It validates the Kratos session via `/sessions/whoami` and issues a signed Nyx JWT. Update xtask login to call this endpoint.

---

## HIGH — Serious vulnerabilities

### H-001 · JWT algorithm confusion attack
**File:** `Monad/Heimdall/src/jwt.rs:118`
```rust
(Algorithm::HS256, _) => decode_with_validation(...)  // accepted regardless of RS256 config
```
`decode_jwt` accepts HS256 tokens even when an RSA public key is configured (RS256 mode). An attacker with the RSA public key (which is often public) can sign an HS256 token using the public key bytes as the HMAC secret. Heimdall accepts it as valid.

**Risk:** Token forgery in RS256-configured production deployments. CVSSv3: 9.1 Critical.

**Fix:** When `public_key_pem` is `Some`, reject all non-RS256 tokens unconditionally.

---

### H-002 · Unbounded request body — memory exhaustion DoS
**File:** `Monad/Heimdall/src/proxy.rs:146`
```rust
axum::body::to_bytes(req.into_body(), usize::MAX).await
```
No limit on request body size. One malformed request with a multi-gigabyte body exhausts Heimdall's memory.

**Risk:** DoS via a single unauthenticated HTTP request.

**Fix:** Cap at 10 MiB: `axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024)`.

---

### H-003 · HIBP failure permanently blocks registration (fail-closed)
**File:** `Monad/xtask/src/commands/create_account.rs:67-70`
```rust
Err(err) => {
    eprintln!("Breached-password check unavailable: {err}");
    continue;  // loops forever while HIBP is down
}
```
If HaveIBeenPwned is unreachable, registration loops endlessly. Users cannot create accounts.

**Risk:** Availability — registration depends on a third-party service.

**Fix:** Warn and proceed (`break value`) when HIBP is unreachable. The check is advisory.

---

### H-004 · Client-side login retries bypass Kratos brute-force protection
**File:** `Monad/xtask/src/commands/login.rs:25-47`
The xtask retries login 5 times, reusing the same flow ID. Each retry is a new auth failure that Kratos records but the client bypasses the UX friction Kratos intends by immediately retrying.

**Risk:** Weakened brute-force signal; audit logs show 5 failures from a single typo.

**Fix:** Remove client retry loop. One attempt per invocation.

---

### H-005 · No `.env.example` — required secrets undocumented
No template exists for environment variables. Developers must reverse-engineer:
`POSTGRES_PASSWORD`, `NYX_APP_DB_PASSWORD`, `NYX_MIGRATION_DB_PASSWORD`, `KRATOS_DB_PASSWORD`, `JWT_SECRET`, `CORS_ALLOWED_ORIGINS`, `KRATOS_COOKIE_SECRET`, `KRATOS_CIPHER_SECRET`, `HEIMDALL_URL`.

**Risk:** Weak/guessable secrets; secrets hardcoded in history.

**Fix:** Create `.env.example` with every required variable and safe placeholder values.

---

## MEDIUM — Should fix soon

### M-001 · Regex compiled on every `validate_nyx_id` call
**File:** `Monad/Heka/src/nyx_id.rs:261`
```rust
let pattern = regex::Regex::new(NYX_ID_PATTERN).unwrap();
```
Called on every ID check (on the request path, hits DB). Compiles regex + heap-allocates on every invocation.

**Fix:** `static NYX_ID_RE: OnceLock<Regex>` — compile once.

---

### M-002 · Nyx ID max length inconsistency between Rust and Kratos schema
**Files:** `Monad/Heka/src/nyx_id.rs:33`, `Prithvi/config/kratos/identity.schema.json:33`
- Rust: `NYX_ID_MAX_LENGTH = 32`
- JSON schema: `"maxLength": 30`

Also, the schema allows dots (`\.`) but the Rust regex (`^[a-zA-Z0-9_]+$`) rejects them.

A 31-character nyx_id passes local validation then fails Kratos with an opaque 422.

**Fix:** Align to 32 in schema. Remove dots from schema pattern to match Rust.

---

### M-003 · `expect()` panic on UUID parse from database
**File:** `Monad/Heka/src/nyx_id.rs:209`
```rust
id_str.parse().expect("Stored identity ID should be valid UUID")
```
Data corruption or a migration bug causes a panic, potentially crashing the async task.

**Fix:** Return a proper error via `.context(...).transpose()`.

---

### M-004 · Duplicate JWT implementations with divergent behaviour
**Files:** `Monad/Heka/src/jwt.rs`, `Monad/Heimdall/src/jwt.rs`
- **Heka**: HS256-only, has `app: NyxApp` claim, no `jti`, no RS256 — tokens cannot be revoked
- **Heimdall**: HS256 + RS256, has `jti` for revocation

Heka's jwt.rs is unreferenced in the current auth request path (Heimdall issues all JWTs via the new token exchange). It should be removed to prevent accidental use.

---

### M-005 · ~~`NYX_INSECURE_TLS` defaults to `false` — breaks all localhost testing~~ — RESOLVED
Resolved by removing `NYX_INSECURE_TLS` entirely. xtask now loads Caddy's local CA cert
via `CADDY_CA_CERT` env var (a PEM file path), which `account-create` / `account-login`
extract from the running Caddy container before invoking cargo. No cert bypass, no
`danger_accept_invalid_certs`. Proper chain of trust through Caddy's built-in CA.

---

### M-006 · No password confirmation prompt
**File:** `Monad/xtask/src/commands/create_account.rs`
Single password entry with no confirmation. A typo locks the user out.

**Fix:** Prompt twice, compare, loop until match.

---

### M-007 · Session lifespan 720 hours (30 days)
**File:** `Prithvi/config/kratos/kratos.yml:222`
Stolen `nyx_session` cookie is valid for 30 days. Recommend 24–72 hours with inactivity extend.

---

### M-008 · `sslmode=disable` in all PostgreSQL DSNs
**Files:** `kratos.yml:41`, `kratos.test.yml:14`
Acceptable for single-host Docker dev. Must be `sslmode=require` in any multi-host or cloud deployment.

---

### M-009 · `/healthz` leaks internal service topology
**File:** `Monad/Heimdall/src/health.rs`
Unauthenticated `/healthz` returns names and reachability of every internal service. Exposes architecture to reconnaissance.

**Recommendation:** Return only aggregate status publicly; put per-service detail behind authentication.

---

### M-010 · Magic-link method enabled unnecessarily
**File:** `Prithvi/config/kratos/kratos.yml:113`
Magic links are clickable, forwardable, and phishable. OTP codes (`method: code`) are safer. `link` method should be disabled.

---

## LOW — Address when convenient

### L-001 · JWT ID uses UUIDv4 instead of platform-standard UUIDv7
**File:** `Monad/Heimdall/src/jwt.rs:73` — `Uuid::new_v4()` for `jti`.

### L-002 · Auth-test Heimdall builds from full `rust:1.94` image
**File:** `Prithvi/compose/auth-test.yml:117` — ~1.5 GB pull + full recompile on every `auth-up`.

### L-003 · Kratos CORS mixes HTTP localhost with HTTPS production origins
**File:** `Prithvi/config/kratos/kratos.yml:52-55` — split by environment.

### L-004 · No xtask command for Uzume app-scoped alias creation
After Nyx account creation, the Uzume profile (`uzume_xxx` alias) must be created separately. No xtask command exists for this step.

### L-005 · Auth rate limit key collision
**File:** `Monad/Heimdall/src/rate_limit.rs:152` — all `/api/nyx/auth/*` paths share one bucket `{ip}:auth`.

### L-006 · Password not zeroed from memory after use
Plain `String` allocations for passwords are not zeroed on drop. Use `secrecy` or `zeroize`.

---

## Summary

| ID    | Severity | Fixed | Description                                           |
|-------|----------|-------|-------------------------------------------------------|
| C-001 | Critical | ✓     | Password discarded — never sent to Kratos             |
| C-002 | Critical | ✓     | Registration OTP + login password incompatible        |
| C-003 | Critical | ✓     | Hardcoded cookie/cipher secrets in kratos.yml         |
| C-004 | Critical | ✓     | Hardcoded DB password in kratos.yml DSN               |
| C-005 | Critical | ✓     | No Kratos→JWT token exchange endpoint                 |
| H-001 | High     | ✓     | JWT algorithm confusion (HS256 in RS256 mode)         |
| H-002 | High     | ✓     | Unbounded proxy body → memory exhaustion DoS          |
| H-003 | High     | ✓     | HIBP outage blocks registration                       |
| H-004 | High     | ✓     | 5 client-side retries bypass Kratos throttle          |
| H-005 | High     | ✓     | No .env.example                                       |
| M-001 | Medium   | ✓     | Regex compiled per call in validate_format            |
| M-002 | Medium   | ✓     | nyx_id maxLength 30 (schema) vs 32 (Rust)             |
| M-003 | Medium   | ✓     | expect() panic on malformed UUID in DB                |
| M-004 | Medium   | note  | Duplicate JWT implementations in Heka/Heimdall        |
| M-005 | Medium   | ✓     | ~~NYX_INSECURE_TLS~~ removed; Caddy CA loaded properly|
| M-006 | Medium   | ✓     | No password confirmation prompt                       |
| M-007 | Medium   | note  | 30-day session lifetime                               |
| M-008 | Medium   | note  | sslmode=disable in all DSNs                           |
| M-009 | Medium   | note  | /healthz leaks internal topology                      |
| M-010 | Medium   | note  | Magic-link method enabled                             |
| L-001 | Low      | note  | JTI uses UUIDv4 not UUIDv7                            |
| L-002 | Low      | note  | auth-test builds Heimdall from source (slow)          |
| L-003 | Low      | note  | Mixed HTTP/HTTPS origins in Kratos CORS               |
| L-004 | Low      | note  | No xtask command for Uzume alias creation             |
| L-005 | Low      | note  | Auth rate limit bucket shared across all auth paths   |
| L-006 | Low      | note  | Password not zeroed from memory                       |
