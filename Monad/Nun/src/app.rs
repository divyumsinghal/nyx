//! The [`NyxApp`] enum — identifies which application is running.
//!
//! Every service knows which app it belongs to. This is used for:
//! - App-scoped alias routing (Heka)
//! - NATS subject namespacing (nyx-events)
//! - PostgreSQL schema selection (Mnemosyne)
//! - Privacy enforcement across app boundaries (Ogma)
//!
//! The enum is `#[non_exhaustive]` so adding new apps is non-breaking.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Identifies a Nyx application.
///
/// # Non-exhaustive
///
/// This enum is marked `#[non_exhaustive]`. Matching on it requires a
/// wildcard arm. This lets new apps be added in minor versions without
/// breaking downstream code.
///
/// ```rust
/// use Nun::NyxApp;
///
/// fn schema_name(app: &NyxApp) -> &'static str {
///     match app {
///         NyxApp::Uzume   => "Uzume",
///         NyxApp::Anteros => "Anteros",
///         NyxApp::Themis  => "Themis",
///         _ => "unknown",
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum NyxApp {
    /// Social media platform — the first Nyx app.
    Uzume,

    /// Dating platform — planned.
    Anteros,

    /// Housing intelligence platform — planned.
    Themis,
}

impl NyxApp {
    /// Return the lowercase name used in NATS subjects and cache key prefixes.
    ///
    /// ```
    /// use Nun::NyxApp;
    /// assert_eq!(NyxApp::Uzume.subject_prefix(), "Uzume");
    /// ```
    pub fn subject_prefix(self) -> &'static str {
        match self {
            NyxApp::Uzume => "Uzume",
            NyxApp::Anteros => "Anteros",
            NyxApp::Themis => "Themis",
        }
    }

    /// Return the PostgreSQL schema name for this app.
    ///
    /// ```
    /// use Nun::NyxApp;
    /// assert_eq!(NyxApp::Uzume.schema(), "Uzume");
    /// ```
    pub fn schema(self) -> &'static str {
        match self {
            NyxApp::Uzume => "Uzume",
            NyxApp::Anteros => "Anteros",
            NyxApp::Themis => "Themis",
        }
    }

    /// Return the URL path prefix used by the Heimdall gateway.
    ///
    /// ```
    /// use Nun::NyxApp;
    /// assert_eq!(NyxApp::Uzume.path_prefix(), "/api/Uzume");
    /// ```
    pub fn path_prefix(self) -> &'static str {
        match self {
            NyxApp::Uzume => "/api/Uzume",
            NyxApp::Anteros => "/api/Anteros",
            NyxApp::Themis => "/api/Themis",
        }
    }
}

impl fmt::Display for NyxApp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.subject_prefix())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_prefix_is_correct() {
        assert_eq!(NyxApp::Uzume.subject_prefix(), "Uzume");
        assert_eq!(NyxApp::Anteros.subject_prefix(), "Anteros");
        assert_eq!(NyxApp::Themis.subject_prefix(), "Themis");
    }

    #[test]
    fn schema_is_correct() {
        assert_eq!(NyxApp::Uzume.schema(), "Uzume");
        assert_eq!(NyxApp::Anteros.schema(), "Anteros");
        assert_eq!(NyxApp::Themis.schema(), "Themis");
    }

    #[test]
    fn path_prefix_is_correct() {
        assert_eq!(NyxApp::Uzume.path_prefix(), "/api/Uzume");
        assert_eq!(NyxApp::Anteros.path_prefix(), "/api/Anteros");
        assert_eq!(NyxApp::Themis.path_prefix(), "/api/Themis");
    }

    #[test]
    fn display_shows_subject_prefix() {
        assert_eq!(NyxApp::Uzume.to_string(), "Uzume");
    }

    #[test]
    fn serde_round_trip() {
        let app = NyxApp::Uzume;
        let json = serde_json::to_string(&app).unwrap();
        let parsed: NyxApp = serde_json::from_str(&json).unwrap();
        assert_eq!(app, parsed);
    }
}
