use nun::types::NyxApp;

fn app_name(app: NyxApp) -> &'static str {
    match app {
        NyxApp::Uzume => "Uzume",
        NyxApp::Anteros => "Anteros",
        NyxApp::Themis => "Themis",
        _ => "nyx",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoriesNamespace {
    Feed,
    Viewers,
    Highlights,
}

pub fn namespaced_key(app: NyxApp, entity: &str, id: impl AsRef<str>) -> String {
    format!("{}:{}:{}", app_name(app), entity, id.as_ref())
}

pub fn stories_feed_key(app: NyxApp, user_id: impl AsRef<str>) -> String {
    format!("{}:stories:feed:{}", app_name(app), user_id.as_ref())
}

pub fn story_viewers_key(app: NyxApp, story_id: impl AsRef<str>) -> String {
    format!("{}:story:{}:viewers", app_name(app), story_id.as_ref())
}

pub fn story_highlights_key(app: NyxApp, user_id: impl AsRef<str>) -> String {
    format!("{}:stories:highlights:{}", app_name(app), user_id.as_ref())
}

pub fn stories_key(app: NyxApp, namespace: StoriesNamespace, id: impl AsRef<str>) -> String {
    match namespace {
        StoriesNamespace::Feed => stories_feed_key(app, id),
        StoriesNamespace::Viewers => story_viewers_key(app, id),
        StoriesNamespace::Highlights => story_highlights_key(app, id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_app_scoped() {
        assert_eq!(
            stories_feed_key(NyxApp::Uzume, "u1"),
            "Uzume:stories:feed:u1"
        );
        assert_eq!(
            story_viewers_key(NyxApp::Uzume, "s1"),
            "Uzume:story:s1:viewers"
        );
        assert_eq!(
            story_highlights_key(NyxApp::Uzume, "u1"),
            "Uzume:stories:highlights:u1"
        );
    }
}
