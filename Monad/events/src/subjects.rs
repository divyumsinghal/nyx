//! Event subject constants.
//! Convention: `{app_or_nyx}.{entity}.{action}`

// Platform-level
pub const USER_CREATED: &str = "nyx.user.created";
pub const USER_DELETED: &str = "nyx.user.deleted";
pub const APPS_LINKED: &str = "nyx.apps.linked";

// Uzume
pub const UZUME_POST_CREATED: &str = "Uzume.post.created";
pub const UZUME_POST_LIKED: &str = "Uzume.post.liked";
pub const UZUME_COMMENT_CREATED: &str = "Uzume.comment.created";
pub const UZUME_STORY_CREATED: &str = "Uzume.story.created";
pub const UZUME_STORY_VIEWED: &str = "Uzume.story.viewed";
pub const UZUME_USER_FOLLOWED: &str = "Uzume.user.followed";
pub const UZUME_USER_BLOCKED: &str = "Uzume.user.blocked";
pub const UZUME_PROFILE_UPDATED: &str = "Uzume.profile.updated";
pub const UZUME_REEL_CREATED: &str = "Uzume.reel.created";
pub const UZUME_REEL_VIEWED: &str = "Uzume.reel.viewed";
pub const UZUME_MEDIA_PROCESSED: &str = "Uzume.media.processed";
