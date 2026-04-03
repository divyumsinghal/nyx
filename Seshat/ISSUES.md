Ignore these two:

Secret management is still environment-variable based, not Vault/managed secret store with rotation workflow.
Evidence: config.rs

End-to-end TLS is not fully cryptographic hop-by-hop. Edge TLS exists via Caddy, but internal hops in compose remain HTTP.
Evidence: main.rs, auth-test.yml

YOU ARE STUCK IN A BAD LOOP, YOU ARE NOT ALLOWED TO RUN TERMINAL COMMANDS ANYWMORE JUST FIX ISSUES PLAN AND FIX EVERY ONE OF THESE ISSUES, YOU CANNOT USE TERMINAL COMMANDS AT ALL:

Dont hardcode stuff, YOU ARE LITERALLY RESPONISIBLE FOR FIXING THOSE KIND OF ISSUES, DONT CREATE THEM, USE PARALLEL AGENTS, **READ DOCS** YOU HAVE BEEN STUCK FOR TOO LONG, FIX FIRST TEST LATER:

Continue with this, use the multiple agents skill to dispack parallel agents for seperate fixes: You are a principal engineer at Meta, bought in to make this production ready.

Right now I am working on the Nyx+ Uzume User account creation, auth, etc workflow. But right now it has become too cluttered and I feel like there are lot of security risks and concerns in the place.

A bunch of bugs and it is not at the perfect position. Your job is to make this **production ready and perfect**. Here is the flow. A user comes to the service and creates an account. This is exactly the same as how you would do on an Instagram.

So you would go to Nick 's, you would create an account. It would need a next ID, which is exactly how an Instagram workflow works. This is exactly the same at this point. After that the user would maybe sign up for Uzume. At this point they would get a computer generated new ID - the Uzume Id.  Now they can Log in using their Nyx ID or their email ID or google auth (gmail).

The Uzume ID is just an in app ID for security and privacy (so their nyx Id is not exposed if they dont want it to be).

Your job is to perfect this flow - expose it using xtask (+ a just command), so that I can test it & harden it - think like what a malicious person would do to compromise it) 

We are not working on frontend now (xtask is emulating the frontend/client side) - everything flows through Heimdall, user only sees heimdall, auth flows through heka.

Follow the DRY Principle - no duplicate code - all auth & account is Heka using Ory - you should research this more to understand Ory using search and read its doc

Basically i want to be able to create a Nyx + Uzume account easily as a user and the user should not be able to compromise the service.

I dont want any duplicate code or files that are there but shouldn't be - I want this production grade.

Any security issues should be fixed by you immediately.

When you stop, I should have the login flow working well and cleanly, create account and sign in using xtask/just

It needs full e2e encryption all the all the bells and whistles you know.

Systematically fix all these issues, use your skills, use parallel agents, fix this in detail, these issues should be fixed and these flows should work in the end:

FIX EVERYTHING, NOTHING IS MINOR, IF YOU FIND NEW ISSUES FIX THEM, YOU WILL HAVE TO SETUP CADDY TO COME UP IN FRONT OF HEIMDALL, ALL TRAFFIC SHOULD BE HTTPS, WE ARE HTTPS ONLY FROM CADDY OUT, BACKEND EXPOSES HTTP ONLY TO XTASK.

# DEEP SECURITY & CODE AUDIT - Instagram-Style Auth Flow

## EXECUTIVE SUMMARY
**Status: NOT PRODUCTION READY**
- 47 security issues identified (7 Critical, 12 High, 18 Medium, 10 Low)
- 23 code quality issues
- 15 architectural flaws
- Manual implementations where libraries should be used
- Missing e2e encryption (TLS not implemented despite config)
- Dead code (CSRF module)

---

## CRITICAL SECURITY ISSUES (7)

### C1. NO TLS/e2e ENCRYPTION - BROKEN CONFIG
**Files:** `@/Monad/Heimdall/src/main.rs:38-41`, `@/Monad/Heimdall/src/config.rs:51-56`
**Severity:** CRITICAL
```rust
// config.rs has TLS paths:
pub tls_cert_path: Option<String>,
pub tls_key_path: Option<String>,

// But main.rs ignores them:
let listener = tokio::net::TcpListener::bind(&addr).await?;
axum::serve(listener, router)  // PLAIN HTTP ONLY
```
**Impact:**
- Passwords flow plaintext from xtask → Heimdall → Kratos
- Session tokens intercepted in transit
- Complete MITM vulnerability
- Violates OWASP Top 10 #2 (Cryptographic Failures)

**Fix:** Use Caddy - docker

---

### C2. PASSWORD DISPLAYED IN TERMINAL
**File:** `@/Monad/xtask/src/commands/create_account.rs:53-58`, `@/Monad/xtask/src/commands/login.rs:22-26`
**Severity:** CRITICAL
```rust
print!("Password: ");
io::stdout().flush()?;
let mut password = String::new();
io::stdin().read_line(&mut password)?;  // VISIBLE ON SCREEN!
```
**Impact:**
- Shoulder surfing attacks
- Terminal history leaks password (`~/.bash_history`)
- Screen recording captures credentials
- Violates security principle: "Never display secrets"

**Fix:** Use `rpassword` crate: `rpassword::read_password()`

---

### C3. OTP DISPLAYED IN TERMINAL
**File:** `@/Monad/xtask/src/commands/create_account.rs:47-51`
**Severity:** CRITICAL
```rust
print!("OTP Code: ");
let mut otp = String::new();
io::stdin().read_line(&mut otp)?;  // VISIBLE!
```
**Impact:**
- Same as password - shoulder surfing, history leaks
- OTPs are time-sensitive but still sensitive credentials

**Fix:** Mask OTP input like password

---

### C4. UNVERIFIED TLS CERTIFICATE VALIDATION
**File:** `@/Monad/xtask/src/auth.rs:27-31`
**Severity:** CRITICAL
```rust
let http = Client::builder()
    .timeout(REQUEST_TIMEOUT)
    .connect_timeout(CONNECT_TIMEOUT)
    .pool_idle_timeout(Duration::from_secs(60))
    .build()  // No certificate validation config!
```
**Impact:**
- If TLS were implemented, this client would accept any certificate
- Vulnerable to certificate spoofing
- No pinning, no custom CA support

**Fix:** Use caddy - docker

---

### C5. SECRET LEAKAGE IN LOGS
**File:** `@/Monad/Heimdall/src/proxy.rs:94-98`
**Severity:** CRITICAL
```rust
info!(
    method = %method,
    upstream_url = %upstream_url,  // Contains query params with secrets!
    "proxying request"
);
```
**Impact:**
- Query parameters (tokens, passwords) logged in plaintext
- Log aggregation systems expose secrets
- Compliance violation (GDPR, SOC2)

**Fix:** Sanitize URLs before logging, strip query params

---

### C6. JWT SECRET FROM ENV VAR (No Vault)
**File:** `@/Monad/Heimdall/src/config.rs:67-68`
**Severity:** CRITICAL
```rust
let jwt_secret = std::env::var("JWT_SECRET")
    .context("JWT_SECRET environment variable is required")?;
```
**Impact:**
- Secrets in environment variables leak in:
  - `/proc/<pid>/environ` (readable by any process)
  - Docker inspect
  - Process dumps
  - Shell history if exported
- No rotation mechanism
- No audit trail of access

**Fix:** Use HashiCorp Vault, AWS Secrets Manager, or Kubernetes secrets with rotation

---

### C7. INSECURE JWT ALGORITHM
**File:** `@/Monad/Heimdall/src/jwt.rs:71-72`
**Severity:** CRITICAL
```rust
let token = encode(
    &Header::new(Algorithm::HS256),  // HS256 is symmetric - key distribution nightmare
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
)?;
```
**Impact:**
- HS256 requires all services to share same secret
- If one service is compromised, all JWTs are forgeable
- No key rotation possible without downtime
- Should use RS256/ES256 (asymmetric) with key ID headers

**Fix:** Migrate to RS256 with JWKS endpoint for key rotation

---

## HIGH SEVERITY ISSUES (12)

### H1. UNNECESSARY CSRF - DEAD CODE
**File:** `@/Monad/Heimdall/src/csrf.rs` (entire file)
**Severity:** HIGH
- JWT Bearer tokens don't need CSRF protection
- CSRF only for cookie-based sessions
- Module imported in lib.rs but never wired to routes
- False sense of security

**Fix:** Delete the file and remove from lib.rs

---

### H2. CUSTOM RATE LIMITING (No Library)
**File:** `@/Monad/Heimdall/src/rate_limit.rs:1-110`
**Severity:** HIGH
```rust
// Manual token bucket implementation
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,  // In-memory only!
    max_requests: u32,
    window: Duration,
}
```
**Issues:**
- Not battle-tested (custom implementation)
- In-memory only (resets on restart)
- Single-server only (no distributed support)
- No Redis/centralized backend
- Memory leak: buckets never cleaned up (infinite HashMap growth)

**Fix:** Use `governor` crate (used by AWS SDK) + `axum-governor`

---

### H3. MANUAL EMAIL VALIDATION
**File:** `@/Monad/xtask/src/commands/create_account.rs:13-15`
**Severity:** HIGH
```rust
fn is_valid_email(email: &str) -> bool {
    let pattern = regex_lite::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    pattern.is_match(email)
}
```
**Issues:**
- Basic regex misses RFC 5322 edge cases
- No internationalized email support (Unicode)
- No MX record validation
- No disposable email detection
- No normalization (uppercase, dots in Gmail, +aliases)

**Fix:** Use `validator` crate with `ValidateEmail` trait

---

### H4. NO PASSWORD STRENGTH CHECK
**File:** `@/Monad/xtask/src/commands/create_account.rs:53-58`
**Severity:** HIGH
```rust
// Only length check, no complexity
if input.len() < 8 {
    eprintln!("Password must be at least 8 characters");
    continue;
}
```
**Issues:**
- Accepts: "password", "12345678", "qwertyui"
- No breach database check
- No common password detection
- No complexity requirements

**Fix:** Use `zxcvbn` crate (Dropbox's strength estimator) + HaveIBeenPwned API

---

### H5. NO OTP BRUTE FORCE PROTECTION
**File:** `@/Monad/xtask/src/commands/create_account.rs:47-51`
**Severity:** HIGH
- No rate limiting on OTP attempts
- 4-digit codes can be brute forced in ~10,000 attempts
- No exponential backoff
- No account lockout

**Fix:** Implement OTP-specific rate limiting (3 attempts max, then lockout)

---

### H6. NO ACCOUNT LOCKOUT
**File:** `@/Monad/xtask/src/commands/login.rs:12-56`
**Severity:** HIGH
- Unlimited failed login attempts
- No progressive delays
- No CAPTCHA after failures
- Credential stuffing attacks possible

**Fix:** Track failed attempts in DB/Redis, lock after 5 failures

---

### H7. HEIMDALL NOT IN TEST WORKFLOW
**File:** `@/Prithvi/compose/auth-test.yml`
**Severity:** HIGH
- Only starts: Postgres, Mailpit, Kratos
- Missing: Heimdall
- xtask calls Heimdall:3000 → connection refused
- Tests can't run

**Fix:** Add Heimdall service to auth-test.yml compose

---

### H8. PLAINTEXT QUERY PARAMS FOR NYX ID
**File:** `@/Monad/xtask/src/auth.rs:39`
**Severity:** HIGH
```rust
.get(format!("{}/api/nyx/id/check-availability?id={}", self.url, nyx_id))
```
**Issues:**
- No URL encoding (nyx_id with special chars breaks URL)
- GET request with query param logs in access logs
- Nyx ID might contain PII

**Fix:** Use POST with body, or at least `urlencoding::encode()`

---

### H9. NO JTI REPLAY PROTECTION
**File:** `@/Monad/Heimdall/src/jwt.rs:33-67`
```rust
pub jti: String,  // JTI present but never validated against replay
```
- JWT has `jti` claim but no validation
- Stolen tokens can be replayed until expiry
- No token revocation list

**Fix:** Implement token blacklist in Redis with JTI as key

---

### H10. NO INPUT SANITIZATION
**File:** `@/Monad/xtask/src/commands/create_account.rs:61-77`
**Severity:** HIGH
- Nyx ID not validated for injection attacks
- Email not normalized (user+tag@gmail.com vs user@GMAIL.com)
- Potential SQL injection (though sqlx uses prepared statements)

**Fix:** Validate Nyx ID format strictly, normalize email

---

### H11. SESSION TOKEN DISPLAYED
**File:** `@/Monad/xtask/src/commands/create_account.rs:95-96`, `@/Monad/xtask/src/commands/login.rs:44-45`
**Severity:** HIGH
```rust
if let Some(token) = body["session_token"].as_str() {
    println!("Session token: {token}");  // DISPLAYED IN TERMINAL!
}
```
**Impact:**
- Session token visible in terminal history
- Any process can read terminal buffer
- Should never display bearer tokens

**Fix:** Don't print sensitive tokens

---

### H12. NO SECURE HEADERS
**File:** `@/Monad/Heimdall/src/routes.rs:60-65`
**Severity:** HIGH
```rust
let cors = CorsLayer::new()
    .allow_origin(allowed_origins)
    // Missing: .vary([Origin])
```
- No HSTS header (Strict-Transport-Security)
- No X-Content-Type-Options
- No X-Frame-Options
- No Content-Security-Policy
- No Referrer-Policy

**Fix:** Add `tower-http::trace` layer with security headers

---

## MEDIUM SEVERITY ISSUES (18)

### M1. MEMORY LEAK IN RATE LIMITER
**File:** `@/Monad/Heimdall/src/rate_limit.rs:26`
```rust
buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,  // Never cleaned!
```
- Buckets accumulate forever per IP
- Memory exhaustion DoS possible

**Fix:** Add TTL cleanup or use `dashmap` with expiration

---

### M2. NO CONNECTION POOLING LIMITS
**File:** `@/Monad/Heimdall/src/state.rs:37-44`
**Severity:** MEDIUM
```rust
let http = Client::builder()
    .pool_max_idle_per_host(10)  // Too high for microservice
```
- No total connection limit
- Could exhaust file descriptors

**Fix:** Add `.pool_max_idle_total()` limits

---

### M3. INCONSISTENT ERROR HANDLING
**File:** `@/Monad/xtask/src/commands/create_account.rs:92-100`
```rust
match result {
    Ok(body) => { println!("..."); }  // Direct print
    Err(e) => { eprintln!("Error: {}", e); }  // Different format
}
```
- Mixed println!/eprintln!
- No structured logging
- No error codes for programmatic handling

**Fix:** Use structured error types, consistent formatting

---

### M4. NO REQUEST TIMEOUT ON DATABASE
**File:** `@/Monad/Heimdall/src/state.rs:53`
```rust
let db = PgPool::connect(&config.database_url).await?;
```
- No acquire timeout configured
- Could hang forever on DB issues

**Fix:** Use `PoolOptions` with `acquire_timeout()`

---

### M5. NO CIRCUIT BREAKER FOR KRATOS
**File:** `@/Monad/Heimdall/src/proxy.rs:133-138`
**Severity:** MEDIUM
```rust
let upstream_response = match upstream_req.send().await {
    Ok(resp) => resp,
    Err(err) => {
        return bad_gateway("Upstream service is unreachable");
    }
};
```
- No circuit breaker pattern
- Will keep trying failed upstream
- No backoff strategy

**Fix:** Use `backoff` crate with circuit breaker

---

### M6. UNWRAP USAGE IN PRODUCTION CODE
**Files:** Multiple locations
**Count:** 29+ unwrap/expect calls
```rust
// auth_layer.rs:69
let Some(token) = auth_value.strip_prefix("Bearer ") else {
    return unauthorized("Bearer token required", "invalid_auth_scheme");
};

// proxy.rs:102
reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap_or(reqwest::Method::GET)
```
**Risk:** Panics on unexpected input

**Fix:** Replace with proper error handling

---

### M7. NO HEALTH CHECK FOR DB
**File:** `@/Monad/Heimdall/src/health.rs` (assumed)
**Severity:** MEDIUM
- `/healthz` likely only checks HTTP server
- No database connectivity check
- Kubernetes won't detect DB failures

**Fix:** Add DB ping to health check

---

### M8. NO METRICS/TELEMETRY
**Severity:** MEDIUM
- No Prometheus metrics
- No auth success/failure rate tracking
- No latency histograms
- Can't alert on anomalies

**Fix:** Add `metrics` crate with Prometheus exporter

---

### M9. NO REQUEST ID PROPAGATION TO KRATOS
**File:** `@/Monad/Heimdall/src/proxy.rs:100-120`
**Severity:** MEDIUM
- Request ID generated but not forwarded to Kratos
- Distributed tracing broken
- Can't correlate logs across services

**Fix:** Forward `X-Request-Id` header to upstream

---

### M10. NO AUDIT LOG PERSISTENCE
**File:** `@/Monad/Heimdall/src/auth_layer.rs:101-142`
**Severity:** MEDIUM
```rust
info!(event = "auth_success", ...);  // Just stdout!
```
- Audit logs go to stdout only
- Lost on container restart
- No tamper-proof storage
- Compliance violation (can't prove what happened)

**Fix:** Write audit logs to append-only database or queue

---

### M11. NO DEVICE FINGERPRINTING
**Severity:** MEDIUM
- Can't detect account takeover from new devices
- No "new device login" email alerts
- No suspicious activity detection

**Fix:** Hash user-agent + IP, track known devices

---

### M12. NO BREACHED PASSWORD CHECK
**Severity:** MEDIUM
- Should check against HaveIBeenPwned API
- Prevents users from using compromised passwords

**Fix:** Add HIBP check during registration

---

### M13. PASSWORD NO CONFIRMATION IN LOGIN
**File:** `@/Monad/xtask/src/commands/login.rs:22-26`
**Severity:** MEDIUM
- Login asks password once (no confirmation)
- Typos cause frustration
- (Less critical than registration confirmation)

---

### M14. EMAIL NOT NORMALIZED
**Severity:** MEDIUM
- user+tag@gmail.com vs user@gmail.com
- John@Example.com vs john@example.com
- Gmail ignores dots: j.o.h.n@gmail.com

**Fix:** Normalize email before storage (lowercase, remove +tags, Gmail dots)

---

### M15. NO RATE LIMIT ON NYX ID CHECK
**File:** `@/Monad/Heimdall/src/routes.rs:275-297`
**Severity:** MEDIUM
- Endpoint at `/api/nyx/id/check-availability`
- No rate limiting (can enumerate all IDs)
- Information disclosure

**Fix:** Add strict rate limit to this endpoint

---

### M16. NO VALIDATION OF UPSTREAM URLS
**File:** `@/Monad/Heimdall/src/config.rs:86-92`
**Severity:** MEDIUM
```rust
let kratos_public_url = require_url("KRATOS_PUBLIC_URL")?;
// Only checks existence, not format!
```
- No URL parsing validation
- Malformed URLs cause runtime panics

**Fix:** Use `Url::parse()` and validate scheme

---

### M17. NO GRACEFUL SHUTDOWN
**File:** `@/Monad/Heimdall/src/main.rs` (assumed)
**Severity:** MEDIUM
- No SIGTERM handler
- In-flight requests dropped on shutdown

**Fix:** Use `tokio::signal` for graceful shutdown

---

### M18. HARDCODED DEFAULTS
**Files:** Multiple
```rust
const DEFAULT_HEIMDALL_URL: &str = "http://localhost:3000";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
```
- Hardcoded timeouts
- Hardcoded ports
- Should come from config

---

## LOW SEVERITY ISSUES (10)

### L1. DEAD CODE: CSRF MODULE
**File:** `@/Monad/Heimdall/src/csrf.rs`
- Imported but never used

### L2. UNUSED IMPORTS
**File:** Multiple files have unused imports

### L3. MAGIC NUMBERS
**File:** `@/Monad/Heimdall/src/rate_limit.rs`
- `max_requests: 100` without named constant

### L4. NO API VERSIONING
- Routes don't have `/v1/` prefix

### L5. NO OPENAPI SPEC
- No documentation of endpoints

### L6. INCONSISTENT NAMING
- `KratosClient` talks to Heimdall (confusing)

### L7. NO TRACING SPANS
- Missing structured tracing spans

### L8. CLIPPY WARNINGS IGNORED
- `#![warn(clippy::pedantic)]` but many issues

### L9. TODO/FIXME COMMENTS
- None found, but likely missing

### L10. NO CHANGELOG
- No tracking of security changes

---

## ARCHITECTURAL FLAWS (15)

### A1. TIGHT COUPLING TO KRATOS
- Heimdall directly proxies Kratos endpoints
- Hard to swap auth provider

### A2. NO SERVICE MESH
- Direct HTTP calls between services
- Should use gRPC or service mesh

### A3. MONOCL STATE
- In-memory rate limiting breaks horizontal scaling

### A4. NO EVENT DRIVEN ARCH
- Audit logs not published to queue

### A5. SYNCHRONOUS BLOCKING
- DB calls block async runtime

### A6. NO CACHE LAYER
- Every auth check hits DB

### A7. NO READ REPLICAS
- DB is single point of failure

### A8. NO CONNECTION RETRY
- One DB fail = total outage

### A9. NO ZERO-DOWNTIME DEPLOY
- No rolling update strategy

### A10. NO FEATURE FLAGS
- Can't disable features without deploy

### A11. NO A/B TESTING
- Can't test auth flow variants

### A12. NO DARK LAUNCH
- Can't test new auth on subset of traffic

### A13. NO CANARY DEPLOYMENTS
- All-or-nothing releases

### A14. NO AUTOMATED ROLLBACK
- Manual intervention on failure

### A15. NO CHAOS TESTING
- No failure injection

---

## CODE QUALITY ISSUES (23)

### Q1-Q10. UNWRAP USAGE (10 instances)
### Q11-Q15. EXPECT USAGE (5 instances)
### Q16-Q20. CLONE OVERHEAD (5 instances)
### Q21-Q23. MAGIC STRINGS (3 instances)

---

## COMPLIANCE VIOLATIONS

### GDPR
- No data retention policy
- No right to erasure mechanism
- No data export capability

### SOC2
- No audit trail integrity
- No access control reviews

### ISO 27001
- No risk assessment
- No security policy

---

## REMEDIATION ROADMAP

### Phase 1: Critical (Week 1)
1. Implement TLS in Heimdall - use caddy - docker
2. Remove CSRF dead code
3. Mask password/OTP input
4. Fix secret logging

### Phase 2: High (Week 2)
1. Replace rate limiter with `governor`
2. Replace email validation with `validator`
3. Add password strength `zxcvbn`
4. Add account lockout

### Phase 3: Medium (Week 3)
1. Add circuit breaker
2. Add metrics
3. Add health checks
4. Add audit log persistence

### Phase 4: Hardening (Week 4)
1. Migrate to RS256 JWT
2. Add device fingerprinting
3. Add breached password check
4. Add 2FA/TOTP support

---

## SUMMARY STATISTICS

| Category | Count | Critical | High | Medium | Low |
|----------|-------|----------|------|--------|-----|
| Security | 47 | 7 | 12 | 18 | 10 |
| Code Quality | 23 | 0 | 5 | 12 | 6 |
| Architecture | 15 | 0 | 3 | 8 | 4 |
| Compliance | 6 | 3 | 2 | 1 | 0 |
| **TOTAL** | **91** | **10** | **22** | **39** | **20** |

---

## VERDICT

**NOT PRODUCTION READY**

This implementation has fundamental security gaps that make it unsuitable for production:
1. No encryption in transit (TLS config is fake)
2. Secrets displayed in terminal
3. Custom security implementations (rate limit, CSRF)
4. No proper secrets management
5. Dead code and broken test workflow
