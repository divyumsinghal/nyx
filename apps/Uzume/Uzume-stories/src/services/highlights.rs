//! Pure domain logic for highlight operations.

use nun::{NyxError, PageResponse};

use crate::models::highlight::{HighlightResponse, HighlightRow};

/// Ensure the caller owns the highlight.
pub fn ensure_highlight_owner(
    highlight: &HighlightRow,
    requester_identity_id: uuid::Uuid,
) -> Result<(), NyxError> {
    if highlight.owner_identity_id == requester_identity_id {
        Ok(())
    } else {
        Err(NyxError::forbidden(
            "not_highlight_owner",
            "Only the highlight owner can perform this action",
        ))
    }
}

/// Validate highlight title shape.
pub fn validate_highlight_title(title: &str) -> Result<(), NyxError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(NyxError::bad_request(
            "highlight_title_required",
            "Highlight title is required",
        ));
    }
    if trimmed.chars().count() > 150 {
        return Err(NyxError::bad_request(
            "highlight_title_too_long",
            "Highlight title must be 150 characters or fewer",
        ));
    }
    Ok(())
}

/// Build a simple non-overflowed page response for highlights.
pub fn build_highlight_page(rows: Vec<HighlightRow>) -> PageResponse<HighlightResponse> {
    PageResponse::new(
        rows.into_iter().map(HighlightResponse::from).collect(),
        None,
        false,
    )
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;

    fn make_highlight(owner_identity_id: Uuid) -> HighlightRow {
        HighlightRow {
            id: Uuid::now_v7(),
            owner_identity_id,
            owner_alias: "alice".to_string(),
            title: "summer".to_string(),
            cover_url: None,
            created_at: Utc::now(),
        }
    }

    // #given a highlight and its owner
    // #when ownership is checked
    // #then authorization is granted
    #[test]
    fn ensure_highlight_owner_allows_owner() {
        let owner = Uuid::now_v7();
        let highlight = make_highlight(owner);

        assert!(ensure_highlight_owner(&highlight, owner).is_ok());
    }

    // #given a highlight and a different requester
    // #when ownership is checked
    // #then authorization is denied with forbidden
    #[test]
    fn ensure_highlight_owner_rejects_non_owner() {
        let owner = Uuid::now_v7();
        let other = Uuid::now_v7();
        let highlight = make_highlight(owner);

        let err = ensure_highlight_owner(&highlight, other).unwrap_err();
        assert_eq!(err.status_code(), 403);
        assert_eq!(err.code(), "not_highlight_owner");
    }
}
