
### Async Media Pipeline Risks
- **External Risk**: While pure Rust media processing libraries (like `OxiMedia` / `avio`) exist in 2026, they are still relatively new. Using mature C-bindings (FFmpeg) carries a higher memory-safety vulnerability risk from untrusted input, but using pure-Rust tools might lack certain esoteric codec support. We need to decide on our dependency footprint for transcoding vs. simple media validation.
- **External Risk**: Deferred provider finalization implies that our state machine must handle "Processing" -> "Failed" cleanly without locking the user's data or resources. This opens a potential vector for Denial of Wallet / Resource Exhaustion if malicious users upload files that intentionally cause processing timeouts. Strict timeouts must be enforced.
