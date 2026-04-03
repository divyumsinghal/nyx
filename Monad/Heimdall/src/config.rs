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
//! | `JWT_PRIVATE_KEY_PEM` | RSA private key PEM used to sign JWTs |
//! | `JWT_PUBLIC_KEY_PEM` | RSA public key PEM used to verify JWTs |
//! | `DATABASE_URL` | PostgreSQL connection string |
//! | `KRATOS_PUBLIC_URL` | Ory Kratos public API base URL |
//! | `MATRIX_URL` | Continuwuity (Matrix) base URL |
//! | `UZUME_PROFILES_URL` | Uzume-profiles service base URL |
//! | `UZUME_FEED_URL` | Uzume-feed service base URL |
//! | `UZUME_STORIES_URL` | Uzume-stories service base URL |
//! | `UZUME_REELS_URL` | Uzume-reels service base URL |
//! | `UZUME_DISCOVER_URL` | Uzume-discover service base URL |
//! | `CORS_ALLOWED_ORIGINS` | Comma-separated list of allowed CORS origins |
//!
//! # Optional environment variables (with defaults)
//!
//! | Variable | Default | Description |
//! |---|---|---|
//! | `HEIMDALL_PORT` | `3000` | TCP port to listen on |
//! | `HEIMDALL_HOST` | `0.0.0.0` | Bind address |
//! | `JWT_EXPIRY_SECS` | `3600` | JWT lifetime in seconds |
//! | `NYX_ENVIRONMENT` | `development` | Environment (development/production) |
//!
//! Heimdall is designed for edge TLS termination (for example Caddy). It serves
//! plain HTTP behind the edge proxy.

use anyhow::{Context, Result};
use reqwest::Url;

/// Runtime configuration for Heimdall.
#[derive(Debug, Clone)]
pub struct HeimdallConfig {
    pub port: u16,
    pub host: String,
    pub jwt_secret: String,
    pub jwt_private_key_pem: Option<String>,
    pub jwt_public_key_pem: Option<String>,
    pub jwt_expiry_secs: u64,
    pub database_url: String,
    pub kratos_public_url: String,
    pub matrix_url: String,
    pub uzume_profiles_url: String,
    pub uzume_feed_url: String,
    pub uzume_stories_url: String,
    pub uzume_reels_url: String,
    pub uzume_discover_url: String,
    pub cors_allowed_origins: Vec<String>,
    pub environment: String,
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
        // let jwt_private_key_pem = std::env::var("JWT_PRIVATE_KEY_PEM").ok();
        // let jwt_public_key_pem = std::env::var("JWT_PUBLIC_KEY_PEM").ok();

        let port = std::env::var("HEIMDALL_PORT")
            .unwrap_or_else(|_| "3000".to_owned())
            .parse::<u16>()
            .context("HEIMDALL_PORT must be a valid port number (0–65535)")?;

        let host = std::env::var("HEIMDALL_HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());

        let jwt_expiry_secs = std::env::var("JWT_EXPIRY_SECS")
            .unwrap_or_else(|_| "3600".to_owned())
            .parse::<u64>()
            .context("JWT_EXPIRY_SECS must be a non-negative integer")?;

        let environment =
            std::env::var("NYX_ENVIRONMENT").unwrap_or_else(|_| "development".to_owned());

        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable is required")?;
        let jwt_private_key_pem = std::env::var("JWT_PRIVATE_KEY_PEM").ok();
        let jwt_public_key_pem = std::env::var("JWT_PUBLIC_KEY_PEM").ok();

        if environment == "production"
            && (jwt_private_key_pem.is_none() || jwt_public_key_pem.is_none())
        {
            anyhow::bail!("JWT_PRIVATE_KEY_PEM and JWT_PUBLIC_KEY_PEM are required in production");
        }

        let kratos_public_url = require_url("KRATOS_PUBLIC_URL")?;
        let matrix_url = require_url("MATRIX_URL")?;
        let uzume_profiles_url = require_url("UZUME_PROFILES_URL")?;
        let uzume_feed_url = require_url("UZUME_FEED_URL")?;
        let uzume_stories_url = require_url("UZUME_STORIES_URL")?;
        let uzume_reels_url = require_url("UZUME_REELS_URL")?;
        let uzume_discover_url = require_url("UZUME_DISCOVER_URL")?;
        let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .context("CORS_ALLOWED_ORIGINS environment variable is required")?;
        let cors_allowed_origins = parse_allowed_origins(&cors_allowed_origins)?;

        Ok(Self {
            port,
            host,
            jwt_secret,
            jwt_private_key_pem,
            jwt_public_key_pem,
            jwt_expiry_secs,
            database_url,
            kratos_public_url,
            matrix_url,
            uzume_profiles_url,
            uzume_feed_url,
            uzume_stories_url,
            uzume_reels_url,
            uzume_discover_url,
            cors_allowed_origins,
            environment,
        })
    }

    /// Returns `true` if running in development mode.
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }

    /// Returns `true` if running in production mode.
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}

fn parse_allowed_origins(raw: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for origin in raw.split(',').map(str::trim).filter(|v| !v.is_empty()) {
        let parsed = Url::parse(origin)
            .with_context(|| format!("Invalid origin in CORS_ALLOWED_ORIGINS: {origin}"))?;
        match parsed.scheme() {
            "http" | "https" => out.push(parsed.as_str().trim_end_matches('/').to_owned()),
            scheme => anyhow::bail!(
                "CORS_ALLOWED_ORIGINS contains unsupported scheme '{scheme}' for origin {origin}"
            ),
        }
    }

    if out.is_empty() {
        anyhow::bail!("CORS_ALLOWED_ORIGINS must contain at least one origin");
    }

    Ok(out)
}

/// Read a required URL env var and strip trailing slashes.
fn require_url(var: &str) -> Result<String> {
    let raw =
        std::env::var(var).with_context(|| format!("{var} environment variable is required"))?;
    let parsed =
        Url::parse(raw.trim()).with_context(|| format!("{var} must be a valid absolute URL"))?;

    match parsed.scheme() {
        "http" | "https" => Ok(parsed.as_str().trim_end_matches('/').to_owned()),
        scheme => anyhow::bail!("{var} must use http or https scheme, got: {scheme}"),
    }
}
