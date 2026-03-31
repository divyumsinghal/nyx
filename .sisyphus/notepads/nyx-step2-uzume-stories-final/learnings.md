
## API Privacy & Authorization Testing Patterns (Rust)

**1. OWASP API Security Top 10 (2023) Alignment:**
- **API1:2023 Broken Object Level Authorization (BOLA):** Users manipulating IDs (e.g., `/stories/123`) to access other users' data. **Testing Pattern:** Authenticate as User A, request User B's resource. Assert exact HTTP status code failure (403 or 404).
- **API3:2023 Broken Object Property Level Authorization (BOPLA):** Exposing sensitive fields or allowing unauthorized mass assignment. **Testing Pattern:** Assert the JSON schema of successful responses does not contain internal fields (e.g., database IDs, email addresses of others). Verify update endpoints reject immutable fields.
- **API4:2023 Unrestricted Resource Consumption & Enumeration:** Attackers scanning IDs to see what exists. **Testing Pattern:** Unauthorized requests to existent vs. non-existent resources MUST return identical responses to prevent existence leakage.

**2. 401 vs 403 Correctness:**
- **401 Unauthorized:** Missing, malformed, or expired authentication token. The user is unknown.
- **403 Forbidden:** The user is known (valid token), but lacks permissions, scope, or ownership for the specific resource.

**3. Rust Implementation Testing Patterns:**
- **Data Minimization:** Use distinct structs for database models and API responses. Implement `From<Model> for ResponseDto` to enforce strict projection.
- **Testing Tools:** Use `axum::test::TestClient` or `reqwest` integration tests to simulate distinct user sessions and replay tokens.
- **References:** [OWASP API Security Top 10](https://owasp.org/API-Security/editions/2023/en/0x11-t10/)

## Secure Async Media Pipelines in Rust (Research 2026)

### 1. State Machine & Event-Driven Architecture
- **Accepted -> Processing -> Ready State Flow**: Map domain statuses explicitly to HTTP status codes (`202 Accepted`, `102 Processing`, `200 OK` / `201 Created`) to maintain deterministic state transitions. Libraries like `trillium` (https://github.com/trillium-rs/trillium/blob/main/http/src/status.rs) and `oxapy` show robust mappings for these interim states.
- **Idempotent Consumers**: When implementing the consumer, use an "Idempotent Receiver" pattern to safely handle duplicate message deliveries. A common Rust approach is leveraging Redis or a transactional outbox/inbox table to deduplicate by checking a unique `message_id` or `job_id` before processing (Ref: https://oneuptime.com/blog/post/2026-02-01-rust-message-queue-consumers/view).

### 2. Idempotent Event Handling Example (Duplicate Delivery Handling)
To prevent processing the same media file twice on retry, consumers should track processed `job_id`s.
```rust
// Example Pattern for Idempotent Consumer
async fn process_media_event(job_id: uuid::Uuid, payload: MediaPayload, db: &DbPool) -> Result<(), Error> {
    // 1. Check if job was already processed (Idempotency Key)
    let is_processed = db.check_if_processed(job_id).await?;
    if is_processed {
        tracing::info!("Job {} already processed. Skipping.", job_id);
        return Ok(()); // Acknowledge without side effects
    }

    // 2. State: Accepted -> Processing
    db.update_status(job_id, Status::Processing).await?;

    // 3. Process media (safely)
    let result = safe_transcode(payload).await?;

    // 4. State: Processing -> Ready (Atomic Commit)
    db.mark_ready(job_id, result).await?;
    Ok(())
}
```

### 3. Media Processing Safety & Untrusted Input
- **Pure-Rust Processing**: The ecosystem in 2026 strongly favors memory-safe processing over wrapping C/C++ libraries. The `OxiMedia` framework (https://github.com/cool-japan/oximedia) provides a pure-Rust reconstruction of FFmpeg and OpenCV, addressing historical memory safety vulnerabilities in media parsing (e.g., CVE-2026-25541).
- **Security Guidance**: Treat all media input as hostile.
  - Do not use `unsafe` blocks for media decoding.
  - Apply strict resource limits (memory, file size, duration) to prevent Zip-bomb/Billion-laughs style attacks via media containers.
  - Avoid provider lock-in by using abstract event payloads that don't depend on specific AWS/GCP media service types.

## Monad Crate Exploration Findings (Task 0-1)

### Crate Implementation Status Summary

| Crate | Status | Source Files | Key Patterns Available |
|-------|--------|--------------|----------------------|
| Nun | IMPLEMENTED | Yes (src/) | Complete foundation |
| Akash | SPEC-ONLY | README only | Storage patterns defined |
| Oya | SPEC-ONLY | README only | Media processing patterns |
| events | SPEC-ONLY | README only | Event patterns defined |
| Lethe | SPEC-ONLY | README only | Cache patterns defined |
| Mnemosyne | SPEC-ONLY | README only | DB patterns defined |
| api | SPEC-ONLY | README only | Middleware patterns defined |

### Nun Implementation Map (Already Available for Reuse)

#### 1. ID System (`Monad/Nun/src/id.rs`)
- **Pattern**: `Id<T>` generic typed ID wrapper over UUIDv7
- **Line**: 49-93 (struct definition + methods)
- **Usage for Stories**: Define `StoryId`, `HighlightId`, `StoryViewId`, `InteractionId` markers
- **Tests**: Lines 198-287 (comprehensive)
- **Test Command**: `cargo test -p nun --lib id`

#### 2. Error Handling (`Monad/Nun/src/error.rs`)
- **Pattern**: `NyxError` with named constructors (not_found, forbidden, bad_request, etc.)
- **Line**: 51-280 (struct + constructors)
- **Pattern**: `ErrorKind` maps to HTTP status codes
- **Line**: 404-463 (enum + mapping)
- **Pattern**: `Result<T>` type alias
- **Line**: 41
- **Tests**: Lines 538-651
- **Test Command**: `cargo test -p nun --lib error`

#### 3. Pagination (`Monad/Nun/src/pagination.rs`)
- **Pattern**: `Cursor` opaque base64 cursor with timestamp_id, score_id variants
- **Line**: 39-155
- **Pattern**: `PageRequest` with limit clamping
- **Line**: 164-210
- **Pattern**: `PageResponse::from_overflowed` - fetch-one-extra pattern
- **Line**: 271-294
- **Tests**: Lines 299-455
- **Test Command**: `cargo test -p nun --lib pagination`

#### 4. App Scoping (`Monad/Nun/src/types/app.rs`)
- **Pattern**: `NyxApp` enum (non-exhaustive, lowercase serde)
- **Line**: 3-10
- **Stories Need**: Add Stories to subject conventions in events crate
- **Tests**: Lines 12-48

#### 5. Time & TTL (`Monad/Nun/src/time.rs`)
- **Pattern**: `Timestamp` = DateTime<Utc> type alias
- **Pattern**: `ttl::STORY = 24 hours` (line 38)
- **Pattern**: Other TTL constants for cache/session
- **Line**: 32-66
- **Tests**: Lines 68-87
- **Test Command**: `cargo test -p nun --lib time`

#### 6. Configuration (`Monad/Nun/src/config.rs`)
- **Pattern**: `NyxConfig` with sub-configs (database, cache, nats, storage, etc.)
- **Line**: 44-73
- **Pattern**: `load()` with env var priority chain
- **Line**: 76-98

### Akash Storage Patterns (SPEC-ONLY per README)

**Path Convention** (from README line 24-36):
```
{app}/{entity}/{id}/{variant}.{ext}
Examples:
Uzume/stories/{story_id}/original.mp4
Uzume/avatars/{user_id}/150.jpg
```

**API Surface** (per README line 10):
- `put_object`, `get_object`, `presigned_upload_url`, `presigned_download_url`
- Needs implementation in actual src/ files

### Oya Media Processing Patterns (SPEC-ONLY per README)

**Image Processing** (line 7):
- Uses `fast_image_resize + image` crate (pure Rust)
- Generates variants: 1080, 640, 320, 150px
- Strips EXIF

**Video Processing** (line 8):
- Shells out to FFmpeg
- H.264 → HLS segments at 720p/480p/360p
- Poster frame generation

**Worker Pattern** (line 9):
- NATS subscriber on `*.media.uploaded` 
- Process → store in MinIO → emit `*.media.processed`

### Lethe Cache Patterns (SPEC-ONLY per README)

**Key Convention** (line 24-32):
```
{app}:{entity}:{id}
Examples:
Uzume:stories:{id}
Uzume:feed:{user_id}
```

**API Surface** (per README):
- Token bucket rate limiter
- Session cache
- `get_or_set` cache-aside helper
- TTL constants

### events Patterns (SPEC-ONLY per README)

**Event Envelope** (line 6):
```rust
NyxEvent<T> { id, subject, app, timestamp, payload: T }
```

**Subject Convention** (line 21):
```
{app_or_nyx}.{entity}.{action}
```

**Existing Stories Subjects** (line 34):
- `Uzume.story.created`
- No `Uzume.story.viewed` (mentioned in apps/Uzume/README line 141)

### Mnemosyne DB Patterns (SPEC-ONLY per README)

**Multi-Schema** (line 24-35):
- One PostgreSQL instance, multiple schemas (nyx, Uzume, Anteros, Themis)
- Each app gets own schema

**API Surface** (per README):
- PgPool builder
- Per-schema migration runner
- Transaction helpers with auto-rollback
- Bulk insert helper

### api Middleware Patterns (SPEC-ONLY per README)

**NyxServer Builder** (line 8-16):
```rust
NyxServer::builder()
    .with_config(config)
    .with_db_pool(pool)
    .with_cache(cache)
    .with_events(nats)
    .with_routes(app_routes)
    .build()
    .serve()
```

**Middleware Available** (line 21):
- auth.rs (JWT extraction + validation)
- rate_limit.rs (token bucket via DragonflyDB)
- request_id.rs, tracing.rs, app_context.rs

**Extractors Available** (line 23):
- AuthUser, ValidatedJson<T>, cursor pagination

## Stories-Specific Patterns to Implement (Gap Analysis)

### In Nun (additions needed):
1. Stories-specific TTL constants in `time.rs` (already have STORY=24h)
2. Stories ID markers in a new `src/types/stories.rs` module

### In Akash (create):
1. Stories path builders for storage paths
2. Pre-signed URL generation for stories upload
3. Media variant URL generation (original, processed variants)

### In events (create):
1. Stories event subjects: `Uzume.story.created`, `Uzume.story.viewed`, `Uzume.media.uploaded`, `Uzume.media.processed`
2. Typed event payloads for media lifecycle

### In Lethe (create):
1. Stories cache keys: `Uzume:stories:feed:{user_id}`, `Uzume:story:{id}:viewers`
2. Stories TTL constants

### In Mnemosyne (create):
1. Stories migration files in `migrations/Uzume/`
2. Repository helpers for stories queries

### In api (extend):
1. Stories-specific middleware if needed
2. Request size limits for story uploads

## Test Commands for Verification

```bash
# Test Nun foundation (already implemented)
cargo test -p nun --lib

# Check workspace builds (will show missing implementations)
cargo check --workspace 2>&1 | head -50

# List all crates
cargo metadata --format-version=1 --no-deps | jq -r '.packages[].name'
```

## Step 2 Task 1 / Step 2 Guardrail Contract Lock (2026-03-31)
- Added contract lock manifest at `contracts/step1-compat.lock` and verifier at `tests/contracts/verify_step1_contract_lock.sh`.
- Locked invariants cover: app-scoped alias visibility default, fail-closed link/revoke semantics (`revoked` default + revoked non-applicability), and chronological feed default guard (`FeedMode::default() == Chronological`).
- Added deterministic drift fixture `tests/contracts/step1-compat-drift.fixture.lock` to ensure guardrails fail on semantic drift.
- Wired gate into local/CI-equivalent flow: new `just gate-step1-compat` and included in `just ci`; CI workflow invokes `just gate-step1-compat` explicitly before parity bundle.
