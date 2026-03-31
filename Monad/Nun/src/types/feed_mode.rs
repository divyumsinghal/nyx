use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FeedMode {
    #[default]
    Chronological,
    Ranking,
    Custom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_chronological() {
        assert_eq!(FeedMode::default(), FeedMode::Chronological);
    }

    #[test]
    fn serde_serializes_variants() {
        assert_eq!(
            serde_json::to_string(&FeedMode::Chronological).unwrap(),
            "\"chronological\""
        );
        assert_eq!(
            serde_json::to_string(&FeedMode::Ranking).unwrap(),
            "\"ranking\""
        );
        assert_eq!(
            serde_json::to_string(&FeedMode::Custom).unwrap(),
            "\"custom\""
        );
    }

    #[test]
    fn serde_deserializes_variants() {
        assert_eq!(
            serde_json::from_str::<FeedMode>("\"chronological\"").unwrap(),
            FeedMode::Chronological
        );
        assert_eq!(
            serde_json::from_str::<FeedMode>("\"ranking\"").unwrap(),
            FeedMode::Ranking
        );
        assert_eq!(
            serde_json::from_str::<FeedMode>("\"custom\"").unwrap(),
            FeedMode::Custom
        );
    }

    #[test]
    fn serde_rejects_unknown_variant() {
        assert!(serde_json::from_str::<FeedMode>("\"ml\"").is_err());
    }
}
