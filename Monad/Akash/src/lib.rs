//! Akash storage path primitives.

use Nun::{Id, NyxApp};

/// Storage entities supported by path helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageEntity {
    Stories,
}

impl StorageEntity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Stories => "stories",
        }
    }
}

/// Build canonical object path: `{app}/{entity}/{id}/{variant}.{ext}`.
pub fn object_path<T>(
    app: NyxApp,
    entity: StorageEntity,
    id: Id<T>,
    variant: &str,
    ext: &str,
) -> String {
    format!(
        "{}/{}/{id}/{variant}.{ext}",
        app.as_str(),
        entity.as_str(),
        id = id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Story;

    #[test]
    fn story_path_uses_required_convention() {
        let story_id = Id::<Story>::new();
        let path = object_path(
            NyxApp::Uzume,
            StorageEntity::Stories,
            story_id,
            "original",
            "jpg",
        );

        assert!(path.starts_with("Uzume/stories/"));
        assert!(path.ends_with("/original.jpg"));
    }
}
