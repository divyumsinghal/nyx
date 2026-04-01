//! Service-specific configuration for `uzume-profiles`.
//!
//! The canonical configuration source is [`NyxConfig`] from Nun. This module
//! adds any fields that are specific to this service (none right now — the
//! service uses the standard port 3001 via `NYX_SERVER__PORT=3001`).

use nun::config::NyxConfig;

/// Load and return the platform config.
///
/// Reads from (in priority order):
/// 1. Environment variables prefixed with `NYX_`
/// 2. `config.development.toml` / `config.production.toml`
/// 3. `config.toml`
pub fn load() -> nun::Result<NyxConfig> {
    NyxConfig::load()
}
