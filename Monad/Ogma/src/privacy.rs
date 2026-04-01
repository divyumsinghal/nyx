//! Privacy enforcement for cross-app messaging.
//!
//! Each app creates Matrix rooms tagged with `nyx.app` metadata.
//! A user's Anteros profile cannot discover their Uzume profile
//! unless they explicitly opt in via cross-app linking in account settings.
use std::collections::HashMap;

use nun::NyxApp;

/// Enforces cross-app privacy rules and generates room metadata tags.
pub struct PrivacyGuard;

impl PrivacyGuard {
    /// Return `true` if cross-app identity linking is permitted between the
    /// two apps for the current user.
    ///
    /// Same-app access (`from_app == to_app`) is always permitted.
    /// Cross-app access requires explicit user opt-in stored in `nyx.app_links`;
    /// that consent table is not yet implemented, so cross-app access is
    /// conservatively denied until the user has opted in.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ogma::PrivacyGuard;
    /// use nun::NyxApp;
    ///
    /// // Same-app: always allowed.
    /// assert!(PrivacyGuard::can_cross_link(NyxApp::Uzume, NyxApp::Uzume));
    ///
    /// // Different apps: denied until user opts in.
    /// assert!(!PrivacyGuard::can_cross_link(NyxApp::Uzume, NyxApp::Anteros));
    /// ```
    pub fn can_cross_link(from_app: NyxApp, to_app: NyxApp) -> bool {
        if from_app == to_app {
            return true;
        }
        // Cross-app linking requires explicit user opt-in via nyx.app_links.
        // The consent check is not yet implemented; return the safe default.
        false
    }

    /// Generate the Matrix room tag for an app-scoped room.
    ///
    /// Format: `nyx.{app_prefix}.{room_type}`
    ///
    /// # Example
    ///
    /// ```rust
    /// use ogma::PrivacyGuard;
    /// use nun::NyxApp;
    ///
    /// assert_eq!(PrivacyGuard::room_tag(NyxApp::Uzume, "dm"), "nyx.uzume.dm");
    /// ```
    pub fn room_tag(app: NyxApp, room_type: &str) -> String {
        format!("nyx.{}.{}", app.subject_prefix().to_lowercase(), room_type)
    }

    /// Build the Matrix room state metadata map for a new app-scoped room.
    ///
    /// The returned map should be embedded as the `initial_state` entries
    /// when calling `POST /_matrix/client/v3/createRoom`.  It contains:
    ///
    /// | Key             | Value                                       |
    /// |-----------------|---------------------------------------------|
    /// | `nyx.app`       | lowercase app name (e.g., `"uzume"`)        |
    /// | `nyx.room_type` | the `room_type` argument (e.g., `"dm"`)     |
    ///
    /// # Example
    ///
    /// ```rust
    /// use ogma::PrivacyGuard;
    /// use nun::NyxApp;
    ///
    /// let meta = PrivacyGuard::room_metadata(NyxApp::Uzume, "dm");
    /// assert_eq!(meta["nyx.app"], "uzume");
    /// assert_eq!(meta["nyx.room_type"], "dm");
    /// ```
    pub fn room_metadata(app: NyxApp, room_type: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(
            "nyx.app".to_owned(),
            app.subject_prefix().to_lowercase(),
        );
        map.insert("nyx.room_type".to_owned(), room_type.to_owned());
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_app_always_allowed() {
        assert!(PrivacyGuard::can_cross_link(NyxApp::Uzume, NyxApp::Uzume));
        assert!(PrivacyGuard::can_cross_link(NyxApp::Anteros, NyxApp::Anteros));
    }

    #[test]
    fn cross_app_denied_without_consent() {
        assert!(!PrivacyGuard::can_cross_link(NyxApp::Uzume, NyxApp::Anteros));
        assert!(!PrivacyGuard::can_cross_link(NyxApp::Anteros, NyxApp::Uzume));
        assert!(!PrivacyGuard::can_cross_link(NyxApp::Uzume, NyxApp::Themis));
    }

    #[test]
    fn room_tag_format() {
        assert_eq!(PrivacyGuard::room_tag(NyxApp::Uzume, "dm"), "nyx.uzume.dm");
        assert_eq!(
            PrivacyGuard::room_tag(NyxApp::Anteros, "match"),
            "nyx.anteros.match"
        );
    }

    #[test]
    fn room_metadata_contains_app_and_type() {
        let meta = PrivacyGuard::room_metadata(NyxApp::Uzume, "dm");
        assert_eq!(meta.get("nyx.app").map(String::as_str), Some("uzume"));
        assert_eq!(meta.get("nyx.room_type").map(String::as_str), Some("dm"));
    }
}
