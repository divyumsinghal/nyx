
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

## Step 2 Task 3 Exploration: Monad/Oya + Monad/events (2026-03-31)

### Crate Status Summary

| Crate | Status | Implementation |
|-------|--------|-----------------|
| Monad/Oya | SPEC-ONLY | README.md only - no src/ |
| Monad/events | SPEC-ONLY | README.md only - no src/ |

### Event Pattern (from events README)

**Event Envelope** (line 6):
```rust
NyxEvent<T> { id, subject, app, timestamp, payload: T }
```

**Subject Convention** (line 21):
```
{app_or_nyx}.{entity}.{action}
```

**Existing Story Subjects** (line 34):
- `Uzume.story.created` - defined

### Media Processing Pipeline (from Oya README)

**Flow**: NATS subscriber on `*.media.uploaded` → process → store in MinIO → emit `*.media.processed`

**State Transition Implied**:
1. `accepted` - initial upload (via upload presigned URL)
2. `processing` - Oya worker picks up `*.media.uploaded` event
3. `ready` - variants generated, `*.media.processed` emitted

### Available Foundation from Nun (IMPLEMENTED)

| Pattern | File | Lines | Notes |
|---------|------|-------|-------|
| Error with HTTP mapping | Monad/Nun/src/error.rs | 51-280 | `ErrorKind` maps to status codes |
| Pagination | Monad/Nun/src/pagination.rs | 39-294 | Cursor-based, fetch-one-extra |
| Timestamp + TTL | Monad/Nun/src/time.rs | 32-66 | `Timestamp = DateTime<Utc>`, `ttl::STORY = 24h` |
| Typed IDs | Monad/Nun/src/id.rs | 49-93 | `Id<T>` over UUIDv7 |
| App enum | Monad/Nun/src/types/app.rs | 3-10 | `NyxApp` non-exhaustive |

### Extension Points for Task 3

1. **Event payload typing**: Add `MediaUploadedPayload`, `MediaProcessedPayload` in events crate
2. **State enum**: Define `MediaStatus { Accepted, Processing, Ready }` in Nun or domain
3. **Idempotency**: Use job_id/media_id in payload - check before processing (pattern documented in Task 0 learnings)
4. **Retry safety**: NATS JetStream acknowledgment semantics

### Contract Lock Relevance

From `contracts/step1-compat.lock`:
- No direct media/lifecycle locks yet
- Pattern: lock at implementation time similar to Task 1

### Recommended Test Commands (when implemented)

```bash
# Test Nun foundation
cargo test -p nun --lib

# Build events crate (will fail - SPEC-ONLY)
cargo build -p nyx-events

# Build Oya crate (will fail - SPEC-ONLY)
cargo build -p oya
```

### Gap Analysis for Task 3

- **Monad/events**: No implementation - needs `envelope.rs`, `subjects.rs`, `publisher.rs`, `subscriber.rs`
- **Monad/Oya**: No implementation - needs `lib.rs`, `worker.rs`, `image.rs`, `video.rs`, `pipeline.rs`, `config.rs`
- **Missing from Nun**: Media status enum, media-specific ID markers, media TTL constants

## Step 2 Task 3: Event-Driven Idempotent Consumer & State Machine Patterns (2026-03-31)

### 1. Concrete Deduplication Strategy: Inbox Pattern via Persistence
A highly reliable pattern to handle duplicate message delivery natively in Rust without relying on the event provider's specific constraints (avoiding lock-in).
**Pattern:** Inbox Table / Idempotency Key Tracking with `ON CONFLICT DO NOTHING`.
**Mechanism:**
- The event payload includes a globally unique `idempotency_key` (or `job_id`).
- Before processing the event, a database transaction is initiated to insert the `idempotency_key` into an `inbox` (or `idempotency`) table.
- If `n_inserted_rows == 0`, the consumer knows this is a duplicate delivery and can safely acknowledge the message to the broker and skip execution, optionally returning the cached response.
**Tradeoffs:**
- *Pros:* Decouples idempotency from the message broker (works with NATS, RabbitMQ, Kafka). Extremely high reliability. Ensures exactly-once *processing* side-effects.
- *Cons:* Adds database round-trip overhead before processing. Requires DB connection per worker.
**Reference:** `LukeMathWalker/zero-to-production` (Chapter 10: Idempotency). [Persistence Code Reference](https://github.com/LukeMathWalker/zero-to-production/blob/main/src/idempotency/persistence.rs).

### 2. Provider-Agnostic Event Emitting: Transactional Outbox
To ensure events are published reliably *only* if the DB state transition succeeds, avoiding "ghost events" or "missed events".
**Pattern:** Transactional Outbox.
**Mechanism:**
- Within the same PostgreSQL transaction where you update `MediaStatus` from `Accepted` -> `Processing`, you insert an `OutboxEvent` record.
- A separate async worker polls/tails the `outbox` table and publishes the event to NATS/RabbitMQ.
- **Provider Lock-in Avoided:** The core service doesn't know about NATS JetStream semantics. It just writes to the `outbox` table.
**Reference:** [Meteroid OSS Outbox Pattern](https://github.com/meteroid-oss/meteroid/blob/main/modules/meteroid/crates/meteroid-store/src/repositories/outbox.rs).

### 3. Deterministic State Machines under Retries
To ensure safe `Accepted -> Processing -> Ready` lifecycles, encode the state transitions using Rust's `Typestate` pattern or explicit `enum` State Machines.
**Pattern:** Typestate / State Machine Enums with Strict Transitions.
**Mechanism:**
- Do not use a generic `status: String` or generic `status: Status` mutator.
- Implement explicit structs for each state (`struct AcceptedMedia`, `struct ProcessingMedia`, `struct ReadyMedia`).
- Define transition methods that consume `self` (taking ownership), preventing double-processing:
  `impl AcceptedMedia { pub fn start_processing(self) -> ProcessingMedia { ... } }`
- **Under Retries:** If a worker crashes during `Processing`, the retry will fetch the DB record. If the record is already marked `Processing`, the worker must know how to safely resume or reset to `Accepted`. The state machine ensures no step can be skipped (e.g., cannot jump from `Accepted` straight to `Ready`).
**Reference:** [Rust Typestate Pattern (oneuptime.com, 2026)](https://oneuptime.com/blog/post/2026-01-30-rust-type-state-pattern/view) & `state-machines` crate patterns.

### 4. Verification Checklist for Local Testing (Task 3)
- [ ] Send duplicate `*.media.uploaded` event -> assert second event is dropped via Inbox constraint.
- [ ] Mock worker crash after `Accepted -> Processing` update -> assert outbox event is successfully published later by outbox relay.
- [ ] Send unexpected `media.processed` event while in `Accepted` state -> assert compile/runtime error rejecting the invalid state transition.

### Pre-signed S3 URL Security (Rust/Task 2)
*   **Presigned PUT vs POST:** `aws-sdk-rust` lacks native `generate_presigned_post` support (Issue #863). To enforce upload constraints natively in Rust using `aws-sdk-s3`, use **Presigned PUT** and strictly lock down the headers during generation.
*   **Strict Upload Contracts (Size & Integrity):** Require the client to provide `Content-MD5` (or `checksum_sha256`) and the *exact* `content_length` in their pre-flight request. The backend validates the size (`size <= MAX_SIZE`), and includes `content_length` and `checksum` in the `put_object()` builder. If the client alters the payload size or content, S3 rejects the request with `SignatureDoesNotMatch` or `InvalidDigest`.
*   **MIME Constraints:** Include `content_type("...")` in the presigned PUT builder. The client MUST use that exact MIME type, preventing bypassing type restrictions.
*   **Key Naming Strategy:** To prevent path traversal and object overwrites, the server MUST generate opaque, randomized object keys (e.g., UUIDs). Clients should only use an `upload_id` reference and never control the raw S3 key.
*   **Expiration Controls:** Keep `expires_in` tight (e.g., 60-120 seconds).
*   **Validation Strategy (Quarantine Pipeline):** Direct uploads to a `quarantine/` prefix. The client must call a `POST /finalize` endpoint after upload. The backend then verifies the object size/existence and optionally scans it before moving it to a `published/` prefix.
*   **Download Safety:** Generate presigned GET URLs with `response_content_disposition("attachment")` and `response_content_type_options("nosniff")` to prevent Stored XSS via inline HTML/SVG rendering.

**Sources:**
- S3 Pre-signed URLs: Security Guide & Threat Model (https://newsletter.securepatterns.dev/p/pre-signed-urls-the-secure-implementation-guide)
- Enforcing Upload Contracts with S3 Presigned URLs (https://medium.com/codetodeploy/enforcing-upload-contracts-with-s3-presigned-urls-45be8cc1437c)
- Securing Amazon S3 presigned URLs for serverless applications (https://aws.amazon.com/blogs/compute/securing-amazon-s3-presigned-urls-for-serverless-applications/)
- AWS SDK Rust Docs (https://docs.aws.amazon.com/sdk-for-rust/latest/dg/presigned-urls.html)

## Step 2 Task 2: Storage/Object Lifecycle/Presign Implementation Patterns (2026-03-31)

### Crate Implementation Status for Task 2

| Crate | Status | Source Files | Notes |
|-------|--------|--------------|-------|
| Monad/Akash | SPEC-ONLY | README only | Storage patterns defined, needs implementation |
| Monad/Nun | IMPLEMENTED | Yes (src/) | Foundation types, config, validation |
| Monad/Oya | SPEC-ONLY | README only | Media processing patterns defined |
| Monad/Lethe | SPEC-ONLY | README only | Cache key patterns defined |
| Monad/events | SPEC-ONLY | README only | Event patterns defined |
| Monad/Mnemosyne | SPEC-ONLY | README only | DB patterns defined |
| Monad/api | SPEC-ONLY | README only | Middleware patterns defined |

### Akash Storage Patterns (SPEC-ONLY - needs implementation)

**Path Convention** (Akash/README.md line 24-36):
```
{app}/{entity}/{id}/{variant}.{ext}
Examples:
Uzume/stories/{story_id}/original.mp4
Uzume/stories/{story_id}/1080.jpg
Uzume/stories/{story_id}/640.jpg
Uzume/avatars/{user_id}/150.jpg
```

**API Surface to Implement** (Akash/README.md line 10):
- `put_object` - Store object in MinIO/S3
- `get_object` - Retrieve object from MinIO/S3
- `presigned_upload_url` - Generate upload URL with expiration
- `presigned_download_url` - Generate download URL with expiration

**Module Structure** (Akash/README.md line 13-21):
```
Akash/
├── src/
│   ├── lib.rs
│   ├── client.rs          # StorageClient (wraps rust-s3 Bucket), connection config
│   ├── upload.rs           # Upload helpers: put_object, presigned_upload_url
│   ├── download.rs         # Download helpers: get_object, presigned_download_url
│   └── paths.rs            # Path convention: {app}/{entity}/{id}/{variant}.{ext}
```

### Reusable Patterns from Nun (IMPLEMENTED)

#### 1. StorageConfig (Nun/config.rs line 252-273)
```rust
pub struct StorageConfig {
    pub endpoint: String,           // e.g., http://localhost:9000
    pub region: String,              // default: us-east-1
    pub bucket: String,              // default: nyx
    pub access_key: Sensitive<String>,
    pub secret_key: Sensitive<String>,
}
```
**Line**: 252-273
**Usage**: Acquired via `NyxConfig::load()` → `config.storage`

#### 2. Validation (Nun/validation.rs)
- **Phone** (line 38-68): E.164 format validation
- **Email** (line 77-120): Basic format check
- **Alias** (line 138-203): 3-30 chars, lowercase alphanumeric + underscore
- **Display name** (line 218-246): 1-50 chars, no control chars
- **Hashtag** (line 260-298): 1-100 chars, alphanumeric + underscore

**Test Command**: `cargo test -p nun --lib validation`

#### 3. Error Handling (Nun/error.rs)
- **Pattern**: `NyxError::bad_request()`, `NyxError::validation()`, `NyxError::not_found()`
- **Line**: 51-280 (constructors), 404-463 (ErrorKind enum)
- **HTTP Mapping**: ErrorKind → HTTP status code

**Test Command**: `cargo test -p nun --lib error`

#### 4. TTL Constants (Nun/time.rs line 32-66)
- `ttl::STORY` = 24 hours (line 38)
- `ttl::SESSION_CACHE` = 15 minutes
- `ttl::FEED_CACHE` = 10 minutes

**Test Command**: `cargo test -p nun --lib time`

### Presigned URL Security Patterns (Research 2026)

**Strict Upload Contract**:
1. **Content-Length**: Backend validates size constraint, includes in presigned URL
2. **Content-MD5**: Client provides checksum, S3 validates integrity
3. **Content-Type**: Lock MIME type in presigned URL, prevents bypass
4. **Key Naming**: Server generates opaque UUID keys, no client path control
5. **Expiration**: Keep expires_in tight (60-120 seconds)

**Quarantine Pattern**:
- Upload to `quarantine/{upload_id}/{filename}`
- Client calls `POST /finalize` after upload
- Backend verifies size/existence before moving to `published/`

**Download Safety**:
- `response_content_disposition("attachment")`
- `response_content_type_options("nosniff")`

### Typed Path/Key Construction Patterns

**Stories Paths**:
```
Uzume/stories/{story_id}/original.{ext}     # Original upload
Uzume/stories/{story_id}/1080.{ext}         # Processed variant
Uzume/stories/{story_id}/640.{ext}           # Processed variant
Uzume/stories/{story_id}/320.{ext}          # Processed variant
Uzume/stories/{story_id}/thumb.jpg          # Thumbnail
```

**Highlight Paths**:
```
Uzume/highlights/{highlight_id}/cover.jpg  # Cover image
```

**Cache Keys** (Lethe pattern):
```
Uzume:stories:feed:{user_id}                # User's stories feed
Uzume:story:{id}:viewers                    # Story viewers list
Uzume:story:{id}:interactions               # Story interactions
```

### Fail-Closed Validation Errors

**Pattern from Nun/error.rs**:
```rust
Err(NyxError::bad_request(
    "invalid_media_type",
    "Only image/jpeg, image/png, video/mp4 allowed",
))
```

**Test Cases to Implement**:
- Invalid MIME type → 400 Bad Request
- Oversize file → 400 Bad Request with max size info
- Invalid key format → 400 Bad Request
- Expired presigned URL → 400/403 depending on implementation

### Deterministic Tests for Accept/Reject Cases

**Happy Path**:
- Valid JPEG upload → presigned URL generated → PUT succeeds → 200 OK

**Reject Cases**:
- Wrong MIME type → 400 with "invalid_media_type"
- Oversize → 400 with "file_too_large"
- Path traversal attempt → 400 with "invalid_key"
- Expired URL → 403 with "presigned_url_expired"

### Implement First Checklist (Task 2)

1. **Akash/Cargo.toml** - Create with Nun dependency, rust-s3
2. **Akash/src/client.rs** - StorageClient wrapping rust-s3 Bucket
3. **Akash/src/paths.rs** - Path builders for stories/highlights
4. **Akash/src/upload.rs** - put_object + presigned_upload_url with validation
5. **Akash/src/download.rs** - get_object + presigned_download_url
6. **Akash/tests/presign.rs** - Validate accept/reject cases
7. **Akash/tests/validation.rs** - MIME/size validation tests
8. **Nun/src/types/media.rs** - MediaType, MediaVariant enums (if not already present)

### Verification Commands

```bash
# Test Nun foundation (already implemented)
cargo test -p nun --lib

# Build Akash crate (will fail - SPEC-ONLY)
cargo build -p akash

# List storage config usage
grep -r "StorageConfig" --include="*.rs" Monad/

# Check path convention usage
grep -r "Uzume/stories" --include="*.rs" .
```

### Gap Analysis for Task 2

- **Akash**: No implementation - needs all src/ files
- **Nun**: May need MediaType/MediaVariant enums in types/media.rs
- **Validation**: Need media-specific validators (MIME whitelist, size limits)
- **Key naming**: Need stories-specific path builders

## Step 2 Task 2 Implementation Learnings (Akash) - 2026-03-31

## Step 2 Task 2 Implementation Learnings (Akash) - 2026-03-31

- StorageClient wraps rust-s3 Bucket via Box<Bucket> from Bucket::new in rust-s3 0.36.
- MinIO/S3-compatible setup works with Region::Custom plus with_path_style for deterministic URLs.
- Credentials::new(Some(access), Some(secret), None, None, None) maps cleanly from nun Sensitive values via expose().
- rust-s3 0.36 presign_put and presign_get are async and return String URLs.
- UploadMetadata validation should be fail-closed: normalize MIME, enforce allowlist, reject zero bytes, enforce max size.
- Typed presign response structs should carry method, URL, and headers to preserve security header contracts.

## Step 2 Task 2 Implementation Learnings (Akash) — 2026-03-31

- `StorageClient` can safely wrap `rust-s3` via `Box<Bucket>` because `Bucket::new` returns boxed bucket in `rust-s3` 0.36.
- For MinIO/S3-compatible endpoints, `Region::Custom { region, endpoint }` with `.with_path_style()` keeps URL generation deterministic in local/self-hosted setups.
- `Credentials::new(Some(access), Some(secret), None, None, None)` integrates cleanly with `nun::Sensitive<String>` using `.expose()`.
- Presign helpers are async in rust-s3 0.36:
  - `presign_put(path, expiry_secs, custom_headers, custom_queries).await`
  - `presign_get(path, expiry_secs, custom_queries).await`
- `UploadMetadata` validation works best as fail-closed:
  - normalize MIME to lowercase trim
  - explicit MIME allowlist
  - reject zero bytes
  - enforce deterministic max size with `NyxError::payload_too_large("upload_too_large", ...)`
- Typed presign response structs (`PresignedUpload`, `PresignedDownload`) should carry method + headers map + URL to keep callers transport-agnostic and security-header aware.

## Step 2 Task 2 Implementation Learnings (Akash) — 2026-03-31

- `StorageClient` can safely wrap `rust-s3` via `Box<Bucket>` because `Bucket::new` returns boxed bucket in `rust-s3` 0.36.
- For MinIO/S3-compatible endpoints, `Region::Custom { region, endpoint }` with `.with_path_style()` keeps URL generation deterministic in local/self-hosted setups.
- `Credentials::new(Some(access), Some(secret), None, None, None)` integrates cleanly with `nun::Sensitive<String>` using `.expose()`.
- Presign helpers are async in rust-s3 0.36:
  - `presign_put(path, expiry_secs, custom_headers, custom_queries).await`
  - `presign_get(path, expiry_secs, custom_queries).await`
- `UploadMetadata` validation works best as fail-closed:
  - normalize MIME to lowercase trim
  - explicit MIME allowlist
  - reject zero bytes
  - enforce deterministic max size with `NyxError::payload_too_large("upload_too_large", ...)`
- Typed presign response structs (`PresignedUpload`, `PresignedDownload`) should carry method + headers map + URL to keep callers transport-agnostic and security-header aware.
Task2 Akash note.


## Step 2 Task 2 Implementation Learnings (Akash) - 2026-03-31

- StorageClient wraps rust-s3 Bucket via Box<Bucket> from Bucket::new in rust-s3 0.36.
- MinIO/S3-compatible setup works with Region::Custom plus with_path_style for deterministic URLs.
- Credentials::new(Some(access), Some(secret), None, None, None) maps cleanly from nun Sensitive values via expose().
- rust-s3 0.36 presign_put and presign_get are async and return String URLs.
- UploadMetadata validation should be fail-closed: normalize MIME, enforce allowlist, reject zero bytes, enforce max size.
- Typed presign response structs should carry method, URL, and headers to preserve security header contracts.

## Step 2 Task 3 Implementation Learnings (events + Oya) - 2026-03-31

### events crate
- `NyxEvent<T>` envelope with `{ id, subject, app, timestamp, payload: T }` compiles and serializes correctly.
- Subject constants: `UZUME_STORY_CREATED`, `UZUME_STORY_VIEWED`, `UZUME_MEDIA_UPLOADED`, `UZUME_MEDIA_PROCESSED` follow `{app}.{entity}.{action}` convention.
- `MediaUploadedPayload` carries `job_id` for idempotency, `entity_type`, `entity_id`, `source_path`, `mime_type`, `size_bytes`.
- `MediaProcessedPayload` carries `job_id` (correlation), `variants` HashMap, `processing_ms`.
- `NatsClient` wraps `async_nats::Client` + `jetstream::Context`, implements `Clone` for sharing between publisher/subscriber.
- `Publisher` is app-scoped at construction time — all published events carry the correct app identifier.
- `Subscriber` uses `async_stream::stream!` macro to convert NATS subscriber into a typed `EventStream<T>`.
- `async-nats` 0.39 `publish()` requires `impl Into<Bytes>` for subject — needs owned `String`, not `&str`.
- `async-nats` 0.39 `subscribe()` also requires owned `String` for subject.
- 13 tests pass: envelope serialization, subject constants, payload serialization, event parsing (valid/invalid/wrong type).

### Oya crate
- `fast_image_resize` 3.x API differs significantly from 5.x: uses `NonZeroU32` for dimensions, `Image::from_vec_u8`/`from_slice_u8`, `Resizer::new(algorithm)` takes algorithm in constructor, `resize()` takes `DynamicImageView`/`DynamicImageViewMut` via `.view()`/`.view_mut()` methods.
- `image` 0.24 API: `GenericImageView` trait must be imported for `.dimensions()` method on `DynamicImage`.
- `ProcessingConfig::default()` provides story entity with 4 image variants (1080, 640, 320, 150px) and 3 video variants (720p, 480p, 360p).
- `MediaPipeline` validates MIME type against allowlist and file size against max before processing.
- `VideoError::FfmpegNotFound` returned when ffmpeg binary is absent — worker fails fast.
- Worker binary (`oya-worker`) subscribes to `Uzume.media.uploaded`, processes media, emits `Uzume.media.processed`.
- Idempotency: worker tracks processed `job_id`s in a `HashSet<uuid::Uuid>` to skip duplicates.
- 18 tests pass: config defaults, image decode/resize/encode, pipeline validation, video error handling.

### Dependency versions for rustc 1.94
- `image = "0.24"` (0.25 requires rustc 1.88)
- `fast_image_resize = "3"` (5.x requires rustc 1.87)
- `async-nats = "0.39"` (workspace)
- `async-stream = "0.3"` (for subscriber stream generation)
