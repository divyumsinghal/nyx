//! Service-specific configuration for `uzume-stories`.
//!
//! Port is 3003, set via `NYX_SERVER__PORT=3003` environment variable.
//! All other configuration comes from the shared [`NyxConfig`].

use nun::config::NyxConfig;

/// Load and return the platform configuration.
///
/// Reads from (in priority order):
/// 1. Environment variables prefixed with `NYX_`
/// 2. `config.development.toml` / `config.production.toml`
/// 3. `config.toml`
pub fn load() -> nun::Result<NyxConfig> {
    NyxConfig::load()
}
