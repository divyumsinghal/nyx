//! Lethe cache key helpers.

use Nun::NyxApp;

/// Build canonical cache key `{app}:{entity}:{id}`.
pub fn cache_key(app: NyxApp, entity: &str, id: &str) -> String {
    format!("{}:{}:{}", app.as_str(), entity, id)
}

/// Build story feed cache key for a viewer.
pub fn story_feed_key(viewer_id: &str) -> String {
    cache_key(NyxApp::Uzume, "stories_feed", viewer_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_feed_key_uses_namespace_convention() {
        let key = story_feed_key("viewer-123");
        assert_eq!(key, "Uzume:stories_feed:viewer-123");
    }
}
