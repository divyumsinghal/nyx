//! Token-bucket rate limiter backed by DragonflyDB.
use crate::client::CacheClient;
use Nun::Result;

/// Check rate limit using INCR + expiry.
///
/// Returns `Ok(())` when the request is within the allowed rate.
/// Returns `Err(NyxError::rate_limited(...))` when the limit is exceeded.
///
/// # Errors
///
/// Propagates cache errors or returns a rate-limit error when exceeded.
pub async fn check_rate_limit(
    cache: &CacheClient,
    key: &str,
    max_requests: u32,
    window_seconds: u64,
) -> Result<()> {
    let count = cache.incr(key).await?;
    if count == 1 {
        // First request in this window: set the expiry.
        // Ignore the error — worst case the key never expires for this window.
        let _ = cache.set(key, &count, window_seconds).await;
    }
    if count > i64::from(max_requests) {
        #[allow(clippy::cast_possible_truncation)]
        return Err(Nun::NyxError::rate_limited(
            window_seconds.min(u64::from(u32::MAX)) as u32,
        ));
    }
    Ok(())
}
