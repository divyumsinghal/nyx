//! Heimdall configuration loaded from environment variables.
//!
//! All upstream base URLs are stored without trailing slashes. Call
//! [`HeimdallConfig::from_env`] once at startup; the resulting struct is
//! `Clone` so it can be shared via `AppState`.
//!
//! # Required environment variables
//!
//! | Variable | Description |
//! |---|---|
//! | `JWT_SECRET` | Shared HMAC-SHA256 secret used to sign/verify JWTs |
//! | `KRATOS_PUBLIC_URL` | Ory Kratos public API base URL |
//! | `MATRIX_URL` | Continuwuity (Matrix) base URL |
//! | `UZUME_PROFILES_URL` | Uzume-profiles service base URL |
//! | `UZUME_FEED_URL` | Uzume-feed service base URL |
//! | `UZUME_STORIES_URL` | Uzume-stories service base URL |
//! | `UZUME_REELS_URL` | Uzume-reels service base URL |
//! | `UZUME_DISCOVER_URL` | Uzume-discover service base URL |
//!
//! # Optional environment variables (with defaults)
//!
//! | Variable | Default | Description |
//! |---|---|---|
//! | `HEIMDALL_PORT` | `3000` | TCP port to listen on |
//! | `HEIMDALL_HOST` | `0.0.0.0` | Bind address |
//! | `JWT_EXPIRY_SECS` | `3600` | JWT lifetime in seconds |

use anyhow::{Context, Result};

/// Runtime configuration for Heimdall.
#[derive(Debug, Clone)]
pub struct HeimdallConfig {
    /// TCP port to bind (`HEIMDALL_PORT`, default `3000`).
    pub port: u16,
    /// Bind address (`HEIMDALL_HOST`, default `"0.0.0.0"`).
    pub host: String,
    /// HMAC secret for JWT signing/verification (`JWT_SECRET`, **required**).
    pub jwt_secret: String,
    /// JWT validity window in seconds (`JWT_EXPIRY_SECS`, default `3600`).
    pub jwt_expiry_secs: u64,
    /// Ory Kratos public API base URL, no trailing slash.
    pub kratos_public_url: String,
    /// Continuwuity (Matrix) base URL, no trailing slash.
    pub matrix_url: String,
    /// Uzume-profiles base URL, no trailing slash.
    pub uzume_profiles_url: String,
    /// Uzume-feed base URL, no trailing slash.
    pub uzume_feed_url: String,
    /// Uzume-stories base URL, no trailing slash.
    pub uzume_stories_url: String,
    /// Uzume-reels base URL, no trailing slash.
    pub uzume_reels_url: String,
    /// Uzume-discover base URL, no trailing slash.
    pub uzume_discover_url: String,
}

impl HeimdallConfig {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if any required variable is absent or if a value
    /// cannot be parsed into its target type.
    pub fn from_env() -> Result<Self> {
        let jwt_secret =
            std::env::var("JWT_SECRET").context("JWT_SECRET environment variable is required")?;

        let port = std::env::var("HEIMDALL_PORT")
            .unwrap_or_else(|_| "3000".to_owned())
            .parse::<u16>()
            .context("HEIMDALL_PORT must be a valid port number (0–65535)")?;

        let host =
            std::env::var("HEIMDALL_HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());

        let jwt_expiry_secs = std::env::var("JWT_EXPIRY_SECS")
            .unwrap_or_else(|_| "3600".to_owned())
            .parse::<u64>()
            .context("JWT_EXPIRY_SECS must be a non-negative integer")?;

        let kratos_public_url = require_url("KRATOS_PUBLIC_URL")?;
        let matrix_url = require_url("MATRIX_URL")?;
        let uzume_profiles_url = require_url("UZUME_PROFILES_URL")?;
        let uzume_feed_url = require_url("UZUME_FEED_URL")?;
        let uzume_stories_url = require_url("UZUME_STORIES_URL")?;
        let uzume_reels_url = require_url("UZUME_REELS_URL")?;
        let uzume_discover_url = require_url("UZUME_DISCOVER_URL")?;

        Ok(Self {
            port,
            host,
            jwt_secret,
            jwt_expiry_secs,
            kratos_public_url,
            matrix_url,
            uzume_profiles_url,
            uzume_feed_url,
            uzume_stories_url,
            uzume_reels_url,
            uzume_discover_url,
        })
    }
}

/// Read a required URL env var and strip trailing slashes.
fn require_url(var: &str) -> Result<String> {
    let raw = std::env::var(var).with_context(|| format!("{var} environment variable is required"))?;
    Ok(raw.trim_end_matches('/').to_owned())
}
