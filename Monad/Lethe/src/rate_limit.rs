use std::time::Duration;

use nun::Result;

use crate::CacheClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub remaining: u32,
    pub retry_after_secs: u32,
}

pub async fn check_rate_limit(
    cache: &CacheClient,
    key: &str,
    limit: u32,
    window: Duration,
) -> Result<RateLimitDecision> {
    let count = cache.incr(key).await?;

    if count == 1 {
        let _ = cache.expire(key, window).await?;
    }

    let used = count.max(0) as u32;
    let allowed = used <= limit;
    let remaining = limit.saturating_sub(used);

    Ok(RateLimitDecision {
        allowed,
        remaining,
        retry_after_secs: window.as_secs() as u32,
    })
}
