//! Config loading tests — Cycle 2 (TDD: written before implementation).
//!
//! Each test sets environment variables explicitly and calls `from_env()`.
//! Tests are run sequentially via a mutex to avoid env-var races.

use std::sync::Mutex;

use heimdall::config::HeimdallConfig;

/// Global mutex to serialize env-var-touching tests.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn set_required_env(jwt_secret: &str) {
    unsafe {
        std::env::set_var("JWT_SECRET", jwt_secret);
        std::env::set_var("KRATOS_PUBLIC_URL", "http://kratos:4433");
        std::env::set_var("MATRIX_URL", "http://matrix:8448");
        std::env::set_var("UZUME_PROFILES_URL", "http://profiles:3001");
        std::env::set_var("UZUME_FEED_URL", "http://feed:3002");
        std::env::set_var("UZUME_STORIES_URL", "http://stories:3003");
        std::env::set_var("UZUME_REELS_URL", "http://reels:3004");
        std::env::set_var("UZUME_DISCOVER_URL", "http://discover:3005");
    }
}

fn clear_env() {
    unsafe {
        for key in &[
            "JWT_SECRET",
            "JWT_EXPIRY_SECS",
            "HEIMDALL_PORT",
            "HEIMDALL_HOST",
            "KRATOS_PUBLIC_URL",
            "MATRIX_URL",
            "UZUME_PROFILES_URL",
            "UZUME_FEED_URL",
            "UZUME_STORIES_URL",
            "UZUME_REELS_URL",
            "UZUME_DISCOVER_URL",
        ] {
            std::env::remove_var(key);
        }
    }
}

// 1. All required vars set → Ok(config).
#[test]
fn test_all_required_vars_set_returns_ok() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_env();
    set_required_env("my-super-secret-jwt-key-32chars!!");

    let config = HeimdallConfig::from_env().expect("should succeed with all vars set");
    assert_eq!(config.jwt_secret, "my-super-secret-jwt-key-32chars!!");
    assert_eq!(config.kratos_public_url, "http://kratos:4433");
}

// 2. JWT_SECRET missing → Err.
#[test]
fn test_missing_jwt_secret_returns_err() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_env();
    set_required_env("placeholder");
    unsafe {
        std::env::remove_var("JWT_SECRET");
    }

    let result = HeimdallConfig::from_env();
    assert!(result.is_err(), "missing JWT_SECRET must return Err");
}

// 3. Upstream URLs have trailing slash stripped.
#[test]
fn test_trailing_slashes_stripped_from_urls() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_env();
    set_required_env("secret-key-at-least-32-characters!!");
    unsafe {
        std::env::set_var("KRATOS_PUBLIC_URL", "http://kratos:4433/");
        std::env::set_var("UZUME_PROFILES_URL", "http://profiles:3001///");
    }

    let config = HeimdallConfig::from_env().expect("should succeed");
    assert_eq!(config.kratos_public_url, "http://kratos:4433");
    assert_eq!(config.uzume_profiles_url, "http://profiles:3001");
}

// 4. HEIMDALL_PORT defaults to 3000 when unset.
#[test]
fn test_port_defaults_to_3000() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_env();
    set_required_env("secret-key-at-least-32-characters!!");
    unsafe {
        std::env::remove_var("HEIMDALL_PORT");
    }

    let config = HeimdallConfig::from_env().expect("should succeed");
    assert_eq!(config.port, 3000);
}

// 5. HEIMDALL_PORT can be overridden.
#[test]
fn test_port_can_be_overridden() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_env();
    set_required_env("secret-key-at-least-32-characters!!");
    unsafe {
        std::env::set_var("HEIMDALL_PORT", "8080");
    }

    let config = HeimdallConfig::from_env().expect("should succeed");
    assert_eq!(config.port, 8080);
}

// 6. JWT_EXPIRY_SECS defaults to 3600 when unset.
#[test]
fn test_jwt_expiry_defaults_to_3600() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_env();
    set_required_env("secret-key-at-least-32-characters!!");
    unsafe {
        std::env::remove_var("JWT_EXPIRY_SECS");
    }

    let config = HeimdallConfig::from_env().expect("should succeed");
    assert_eq!(config.jwt_expiry_secs, 3600);
}
