//! Retry with exponential backoff and full jitter (WP-0124, ADR-0009 §1).
//!
//! Only `transient` errors retry: up to 3 attempts, delay drawn from
//! `0..=min(cap, base * 2^n)` with base 500 ms and cap 20 s. A provider's
//! `Retry-After` (`retry_after_ms`) replaces the drawn delay, still capped.
//! Every other class returns at once.
//!
//! `RetryClock` supplies both the sleep and the jitter, so a test controls
//! time completely and never sleeps for real.

use std::future::Future;
use std::pin::Pin;

use rand_core::{OsRng, RngCore};

use crate::domain::{AppError, AppResult, ErrorClass};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Attempts including the first.
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 500,
            max_delay_ms: 20_000,
        }
    }
}

/// Where a retry loop gets its time from.
pub trait RetryClock {
    fn sleep(&self, ms: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
    /// A number in `0.0..=1.0` that picks the delay inside the backoff window.
    fn jitter_unit(&self) -> f64;
}

/// Real time (tokio) and real randomness.
pub struct TokioClock;

impl RetryClock for TokioClock {
    fn sleep(&self, ms: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(tokio::time::sleep(std::time::Duration::from_millis(ms)))
    }

    fn jitter_unit(&self) -> f64 {
        (OsRng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

pub fn is_retryable(error: &AppError) -> bool {
    error.class() == ErrorClass::Transient
}

/// The full-jitter delay before retry number `attempt + 1` (0-indexed).
pub fn backoff_delay_ms(policy: &RetryPolicy, attempt: u32, jitter_unit: f64) -> u64 {
    let window = policy
        .base_delay_ms
        .saturating_mul(1u64 << attempt.min(32))
        .min(policy.max_delay_ms);
    (window as f64 * jitter_unit.clamp(0.0, 1.0)).round() as u64
}

/// How long to wait after `error` on attempt `attempt` (0-indexed), or
/// `None` when the error must not be retried.
pub fn retry_delay_for(
    error: &AppError,
    attempt: u32,
    policy: &RetryPolicy,
    jitter_unit: f64,
) -> Option<u64> {
    if !is_retryable(error) {
        return None;
    }
    if let AppError::Classified {
        retry_after_ms: Some(ms),
        ..
    } = error
    {
        return Some((*ms).min(policy.max_delay_ms));
    }
    Some(backoff_delay_ms(policy, attempt, jitter_unit))
}

/// Runs `operation` until it succeeds, fails with a non-transient error,
/// or uses up `policy.max_attempts`. Returns the last error in that case.
pub async fn retry<T, F, Fut>(
    policy: &RetryPolicy,
    clock: &dyn RetryClock,
    mut operation: F,
) -> AppResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = AppResult<T>>,
{
    let mut attempt = 0;
    loop {
        let error = match operation().await {
            Ok(value) => return Ok(value),
            Err(e) => e,
        };
        if attempt + 1 >= policy.max_attempts.max(1) {
            return Err(error);
        }
        let Some(delay) = retry_delay_for(&error, attempt, policy, clock.jitter_unit()) else {
            return Err(error);
        };
        clock.sleep(delay).await;
        attempt += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_backoff_window_doubles_up_to_the_cap() {
        let policy = RetryPolicy::default();
        let full: Vec<u64> = (0..8).map(|n| backoff_delay_ms(&policy, n, 1.0)).collect();
        assert_eq!(full, [500, 1000, 2000, 4000, 8000, 16000, 20000, 20000]);
        assert_eq!(backoff_delay_ms(&policy, 3, 0.0), 0);
        assert_eq!(backoff_delay_ms(&policy, 3, 0.25), 1000);
    }

    #[test]
    fn real_jitter_stays_in_range() {
        for _ in 0..100 {
            let u = TokioClock.jitter_unit();
            assert!((0.0..=1.0).contains(&u), "{u}");
        }
    }

    #[test]
    fn retry_after_is_honoured_but_capped() {
        let policy = RetryPolicy::default();
        let with = |ms| AppError::Classified {
            class: ErrorClass::Transient,
            code: "provider.rate_limited",
            message: "slow down".into(),
            retry_after_ms: Some(ms),
            http_status: Some(429),
        };
        assert_eq!(retry_delay_for(&with(3000), 0, &policy, 1.0), Some(3000));
        assert_eq!(
            retry_delay_for(&with(3_600_000), 0, &policy, 1.0),
            Some(20_000)
        );
    }
}
