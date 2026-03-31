
## API Privacy & Authorization Testing Policy

**1. Security Test Matrix Mandatory Checklist:**
Every authenticated endpoint MUST have at minimum the following tests:
- `test_unauthorized_401`: Request with missing/invalid token. Expect `401 Unauthorized`.
- `test_forbidden_403`: Request with valid token but wrong user/owner. Expect `403 Forbidden` (or 404 if hiding existence).
- `test_data_minimization`: Assert the successful JSON schema strictly excludes internal fields, internal IDs, and other users' private data (BOPLA prevention).
- `test_anti_enumeration`: Assert identical HTTP status codes for unauthorized resource access and non-existent resource access to prevent existence leakage.

**2. Strict Response Data Transfer Objects (DTOs):**
- **Decision:** No implicit serialization of database models. We will enforce strict separation of internal models (`StoryModel`) and response DTOs (`StoryResponse`).
- **Rationale:** Prevent accidental Broken Object Property Level Authorization (API3:2023) and data leakage.

**3. Anti-Enumeration and 401/403/404 Rules:**
- **Decision:** Return `401` only for token issues. Return `403` for known users without permission. Where resource existence is sensitive, return `404` instead of `403` for unauthorized users to prevent enumeration.

## Monad Crate Architectural Decisions for Stories

### 1. Stories ID System Strategy
- **Decision**: Use Nun's `Id<T>` pattern for all Stories entity types
- **Rationale**: Compile-time type safety prevents mixing StoryId with HighlightId
- **Implementation**: Define markers in `Monad/Nun/src/types/` or extend in Uzume-stories
- **Test Verification**: Type system prevents accidental ID mixing at compile time

### 2. Storage Path Convention
- **Decision**: Use Akash path convention `{app}/stories/{id}/{variant}.{ext}`
- **Rationale**: Consistent with existing pattern (posts, avatars, reels)
- **Variants needed**: `original`, `1080`, `640`, `320` for images; `original`, `hls/master.m3u8` for videos
- **Code Location**: `Monad/Akash/src/paths.rs`

### 3. Async Media Processing Flow
- **Decision**: Use event-driven pattern per Oya design: upload -> emit event -> process -> emit ready
- **Rationale**: Non-blocking upload, parallel processing, retry-safe via NATS at-least-once
- **Events needed**: `Uzume.media.uploaded` -> Oya worker -> `Uzume.media.processed` -> stories service updates
- **Idempotency**: Use job_id UUID to deduplicate duplicate event deliveries

### 4. Cache Key Strategy
- **Decision**: Use Lethe's `{app}:{entity}:{id}` pattern for stories
- **Keys needed**:
  - `Uzume:stories:feed:{user_id}` - home stories feed (TTL: 5 min)
  - `Uzume:story:{id}` - single story cache (TTL: 1 min)
  - `Uzume:story:{id}:viewers` - viewer list cache (TTL: 1 min)
- **Invalidation**: On story creation/deletion/expire

### 5. Database Schema Strategy
- **Decision**: Use Mnemosyne pattern with Uzume schema
- **Migrations location**: `migrations/Uzume/0003_stories.up.sql`
- **Tables needed**: stories, story_views, story_interactions, highlights, highlight_items
- **Key pattern**: Use cursor pagination with ID sort (time-sortable via UUIDv7)

### 6. API Middleware Strategy
- **Decision**: Reuse existing nyx-api middleware (auth, rate_limit, request_id)
- **Additions needed**: 
  - File upload size limits in config
  - Request timeout for long-running media uploads

### 7. Pagination Strategy
- **Decision**: Use Nun's `PageResponse::from_overflowed` with timestamp_id cursors
- **Rationale**: ID-based cursor works with UUIDv7 for time-sortable stories
- **Feed query**: Fetch stories from followed users with `created_at` ordering

### 8. Testing Strategy
- **Decision**: Follow Nun's testing patterns (unit tests in same file, integration via testcontainers)
- **Test location**: `tests/` subdirectory per crate
- **Security tests**: Must include BOLA, BOPLA, enumeration protection per decisions.md

## Key Files to Create (Implementation Map)

### Monad/Akash (storage):
- `src/stories.rs` - Stories path builders, media URL generation
- `src/lib.rs` - re-export stories modules

### Monad/events:
- `src/subjects.rs` - Add `Uzume_STORY_CREATED`, `Uzume_STORY_VIEWED`, etc.
- `src/payloads/stories.rs` - Typed event payloads for stories

### Monad/Lethe:
- `src/stories.rs` - Stories cache keys, TTL constants

### Monad/Mnemosyne:
- `src/stories_helpers.rs` - Repository helpers for stories queries

### Monad/api:
- `src/middleware/upload_limits.rs` - File size limits for stories

### migrations/Uzume:
- `0003_stories.up.sql` - Stories tables
- `0003_stories.down.sql` - Rollback

## Explicit Test Commands for Quick Verification

```bash
# Verify Nun foundation tests pass
cargo test -p nun --lib -- --nocapture

# Check Akash module structure (will show empty if not implemented)
ls -la Monad/Akash/src/ 2>/dev/null || echo "Akash src/ not implemented"

# Check events subjects (will show empty if not implemented)
ls -la Monad/events/src/ 2>/dev/null || echo "events src/ not implemented"

# Check Lethe module structure
ls -la Monad/Lethe/src/ 2>/dev/null || echo "Lethe src/ not implemented"

# List current workspace crates
cargo metadata --format-version=1 --no-deps 2>/dev/null | jq -r '.packages[] | select(.name | contains("stories") or contains("akash") or contains("oya") or contains("lethe") or contains("events") or contains("mnemosyne")) | .name'
```

## Idempotency Patterns Already in Codebase

- **Nun Id<T>**: UUIDv7 provides built-in idempotency via timestamp component
- **Cursor pagination**: Opaque cursor prevents manipulation
- **Error handling**: Named constructors produce deterministic errors
- **Event envelope**: Contains unique message ID for deduplication

## Anti-Patterns to Avoid

1. **Don't put domain logic in shared crates** - Lethe, Akash, events are infra, not domain
2. **Don't bypass Nun** - Always use Nun types for IDs, errors, pagination
3. **Don't use offset pagination** - Must use cursor-based per platform convention
4. **Don't hardcode cache keys** - Use Lethe's key builder pattern
5. **Don't skip event-driven** - Media processing must be async per plan
