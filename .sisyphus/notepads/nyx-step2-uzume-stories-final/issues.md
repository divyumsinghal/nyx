
### Async Media Pipeline Risks
- **External Risk**: While pure Rust media processing libraries (like `OxiMedia` / `avio`) exist in 2026, they are still relatively new. Using mature C-bindings (FFmpeg) carries a higher memory-safety vulnerability risk from untrusted input, but using pure-Rust tools might lack certain esoteric codec support. We need to decide on our dependency footprint for transcoding vs. simple media validation.
- **External Risk**: Deferred provider finalization implies that our state machine must handle "Processing" -> "Failed" cleanly without locking the user's data or resources. This opens a potential vector for Denial of Wallet / Resource Exhaustion if malicious users upload files that intentionally cause processing timeouts. Strict timeouts must be enforced.

## Step 2 Task 1 / Step 2 Issues (2026-03-31)
- Environment issue: `just` is not installed locally in this runner (`zsh: command not found: just`), so direct `just gate-step1-compat` execution could not be run here; equivalent gate commands were executed directly via shell.
- Workspace issue: root `Cargo.toml` currently defines `[package]` without a root target, so direct `cargo test -p nun --lib` and `cargo check --workspace` from repo root fail manifest parsing in this environment. Contract guardrails were validated via the dedicated script gate path.
