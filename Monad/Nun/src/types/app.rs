use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum NyxApp {
    Uzume,
    Anteros,
    Themis,
}

impl NyxApp {
    /// Returns the subject prefix for NATS subjects and namespacing.
    ///
    /// Matches the serialized (lowercase) name of the app variant.
    pub fn subject_prefix(self) -> &'static str {
        match self {
            NyxApp::Uzume => "Uzume",
            NyxApp::Anteros => "Anteros",
            NyxApp::Themis => "Themis",
        }
    }

    /// Returns the PostgreSQL schema name for this app.
    pub fn schema(self) -> &'static str {
        match self {
            NyxApp::Uzume => "Uzume",
            NyxApp::Anteros => "Anteros",
            NyxApp::Themis => "Themis",
        }
    }

    /// Returns the Heimdall API path prefix for this app.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_serializes_to_lowercase() {
        assert_eq!(serde_json::to_string(&NyxApp::Uzume).unwrap(), "\"uzume\"");
        assert_eq!(
            serde_json::to_string(&NyxApp::Anteros).unwrap(),
            "\"anteros\""
        );
        assert_eq!(
            serde_json::to_string(&NyxApp::Themis).unwrap(),
            "\"themis\""
        );
    }

    #[test]
    fn serde_deserializes_from_lowercase() {
        assert_eq!(
            serde_json::from_str::<NyxApp>("\"uzume\"").unwrap(),
            NyxApp::Uzume
        );
        assert_eq!(
            serde_json::from_str::<NyxApp>("\"anteros\"").unwrap(),
            NyxApp::Anteros
        );
        assert_eq!(
            serde_json::from_str::<NyxApp>("\"themis\"").unwrap(),
            NyxApp::Themis
        );
    }

    #[test]
    fn serde_rejects_unknown_value() {
        assert!(serde_json::from_str::<NyxApp>("\"unknown\"").is_err());
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
}
