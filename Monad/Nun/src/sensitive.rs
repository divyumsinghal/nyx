//! A wrapper type that redacts its contents in [`Debug`] and [`Display`] output.
//!
//! Use [`Sensitive`] for any value that must never appear in logs, error messages,
//! or debug output: passwords, API keys, connection strings with credentials,
//! session tokens.
//!
//! ```rust,ignore
//! use nun::Sensitive;
//!
//! let api_key = Sensitive::new("sk-abc123".to_string());
//! println!("{api_key:?}");  // prints: [REDACTED]
//!
//! // Explicit access when you actually need the value:
//! let raw: &str = api_key.expose();
//! ```
//!
//! # No `Deref`
//!
//! `Sensitive<T>` deliberately does **not** implement [`Deref`]. This forces
//! callers to use [`.expose()`](Sensitive::expose), making secret access visible
//! in code review and `grep`-able in the codebase.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A value that is redacted in `Debug` and `Display` output.
///
/// # Serialization
///
/// `Sensitive<T>` is transparent for `Deserialize` (reads the inner value
/// normally from config files / env vars). `Serialize` is **not** implemented
/// by default to prevent accidental serialization of secrets into API responses.
/// If you need to serialize (e.g., forwarding config to a subprocess), use
/// `.expose()` explicitly.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Sensitive<T>(T);

impl<T> Sensitive<T> {
    /// Wrap a value, marking it as sensitive.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Access the inner value. The name is deliberately explicit to make
    /// secret access visible in code review.
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Consume the wrapper and return the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Apply a function to the inner value, keeping it wrapped.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Sensitive<U> {
        Sensitive(f(self.0))
    }
}

impl<T> fmt::Debug for Sensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl<T> fmt::Display for Sensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

/// Deserialize transparently — the inner value is read as if `Sensitive` weren't there.
/// This is needed for config loading (env vars, TOML files).
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Sensitive<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self::new)
    }
}

/// Serialize is intentionally NOT derived. Secrets should not be serialized
/// into API responses. If you need serialization (e.g., for internal config
/// forwarding), use `.expose()` and serialize the inner value directly.
///
/// This impl exists solely for cases where Sensitive<T> is embedded in a
/// struct that needs Serialize for non-API purposes (e.g., config diffing).
/// It serializes the redacted string, NOT the actual value.
impl<T> Serialize for Sensitive<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("[REDACTED]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_is_redacted() {
        let s = Sensitive::new("my-secret-key");
        assert_eq!(format!("{s:?}"), "[REDACTED]");
    }

    #[test]
    fn display_is_redacted() {
        let s = Sensitive::new("my-secret-key");
        assert_eq!(format!("{s}"), "[REDACTED]");
    }

    #[test]
    fn expose_returns_inner() {
        let s = Sensitive::new("my-secret-key".to_string());
        assert_eq!(s.expose(), "my-secret-key");
    }

    #[test]
    fn into_inner_consumes() {
        let s = Sensitive::new("value".to_string());
        let inner = s.into_inner();
        assert_eq!(inner, "value");
    }

    #[test]
    fn map_transforms_inner() {
        let s = Sensitive::new("hello".to_string());
        let upper = s.map(|v| v.to_uppercase());
        assert_eq!(upper.expose(), "HELLO");
    }

    #[test]
    fn deserialize_is_transparent() {
        let json = "\"secret-value\"";
        let s: Sensitive<String> = serde_json::from_str(json).unwrap();
        assert_eq!(s.expose(), "secret-value");
    }

    #[test]
    fn serialize_is_redacted() {
        let s = Sensitive::new("secret-value".to_string());
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"[REDACTED]\"");
    }

    #[test]
    fn struct_with_sensitive_field_debug() {
        #[derive(Debug)]
        struct Config {
            url: String,
            password: Sensitive<String>,
        }

        let config = Config {
            url: "postgres://localhost".to_string(),
            password: Sensitive::new("hunter2".to_string()),
        };

        assert_eq!(config.url, "postgres://localhost");
        assert_eq!(config.password.expose(), "hunter2");

        let debug = format!("{config:?}");
        assert!(debug.contains("postgres://localhost"));
        assert!(!debug.contains("hunter2"));
        assert!(debug.contains("[REDACTED]"));
    }
}
