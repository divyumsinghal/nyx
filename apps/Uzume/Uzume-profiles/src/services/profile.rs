//! Pure domain logic for profile operations.
//!
//! Functions in this module perform no I/O — they operate on types already
//! fetched from the database and return domain results. This makes them
//! straightforward to unit-test without database fixtures.

use crate::models::profile::{ProfileRow, ProfileUpdate};
use nun::NyxError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Request / response domain types ─────────────────────────────────────────

/// HTTP request body for `PATCH /profiles/me`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct PatchProfileRequest {
    #[validate(length(min = 1, max = 80, message = "Display name must be 1–80 characters"))]
    pub display_name: Option<String>,

    #[validate(length(max = 300, message = "Bio must be at most 300 characters"))]
    pub bio: Option<String>,

    #[validate(url(message = "avatar_url must be a valid URL"))]
    pub avatar_url: Option<String>,

    pub is_private: Option<bool>,
}

impl PatchProfileRequest {
    /// Convert the request body into a database update payload.
    #[must_use]
    pub fn into_profile_update(self) -> ProfileUpdate {
        ProfileUpdate {
            display_name: self.display_name,
            bio: self.bio,
            avatar_url: self.avatar_url,
            is_private: self.is_private,
        }
    }
}

/// The public-facing profile representation returned to API clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub is_private: bool,
    pub is_verified: bool,
    pub follower_count: i64,
    pub following_count: i64,
    pub post_count: i64,
}

impl From<ProfileRow> for ProfileResponse {
    fn from(row: ProfileRow) -> Self {
        Self {
            id: row.id,
            alias: row.alias,
            display_name: row.display_name,
            bio: row.bio,
            avatar_url: row.avatar_url,
            is_private: row.is_private,
            is_verified: row.is_verified,
            follower_count: row.follower_count,
            following_count: row.following_count,
            post_count: row.post_count,
        }
    }
}

// ── Domain logic ─────────────────────────────────────────────────────────────

/// Determine whether `viewer_identity_id` is allowed to view the profile.
///
/// Private profiles are only visible to the owner themselves. Public profiles
/// are visible to everyone. The full cross-app link-policy check happens in
/// the Heka layer; this function handles the simple Uzume-internal case.
pub fn check_profile_visibility(
    profile: &ProfileRow,
    viewer_identity_id: Option<Uuid>,
) -> Result<(), NyxError> {
    if !profile.is_private {
        return Ok(());
    }

    match viewer_identity_id {
        Some(viewer) if viewer == profile.identity_id => Ok(()),
        _ => Err(NyxError::forbidden(
            "profile_private",
            "This profile is private",
        )),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_profile(is_private: bool, identity_id: Uuid) -> ProfileRow {
        ProfileRow {
            id: Uuid::now_v7(),
            identity_id,
            alias: "alice".to_string(),
            display_name: "Alice".to_string(),
            bio: None,
            avatar_url: None,
            is_private,
            is_verified: false,
            follower_count: 0,
            following_count: 0,
            post_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn public_profile_visible_to_anyone() {
        let owner = Uuid::now_v7();
        let profile = make_profile(false, owner);
        assert!(check_profile_visibility(&profile, None).is_ok());
        assert!(check_profile_visibility(&profile, Some(Uuid::now_v7())).is_ok());
    }

    #[test]
    fn private_profile_visible_to_owner() {
        let owner = Uuid::now_v7();
        let profile = make_profile(true, owner);
        assert!(check_profile_visibility(&profile, Some(owner)).is_ok());
    }

    #[test]
    fn private_profile_hidden_from_others() {
        let owner = Uuid::now_v7();
        let stranger = Uuid::now_v7();
        let profile = make_profile(true, owner);

        let err = check_profile_visibility(&profile, Some(stranger)).unwrap_err();
        assert_eq!(err.status_code(), 403);
        assert_eq!(err.code(), "profile_private");
    }

    #[test]
    fn private_profile_hidden_from_unauthenticated() {
        let owner = Uuid::now_v7();
        let profile = make_profile(true, owner);
        let err = check_profile_visibility(&profile, None).unwrap_err();
        assert_eq!(err.status_code(), 403);
    }

    #[test]
    fn patch_request_converts_to_update() {
        let req = PatchProfileRequest {
            display_name: Some("Bob".to_string()),
            bio: Some("Hello".to_string()),
            avatar_url: None,
            is_private: Some(true),
        };
        let update = req.into_profile_update();
        assert_eq!(update.display_name.as_deref(), Some("Bob"));
        assert_eq!(update.bio.as_deref(), Some("Hello"));
        assert!(update.avatar_url.is_none());
        assert_eq!(update.is_private, Some(true));
    }

    #[test]
    fn profile_response_from_row() {
        let id = Uuid::now_v7();
        let identity_id = Uuid::now_v7();
        let now = Utc::now();
        let row = ProfileRow {
            id,
            identity_id,
            alias: "alice".to_string(),
            display_name: "Alice".to_string(),
            bio: Some("bio".to_string()),
            avatar_url: None,
            is_private: false,
            is_verified: true,
            follower_count: 10,
            following_count: 5,
            post_count: 3,
            created_at: now,
            updated_at: now,
        };
        let resp = ProfileResponse::from(row);
        assert_eq!(resp.id, id);
        assert_eq!(resp.alias, "alice");
        assert_eq!(resp.follower_count, 10);
        assert!(resp.is_verified);
    }

    #[test]
    fn patch_request_validate_rejects_empty_display_name() {
        use validator::Validate;
        let req = PatchProfileRequest {
            display_name: Some(String::new()),
            ..Default::default()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn patch_request_validate_rejects_invalid_url() {
        use validator::Validate;
        let req = PatchProfileRequest {
            avatar_url: Some("not-a-url".to_string()),
            ..Default::default()
        };
        assert!(req.validate().is_err());
    }
}
