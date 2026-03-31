use std::collections::HashSet;

use serde::{Deserialize, Deserializer, Serialize};

use crate::NyxApp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkDirection {
    OneWay,
    TwoWay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum LinkPolicy {
    OneWay,
    TwoWay,
    AppSelective {
        apps: Vec<NyxApp>,
        direction: LinkDirection,
    },
    Revoked,
}

impl<'de> Deserialize<'de> for LinkPolicy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
        enum RawLinkPolicy {
            OneWay,
            TwoWay,
            AppSelective {
                apps: Vec<NyxApp>,
                direction: LinkDirection,
            },
            Revoked,
        }

        let raw = RawLinkPolicy::deserialize(deserializer)?;
        let policy = match raw {
            RawLinkPolicy::OneWay => Self::OneWay,
            RawLinkPolicy::TwoWay => Self::TwoWay,
            RawLinkPolicy::AppSelective { apps, direction } => {
                if apps.is_empty() {
                    return Err(serde::de::Error::custom(
                        "app_selective policy must include at least one app",
                    ));
                }

                let unique_apps = apps.iter().copied().collect::<HashSet<_>>();
                if unique_apps.len() != apps.len() {
                    return Err(serde::de::Error::custom(
                        "app_selective policy must not contain duplicate apps",
                    ));
                }

                Self::AppSelective { apps, direction }
            }
            RawLinkPolicy::Revoked => Self::Revoked,
        };

        Ok(policy)
    }
}

impl Default for LinkPolicy {
    fn default() -> Self {
        Self::Revoked
    }
}

impl LinkPolicy {
    pub fn is_revoked(&self) -> bool {
        matches!(self, Self::Revoked)
    }

    pub fn direction(&self) -> Option<LinkDirection> {
        match self {
            Self::OneWay => Some(LinkDirection::OneWay),
            Self::TwoWay => Some(LinkDirection::TwoWay),
            Self::AppSelective { direction, .. } => Some(*direction),
            Self::Revoked => None,
        }
    }

    pub fn applies_to_app(&self, app: NyxApp) -> bool {
        match self {
            Self::OneWay | Self::TwoWay => true,
            Self::AppSelective { apps, .. } => apps.contains(&app),
            Self::Revoked => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip_one_way() {
        let policy = LinkPolicy::OneWay;
        let json = serde_json::to_string(&policy).unwrap();
        let parsed: LinkPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, policy);
    }

    #[test]
    fn serde_round_trip_two_way() {
        let policy = LinkPolicy::TwoWay;
        let json = serde_json::to_string(&policy).unwrap();
        let parsed: LinkPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, policy);
    }

    #[test]
    fn serde_round_trip_app_selective() {
        let policy = LinkPolicy::AppSelective {
            apps: vec![NyxApp::Uzume, NyxApp::Themis],
            direction: LinkDirection::TwoWay,
        };
        let json = serde_json::to_string(&policy).unwrap();
        let parsed: LinkPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, policy);
    }

    #[test]
    fn serde_round_trip_revoked() {
        let policy = LinkPolicy::Revoked;
        let json = serde_json::to_string(&policy).unwrap();
        let parsed: LinkPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, policy);
    }

    #[test]
    fn helper_methods_model_semantics() {
        let selective = LinkPolicy::AppSelective {
            apps: vec![NyxApp::Uzume],
            direction: LinkDirection::OneWay,
        };
        assert_eq!(selective.direction(), Some(LinkDirection::OneWay));
        assert!(selective.applies_to_app(NyxApp::Uzume));
        assert!(!selective.applies_to_app(NyxApp::Themis));

        let revoked = LinkPolicy::Revoked;
        assert!(revoked.is_revoked());
        assert_eq!(revoked.direction(), None);
        assert!(!revoked.applies_to_app(NyxApp::Uzume));
    }

    #[test]
    fn default_policy_is_revoked() {
        assert_eq!(LinkPolicy::default(), LinkPolicy::Revoked);
    }

    #[test]
    fn serde_rejects_invalid_tag() {
        let invalid = "{\"type\":\"invalid\"}";
        assert!(serde_json::from_str::<LinkPolicy>(invalid).is_err());
    }

    #[test]
    fn serde_rejects_invalid_direction() {
        let invalid =
            "{\"type\":\"app_selective\",\"apps\":[\"uzume\"],\"direction\":\"sideways\"}";
        assert!(serde_json::from_str::<LinkPolicy>(invalid).is_err());
    }

    #[test]
    fn serde_rejects_unknown_field_on_app_selective() {
        let invalid =
            "{\"type\":\"app_selective\",\"apps\":[\"uzume\"],\"direction\":\"one_way\",\"extra\":true}";
        assert!(serde_json::from_str::<LinkPolicy>(invalid).is_err());
    }

    #[test]
    fn serde_rejects_empty_app_selective_apps() {
        // #given
        let invalid = "{\"type\":\"app_selective\",\"apps\":[],\"direction\":\"one_way\"}";

        // #when
        let parsed = serde_json::from_str::<LinkPolicy>(invalid);

        // #then
        assert!(parsed.is_err());
    }

    #[test]
    fn serde_rejects_duplicate_app_selective_apps() {
        // #given
        let invalid =
            "{\"type\":\"app_selective\",\"apps\":[\"uzume\",\"uzume\"],\"direction\":\"two_way\"}";

        // #when
        let parsed = serde_json::from_str::<LinkPolicy>(invalid);

        // #then
        assert!(parsed.is_err());
    }
}
