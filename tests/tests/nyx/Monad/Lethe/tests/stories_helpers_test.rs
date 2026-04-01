use nun::types::NyxApp;

use lethe::{
    helpers::ttl,
    keys::{stories_key, StoriesNamespace},
};

#[test]
fn stories_namespaces_are_app_scoped() {
    assert_eq!(
        stories_key(NyxApp::Uzume, StoriesNamespace::Feed, "user-1"),
        "Uzume:stories:feed:user-1"
    );
    assert_eq!(
        stories_key(NyxApp::Uzume, StoriesNamespace::Viewers, "story-1"),
        "Uzume:story:story-1:viewers"
    );
    assert_eq!(
        stories_key(NyxApp::Uzume, StoriesNamespace::Highlights, "user-1"),
        "Uzume:stories:highlights:user-1"
    );
}

#[test]
fn stories_ttls_are_stable_contracts() {
    assert_eq!(ttl::STORIES_FEED.as_secs(), 300);
    assert_eq!(ttl::STORY_VIEWERS.as_secs(), 60);
    assert_eq!(ttl::STORY_HIGHLIGHTS.as_secs(), 300);
}
