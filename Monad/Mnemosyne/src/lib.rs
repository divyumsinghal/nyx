//! Mnemosyne schema constants for stories.

/// Canonical PostgreSQL schema used by Uzume services.
pub const UZUME_SCHEMA: &str = "Uzume";

/// Canonical stories table names for repository layer wiring.
pub mod tables {
    pub const STORIES: &str = "stories";
    pub const STORY_VIEWS: &str = "story_views";
    pub const STORY_INTERACTIONS: &str = "story_interactions";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stories_table_constants_are_stable() {
        assert_eq!(UZUME_SCHEMA, "Uzume");
        assert_eq!(tables::STORIES, "stories");
        assert_eq!(tables::STORY_VIEWS, "story_views");
    }
}
