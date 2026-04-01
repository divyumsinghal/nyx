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
}
