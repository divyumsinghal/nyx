
### Async Media Pipeline Risks
- **External Risk**: While pure Rust media processing libraries (like `OxiMedia` / `avio`) exist in 2026, they are still relatively new. Using mature C-bindings (FFmpeg) carries a higher memory-safety vulnerability risk from untrusted input, but using pure-Rust tools might lack certain esoteric codec support. We need to decide on our dependency footprint for transcoding vs. simple media validation.
- **External Risk**: Deferred provider finalization implies that our state machine must handle "Processing" -> "Failed" cleanly without locking the user's data or resources. This opens a potential vector for Denial of Wallet / Resource Exhaustion if malicious users upload files that intentionally cause processing timeouts. Strict timeouts must be enforced.

## Step 2 Task 1 / Step 2 Issues (2026-03-31)
- Environment issue: `just` is not installed locally in this runner (`zsh: command not found: just`), so direct `just gate-step1-compat` execution could not be run here; equivalent gate commands were executed directly via shell.
- Workspace issue: root `Cargo.toml` currently defines `[package]` without a root target, so direct `cargo test -p nun --lib` and `cargo check --workspace` from repo root fail manifest parsing in this environment. Contract guardrails were validated via the dedicated script gate path.

### Pre-signed URL Implementation Risks & Gotchas
*   **Missing Native POST Policy Support in Rust:** `aws-sdk-s3` in Rust lacks a built-in `generate_presigned_post` builder (tracked in open Issue #863). Workarounds require using third-party crates (e.g., `presigned-post-rs`), writing raw `aws-sigv4` generation, or relying on strict exact-match `PUT` URLs.
*   **PUT Content-Length-Range Limitation:** S3 PUT URLs cannot natively enforce a `content-length-range` via URL signature. To enforce a max size with a PUT URL, the client must declare the *exact* file size upfront, and the backend must validate it against the max size limit before signing the exact `content_length` into the URL.
*   **Replay / PUT Reuse Vulnerability:** Standard S3 presigned URLs do not inherently enforce "single-use". An attacker intercepting the URL within its TTL window can re-upload over the same key. Mitigations involve extremely short TTLs and/or tracking state via a `/finalize` endpoint.
*   **Client Integration Brittleness:** When locking headers (like `Content-Type`, `Content-MD5`, and `Content-Length`) into a presigned PUT URL, frontend implementation becomes fragile. If the browser `fetch()` API alters or omits the exact headers (e.g., adding unexpected boundaries), S3 will abruptly reject the upload with `SignatureDoesNotMatch`.

## Step 2 Task 2: Storage/Object Lifecycle/Presign Risks & Gaps (2026-03-31)

### External Risks

1. **rust-s3 Library Maturity**: The `rust-s3` crate (version 0.36) is used in workspace dependencies. Need to verify it supports all S3-compatible operations (presigned URLs, multipart uploads) needed for stories media. Alternative: `aws-sdk-s3` (more mature but heavier).

2. **Presigned URL Expiration Races**: If presigned URL expires before client completes upload, need clear error handling and retry flow. Consider short expiration (60s) + client-side retry.

3. **MinIO vs S3 Feature Parity**: If using MinIO for development, verify presigned URL behavior matches AWS S3 exactly. Some MinIO versions havequirks with headers.

### Implementation Gaps

1. **Akash Crate Not Implemented**: Only README.md exists, no src/ files. Full implementation needed:
   - client.rs
   - paths.rs
   - upload.rs
   - download.rs

2. **Media Validation Not Implemented**: Nun has general validators (alias, email, phone) but no media-specific validators for MIME type whitelist and size limits.

3. **Stories ID Markers**: Need to define `StoryId`, `HighlightId` markers in Nun using `Id<T>` pattern.

4. **Media Status Enum Missing**: No `MediaStatus` enum for Accepted→Processing→Ready state tracking in database.

5. **Path Builder Type Safety**: Current path convention is string-based. Consider type-safe builders to prevent invalid paths.

### Security Considerations

1. **Path Traversal Prevention**: Ensure paths constructed via `{story_id}` variable cannot escape the `Uzume/stories/` prefix.

2. **Object Overwrite**: Stories should use UUID-based keys to prevent overwrite attacks. Must reject client-provided filenames.

3. **Upload Quarantine**: Consider quarantine pattern for untrusted uploads before finalization.

4. **Download XSS Prevention**: Ensure presigned download URLs set proper Content-Type headers to prevent inline HTML/SVG XSS.

### Testing Gaps

1. **No Akash Tests**: No test files exist in Monad/Akash/

2. **Presign Validation**: Need comprehensive tests for:
   - Valid MIME types accepted
   - Invalid MIME types rejected
   - Size limit enforcement
   - Expired URL rejection
   - Path traversal blocked

3. **Integration with Oya**: Need integration tests for upload → event → process → ready flow.

### Dependency Notes

- From workspace Cargo.toml: `rust-s3 = { version = "0.36", features = ["tokio-rustls-tls"] }`
- Akash depends on Nun (for error types, config, ID system)
- Akash will be used by Uzume-stories for media upload/download
- Akash will be used by Oya for storing processed variants
