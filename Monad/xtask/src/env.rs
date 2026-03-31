//! Environment variable helpers for the `nyx-xtask` CLI.
#![warn(clippy::pedantic)]

/// Read a required environment variable, returning a descriptive error if absent.
///
/// # Errors
///
/// Returns [`anyhow::Error`] when the variable is not set in the process environment.
///
/// # Examples
///
/// ```no_run
/// let url = nyx_xtask::env::require("DATABASE_URL").unwrap();
/// ```
pub fn require(key: &str) -> anyhow::Result<String> {
    std::env::var(key)
        .map_err(|_| anyhow::anyhow!("Required environment variable `{key}` is not set"))
}
