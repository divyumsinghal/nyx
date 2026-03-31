//! Storage path conventions.
//! Format: `{app}/{entity}/{id}/{variant}.{ext}`
use Nun::NyxApp;

pub struct StoragePath;

impl StoragePath {
    pub fn avatar(app: NyxApp, user_id: &str) -> String {
        format!("{}/avatars/{}/original.webp", app.subject_prefix(), user_id)
    }

    pub fn post_media(post_id: &str, index: u32, ext: &str) -> String {
        format!("Uzume/posts/{}/{}.{}", post_id, index, ext)
    }

    pub fn story_media(story_id: &str, ext: &str) -> String {
        format!("Uzume/stories/{}/media.{}", story_id, ext)
    }

    pub fn reel_video(reel_id: &str) -> String {
        format!("Uzume/reels/{}/video.mp4", reel_id)
    }

    pub fn reel_thumbnail(reel_id: &str) -> String {
        format!("Uzume/reels/{}/thumbnail.webp", reel_id)
    }
}
