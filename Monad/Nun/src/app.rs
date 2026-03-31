//! App identifiers shared across the Nyx ecosystem.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::NyxError;

/// Canonical app identifiers used in routing, storage paths, and events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum NyxApp {
    Nyx,
    Uzume,
    Anteros,
    Themis,
}

impl NyxApp {
    /// Return the stable wire-format value for this app.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nyx => "nyx",
            Self::Uzume => "Uzume",
            Self::Anteros => "Anteros",
            Self::Themis => "Themis",
        }
    }
}

impl fmt::Display for NyxApp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NyxApp {
    type Err = NyxError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "nyx" => Ok(Self::Nyx),
            "Uzume" => Ok(Self::Uzume),
            "Anteros" => Ok(Self::Anteros),
            "Themis" => Ok(Self::Themis),
            _ => Err(NyxError::bad_request(
                "invalid_app",
                "Unknown application identifier",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_as_str_matches_wire_values() {
        assert_eq!(NyxApp::Nyx.as_str(), "nyx");
        assert_eq!(NyxApp::Uzume.as_str(), "Uzume");
    }

    #[test]
    fn parse_valid_app() {
        let app = NyxApp::from_str("Uzume").expect("expected valid app");
        assert_eq!(app, NyxApp::Uzume);
    }

    #[test]
    fn parse_invalid_app() {
        let err = NyxApp::from_str("unknown").expect_err("expected invalid app");
        assert_eq!(err.code(), "invalid_app");
    }
}
