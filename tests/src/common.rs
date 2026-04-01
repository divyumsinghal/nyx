//! Common test utilities and helpers.

use std::sync::Once;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static INIT: Once = Once::new();

/// Initialize tracing for tests. Safe to call multiple times.
///
/// Uses `RUST_LOG` env var if set, otherwise defaults to `info`.
/// Call this at the start of integration tests that need logging.
pub fn init_test_tracing() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();

        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().with_test_writer())
            .init();
    });
}

/// Sleep for a short duration in tests. Useful for waiting for async operations.
pub async fn test_sleep_ms(ms: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
}

/// Generate a random string of specified length (alphanumeric).
pub fn random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a random email address for testing.
pub fn random_email() -> String {
    format!("test-{}@example.com", random_string(10).to_lowercase())
}

/// Generate a random phone number (E.164 format).
pub fn random_phone() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("+1555{:07}", rng.gen_range(1_000_000..10_000_000))
}

/// Generate a random username (lowercase alphanumeric + underscore).
pub fn random_username() -> String {
    format!("user_{}", random_string(8).to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_string_generates_correct_length() {
        let s = random_string(10);
        assert_eq!(s.len(), 10);
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn random_email_is_valid_format() {
        let email = random_email();
        assert!(email.contains('@'));
        assert!(email.ends_with("@example.com"));
    }

    #[test]
    fn random_phone_is_e164_format() {
        let phone = random_phone();
        assert!(phone.starts_with("+1555"));
        assert_eq!(phone.len(), 12);
    }

    #[test]
    fn random_username_is_valid() {
        let username = random_username();
        assert!(username.starts_with("user_"));
        assert!(username.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
    }
}
