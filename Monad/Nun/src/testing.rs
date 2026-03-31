//! Test utilities for the Nyx platform.
//!
//! Available when `cfg(test)` or the `test` feature is enabled.
//! Provides random ID generators, pre-built test configurations, and
//! assertion helpers for error kinds.
//!
//! # Usage
//!
//! ```rust,ignore
//! use nun::testing::{test_id, test_config};
//!
//! let post_id: Id<Post> = test_id();
//! let config = test_config();
//! ```

use uuid::Uuid;

use crate::config::{
    AuthConfig, CacheConfig, DatabaseConfig, Environment, MessagingConfig, NatsConfig, NyxConfig,
    SearchConfig, ServerConfig, StorageConfig,
};
use crate::error::{ErrorKind, NyxError};
use crate::id::Id;
use crate::sensitive::Sensitive;

/// Generate a random typed ID. Convenience wrapper around `Id::new()`.
pub fn test_id<T>() -> Id<T> {
    Id::new()
}

/// Generate a typed ID from a specific UUID. Useful for deterministic tests.
pub fn id_from_uuid<T>(uuid: Uuid) -> Id<T> {
    Id::from_uuid(uuid)
}

/// A pre-built configuration for tests, pointing to localhost defaults.
///
/// All URLs use localhost with standard ports. Secrets are dummy values.
/// This config is sufficient for unit tests (no real infrastructure needed)
/// and can be used as a base for integration tests that override specific values.
pub fn test_config() -> NyxConfig {
    NyxConfig {
        environment: Environment::Development,
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            request_timeout_secs: 5,
        },
        database: DatabaseConfig {
            url: Sensitive::new("postgres://nyx:nyx@localhost:5432/nyx_test".to_string()),
            max_connections: 2,
            min_connections: 1,
            acquire_timeout_secs: 2,
            idle_timeout_secs: 60,
        },
        cache: CacheConfig {
            url: Sensitive::new("redis://localhost:6379".to_string()),
            pool_size: 2,
        },
        nats: NatsConfig {
            url: "nats://localhost:4222".to_string(),
        },
        storage: StorageConfig {
            endpoint: "http://localhost:9000".to_string(),
            region: "us-east-1".to_string(),
            bucket: "nyx-test".to_string(),
            access_key: Sensitive::new("minioadmin".to_string()),
            secret_key: Sensitive::new("minioadmin".to_string()),
        },
        search: SearchConfig {
            url: "http://localhost:7700".to_string(),
            api_key: Sensitive::new("test-master-key".to_string()),
        },
        auth: AuthConfig {
            public_url: "http://localhost:4433".to_string(),
            admin_url: "http://localhost:4434".to_string(),
        },
        messaging: MessagingConfig {
            homeserver_url: "http://localhost:6167".to_string(),
            server_name: "nyx.test".to_string(),
        },
    }
}

/// A test server config with a specific port.
pub fn test_server_config(port: u16) -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        request_timeout_secs: 5,
    }
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Assert that a `Result` is an `Err` with a specific [`ErrorKind`].
///
/// ```rust,ignore
/// let result = find_post("nonexistent");
/// assert_error_kind(&result, ErrorKind::NotFound);
/// ```
pub fn assert_error_kind<T: std::fmt::Debug>(result: &Result<T, NyxError>, expected: ErrorKind) {
    match result {
        Ok(val) => panic!("expected error {expected:?}, got Ok({val:?})"),
        Err(err) => assert_eq!(
            err.kind(),
            expected,
            "expected {expected:?}, got {:?} (code: {}, message: {})",
            err.kind(),
            err.code(),
            err.message(),
        ),
    }
}

/// Assert that a `Result` is an `Err` with a specific error code.
pub fn assert_error_code<T: std::fmt::Debug>(result: &Result<T, NyxError>, expected_code: &str) {
    match result {
        Ok(val) => panic!("expected error code {expected_code:?}, got Ok({val:?})"),
        Err(err) => assert_eq!(
            err.code(),
            expected_code,
            "expected code {expected_code:?}, got {:?}",
            err.code(),
        ),
    }
}

/// Assert that a `Result` is `Ok`.
pub fn assert_ok<T, E: std::fmt::Debug>(result: &Result<T, E>) {
    if let Err(ref err) = result {
        panic!("expected Ok, got Err({err:?})");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeEntity;

    #[test]
    fn test_id_generates_unique_ids() {
        let a: Id<FakeEntity> = test_id();
        let b: Id<FakeEntity> = test_id();
        assert_ne!(a, b);
    }

    #[test]
    fn test_config_is_valid() {
        let config = test_config();
        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.server.port, 3000);
        assert!(config.is_development());
    }

    #[test]
    fn assert_error_kind_passes_on_match() {
        let result: Result<(), NyxError> = Err(NyxError::not_found("x", "y"));
        assert_error_kind(&result, ErrorKind::NotFound);
    }

    #[test]
    #[should_panic(expected = "expected")]
    fn assert_error_kind_panics_on_mismatch() {
        let result: Result<(), NyxError> = Err(NyxError::not_found("x", "y"));
        assert_error_kind(&result, ErrorKind::Unauthorized);
    }

    #[test]
    fn assert_error_code_passes_on_match() {
        let result: Result<(), NyxError> = Err(NyxError::not_found("post_not_found", "y"));
        assert_error_code(&result, "post_not_found");
    }
}
