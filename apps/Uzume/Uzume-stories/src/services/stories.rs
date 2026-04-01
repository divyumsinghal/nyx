//! Pure domain logic for story operations.
//!
//! Helpers here are side-effect-free and easy to unit test.

use chrono::Utc;
use nun::{Cursor, NyxError, PageResponse};
use uuid::Uuid;

use crate::models::{
    story::{MediaType, StoryResponse, StoryRow, StoryStatus},
    viewer::{StoryViewRow, ViewerResponse},
};

/// Parse a content type into the supported story media type.
pub fn media_type_from_content_type(content_type: &str) -> Result<MediaType, NyxError> {
    if content_type.starts_with("image/") {
        Ok(MediaType::Image)
    } else if content_type.starts_with("video/") {
        Ok(MediaType::Video)
    } else {
        Err(NyxError::bad_request(
            "unsupported_media_type",
            "Only image/* and video/* content types are supported",
        ))
    }
}

/// Enforce story visibility and authorization.
///
/// Authors can always view their own stories. Non-authors can only view
/// stories that are active and not expired.
pub fn ensure_story_visible(
    story: &StoryRow,
    viewer_identity_id: Option<Uuid>,
) -> Result<(), NyxError> {
    if viewer_identity_id.is_some_and(|viewer| viewer == story.author_identity_id) {
        return Ok(());
    }

    if story.status == StoryStatus::Active && story.expires_at > Utc::now() {
        Ok(())
    } else {
        Err(NyxError::not_found("story_not_found", "Story not found"))
    }
}

/// Decide whether calling `record_view` is meaningful.
///
/// Database-level idempotency is guaranteed by `ON CONFLICT DO NOTHING`; this
/// helper avoids unnecessary writes for invalid view attempts.
pub fn should_record_view(story: &StoryRow, viewer_identity_id: Uuid) -> bool {
    story.author_identity_id != viewer_identity_id
        && story.status == StoryStatus::Active
        && story.expires_at > Utc::now()
}

/// Ensure the caller is the story author.
pub fn ensure_story_owner(story: &StoryRow, requester_identity_id: Uuid) -> Result<(), NyxError> {
    if story.author_identity_id == requester_identity_id {
        Ok(())
    } else {
        Err(NyxError::forbidden(
            "not_story_owner",
            "Only the story owner can perform this action",
        ))
    }
}

/// Build a paginated story page from query rows.
pub fn build_story_page(rows: Vec<StoryRow>, limit: u16) -> PageResponse<StoryResponse> {
    PageResponse::from_overflowed(
        rows.into_iter().map(StoryResponse::from).collect(),
        limit,
        |story| Cursor::timestamp_id(story.created_at, story.id),
    )
}

/// Build a paginated story-viewer page from query rows.
pub fn build_viewer_page(rows: Vec<StoryViewRow>, limit: u16) -> PageResponse<ViewerResponse> {
    let limit = usize::from(limit);
    let has_more = rows.len() > limit;

    let mut kept = rows;
    if has_more {
        kept.truncate(limit);
    }

    let next_cursor = if has_more {
        kept.last()
            .map(|row| Cursor::timestamp_id(row.viewed_at, row.viewer_identity_id).encode())
    } else {
        None
    };

    let items = kept.into_iter().map(ViewerResponse::from).collect();

    PageResponse {
        items,
        next_cursor,
        has_more,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;

    fn make_story(status: StoryStatus, expires_at: chrono::DateTime<Utc>) -> StoryRow {
        StoryRow {
            id: Uuid::now_v7(),
            author_identity_id: Uuid::now_v7(),
            author_alias: "alice".to_string(),
            media_url: Some("https://cdn.example/story.jpg".to_string()),
            media_type: MediaType::Image,
            duration_secs: Some(5),
            status,
            view_count: 0,
            expires_at,
            created_at: Utc::now(),
        }
    }

    // #given a story owned by the requester
    // #when visibility is checked
    // #then access is allowed regardless of status
    #[test]
    fn ensure_story_visible_allows_owner() {
        let mut story = make_story(StoryStatus::Pending, Utc::now() - Duration::hours(1));
        let owner = Uuid::now_v7();
        story.author_identity_id = owner;

        assert!(ensure_story_visible(&story, Some(owner)).is_ok());
    }

    // #given an active, unexpired story
    // #when a non-owner requests visibility
    // #then access is allowed
    #[test]
    fn ensure_story_visible_allows_public_active_story() {
        let story = make_story(StoryStatus::Active, Utc::now() + Duration::minutes(10));

        assert!(ensure_story_visible(&story, Some(Uuid::now_v7())).is_ok());
        assert!(ensure_story_visible(&story, None).is_ok());
    }

    // #given an expired or non-active story
    // #when a non-owner requests visibility
    // #then access is denied as not found
    #[test]
    fn ensure_story_visible_rejects_non_owner_for_inactive_story() {
        let pending = make_story(StoryStatus::Pending, Utc::now() + Duration::hours(1));
        let expired = make_story(StoryStatus::Active, Utc::now() - Duration::seconds(1));

        let err_pending = ensure_story_visible(&pending, Some(Uuid::now_v7())).unwrap_err();
        assert_eq!(err_pending.status_code(), 404);
        assert_eq!(err_pending.code(), "story_not_found");

        let err_expired = ensure_story_visible(&expired, Some(Uuid::now_v7())).unwrap_err();
        assert_eq!(err_expired.status_code(), 404);
        assert_eq!(err_expired.code(), "story_not_found");
    }

    // #given repeated view attempts
    // #when should_record_view is evaluated
    // #then author views are skipped and active non-owner views are accepted
    #[test]
    fn should_record_view_respects_idempotency_guardrails() {
        let mut active_story = make_story(StoryStatus::Active, Utc::now() + Duration::minutes(10));
        let owner = Uuid::now_v7();
        active_story.author_identity_id = owner;
        let viewer = Uuid::now_v7();

        assert!(should_record_view(&active_story, viewer));
        assert!(!should_record_view(&active_story, owner));

        let expired_story = make_story(StoryStatus::Active, Utc::now() - Duration::minutes(1));
        assert!(!should_record_view(&expired_story, viewer));
    }
}
