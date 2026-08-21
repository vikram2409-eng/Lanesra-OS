//! Integration Hub (spec §19): inbound rate limiting for the `/api/v1`
//! REST API, keyed per API client and configurable via
//! `integration_settings.api_rate_limit_per_minute`.
//!
//! A real, if blunt, mechanism: a fixed one-minute window rather than a
//! sliding window or token bucket, so a client can burst up to its limit
//! at the start of one minute and again at the start of the next, instead
//! of a perfectly smoothed rate. That's a deliberate, stated scope limit
//! - good enough for the "small team on a LAN / small self-hosted
//! deployment" scale this server targets. A distributed limiter (shared
//! across multiple server processes) is out of scope for a
//! single-process, mutex-guarded server.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, (u64, i64)>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// `Ok(())` if this request is within `limit_per_minute` for `key` in
    /// the current one-minute window; `Err(retry_after_seconds)` otherwise.
    pub fn check(&self, key: &str, limit_per_minute: i64) -> Result<(), u64> {
        let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let window = now_secs / 60;
        let mut buckets = self.buckets.lock().unwrap();
        let entry = buckets.entry(key.to_string()).or_insert((window, 0));
        if entry.0 != window {
            *entry = (window, 0);
        }
        entry.1 += 1;
        if entry.1 > limit_per_minute.max(1) {
            return Err(60 - (now_secs % 60));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_the_limit_then_rejects() {
        let limiter = RateLimiter::new();
        for _ in 0..5 {
            assert!(limiter.check("client-a", 5).is_ok());
        }
        assert!(limiter.check("client-a", 5).is_err());
    }

    #[test]
    fn tracks_each_key_independently() {
        let limiter = RateLimiter::new();
        for _ in 0..3 {
            assert!(limiter.check("client-a", 3).is_ok());
        }
        assert!(limiter.check("client-a", 3).is_err());
        // A different client's own bucket is untouched by client-a's usage.
        assert!(limiter.check("client-b", 3).is_ok());
    }
}
