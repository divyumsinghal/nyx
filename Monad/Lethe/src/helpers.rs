use std::{future::Future, time::Duration};

use nun::{NyxError, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::CacheClient;

pub mod ttl {
    use std::time::Duration;

    pub const STORIES_FEED: Duration = Duration::from_secs(5 * 60);
    pub const STORY_VIEWERS: Duration = Duration::from_secs(60);
    pub const STORY_HIGHLIGHTS: Duration = Duration::from_secs(5 * 60);
}

pub async fn get_or_set<T, F, Fut>(
    cache: &CacheClient,
    key: &str,
    ttl: Duration,
    fetcher: F,
) -> Result<T>
where
    T: Serialize + DeserializeOwned + Clone,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    if let Some(value) = cache.get_json::<T>(key).await? {
        return Ok(value);
    }

    let value = fetcher().await?;
    cache.set_json(key, &value, ttl).await?;
    Ok(value)
}

pub fn ttl_to_secs(ttl: Duration) -> Result<i64> {
    i64::try_from(ttl.as_secs())
        .map_err(|_| NyxError::bad_request("invalid_ttl", "TTL exceeds supported range"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_ttls_are_expected() {
        assert_eq!(ttl::STORIES_FEED.as_secs(), 300);
        assert_eq!(ttl::STORY_VIEWERS.as_secs(), 60);
        assert_eq!(ttl::STORY_HIGHLIGHTS.as_secs(), 300);
    }
}
