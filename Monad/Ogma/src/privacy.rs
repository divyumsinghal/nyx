//! Privacy enforcement for cross-app messaging.
//!
//! Each app creates Matrix rooms tagged with `nyx.app` metadata.
//! A user's Anteros profile cannot discover their Uzume profile
//! unless they explicitly opt in via cross-app linking in account settings.
use Nun::NyxApp;

pub struct PrivacyGuard;

impl PrivacyGuard {
    /// Return `true` if cross-app identity linking is permitted between the
    /// two apps for the current user.
    ///
    /// This is a placeholder — the real implementation queries the
    /// `nyx.app_links` consent table. Same-app access is always allowed.
    pub fn can_cross_link(from_app: NyxApp, to_app: NyxApp) -> bool {
        from_app == to_app
    }

    /// Generate the Matrix room tag for an app-scoped room.
    ///
    /// Format: `nyx.{app_prefix}.{room_type}`
    pub fn room_tag(app: NyxApp, room_type: &str) -> String {
        format!("nyx.{}.{}", app.subject_prefix().to_lowercase(), room_type)
    }
}
