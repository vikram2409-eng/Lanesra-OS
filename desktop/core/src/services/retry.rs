//! Retry with exponential backoff and full jitter (WP-0124).
//!
//! Implements ADR-0009 §1 retry rules for `transient` errors:
//! - Up to 3 attempts (configurable)
//! - Exponential backoff with full jitter: base 500ms, cap 20s
//! - Honours `retry_after_ms` from the error (e.g., from `Retry-After` header)
//!
//! The `Sleeper` trait allows tests to inject a controllable clock
//! so no test sleeps for real.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::{AppError, AppResult, ErrorClass};

/// Policy for retrying transient failures.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the first). Default: 3.
    pub max_attempts: u32,
    /// Base delay in milliseconds for exponential backoff. Default: 500.
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds (cap). Default: 20_000.
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

/// A sleeper that can be replaced in tests with a controllable clock.
pub trait Sleeper {
    /// Sleep for the given duration in milliseconds.
    fn sleep_ms(&self, ms: u64);
}

/// Real sleeper using thread sleep (for non-async contexts).
/// For async contexts, callers should use tokio::time::sleep directly.
pub struct ThreadSleeper;

impl Sleeper for ThreadSleeper {
    fn sleep_ms(&self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

/// Calculate the backoff delay for a given attempt (0-indexed).
/// Uses exponential backoff with full jitter:
/// `delay = random(0, min(cap, base * 2^attempt))`
///
/// Uses a simple time-based PRNG for jitter (not cryptographic).
pub fn backoff_delay_ms(attempt: u32, base_ms: u64, cap_ms: u64) -> u64 {
    let exp_delay = base_ms.saturating_mul(1u64 << attempt.min(20));
    let capped = exp_delay.min(cap_ms);
    if capped == 0 {
        return 0;
    }
    // Simple jitter using system time nanoseconds
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    // Mix in the attempt for variation across calls in the same nanosecond
    let seed = nanos.wrapping_add(attempt as u64 * 0x9E3779B9);
    // Simple LCG for pseudo-randomness
    let rand_val = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    rand_val % (capped + 1)
}

/// Determine if an error is retryable (transient class only).
pub fn is_retryable(error: &AppError) -> bool {
    matches!(
        error,
        AppError::Classified {
            class: ErrorClass::Transient,
            ..
        }
    )
}

/// Get the retry delay for an error, honouring `retry_after_ms` if present.
pub fn retry_delay_for(error: &AppError, attempt: u32, policy: &RetryPolicy) -> Option<u64> {
    if !is_retryable(error) {
        return None;
    }

    // Honour Retry-After if the error carries it
    if let AppError::Classified {
        retry_after_ms: Some(ms),
        ..
    } = error
    {
        return Some(*ms);
    }

    Some(backoff_delay_ms(attempt, policy.base_delay_ms, policy.max_delay_ms))
}

/// Retry an operation with exponential backoff.
///
/// The operation is attempted up to `policy.max_attempts` times.
/// Only `transient` errors are retried; all other errors return immediately.
///
/// Returns the successful result, or the last error if all attempts fail.
pub fn retry_with_backoff<T, F, S>(
    policy: &RetryPolicy,
    sleeper: &S,
    mut operation: F,
) -> AppResult<T>
where
    F: FnMut() -> AppResult<T>,
    S: Sleeper,
{
    let mut last_error = None;

    for attempt in 0..policy.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Only retry transient errors
                if !is_retryable(&e) {
                    return Err(e);
                }

                // Don't sleep after the last attempt
                if attempt + 1 >= policy.max_attempts {
                    return Err(e);
                }

                // Calculate delay and sleep
                if let Some(delay_ms) = retry_delay_for(&e, attempt, policy) {
                    sleeper.sleep_ms(delay_ms);
                }

                last_error = Some(e);
            }
        }
    }

    // Should be unreachable, but return the last error if we get here
    Err(last_error.unwrap_or_else(|| {
        AppError::classified(
            ErrorClass::Internal,
            "internal",
            "retry loop exited without result",
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Test sleeper that records delays instead of sleeping.
    struct TestSleeper {
        delays: Rc<RefCell<Vec<u64>>>,
    }

    impl Sleeper for TestSleeper {
        fn sleep_ms(&self, ms: u64) {
            self.delays.borrow_mut().push(ms);
        }
    }

    #[test]
    fn backoff_increases_exponentially() {
        // With no jitter (we can't control the RNG, so just check bounds)
        for attempt in 0..5 {
            let delay = backoff_delay_ms(attempt, 500, 20_000);
            let max_expected = (500u64 << attempt).min(20_000);
            assert!(
                delay <= max_expected,
                "attempt {attempt}: delay {delay} exceeds max {max_expected}"
            );
        }
    }

    #[test]
    fn backoff_respects_cap() {
        for _ in 0..10 {
            let delay = backoff_delay_ms(10, 500, 20_000);
            assert!(delay <= 20_000, "delay {delay} exceeds cap");
        }
    }

    #[test]
    fn non_transient_errors_not_retried() {
        let policy = RetryPolicy::default();
        let delays = Rc::new(RefCell::new(Vec::new()));
        let sleeper = TestSleeper {
            delays: delays.clone(),
        };

        let mut attempts = 0;
        let result: AppResult<()> = retry_with_backoff(&policy, &sleeper, || {
            attempts += 1;
            Err(AppError::classified(
                ErrorClass::Policy,
                "policy.denied",
                "denied",
            ))
        });

        assert!(result.is_err());
        assert_eq!(attempts, 1, "policy error should not retry");
        assert!(delays.borrow().is_empty(), "should not sleep");
    }

    #[test]
    fn transient_retries_up_to_max_attempts() {
        let policy = RetryPolicy {
            max_attempts: 3,
            ..Default::default()
        };
        let delays = Rc::new(RefCell::new(Vec::new()));
        let sleeper = TestSleeper {
            delays: delays.clone(),
        };

        let mut attempts = 0;
        let result: AppResult<()> = retry_with_backoff(&policy, &sleeper, || {
            attempts += 1;
            Err(AppError::classified(
                ErrorClass::Transient,
                "provider.unavailable",
                "unavailable",
            ))
        });

        assert!(result.is_err());
        assert_eq!(attempts, 3, "should attempt 3 times");
        assert_eq!(delays.borrow().len(), 2, "should sleep twice");
    }

    #[test]
    fn success_on_second_attempt() {
        let policy = RetryPolicy::default();
        let delays = Rc::new(RefCell::new(Vec::new()));
        let sleeper = TestSleeper {
            delays: delays.clone(),
        };

        let mut attempts = 0;
        let result = retry_with_backoff(&policy, &sleeper, || {
            attempts += 1;
            if attempts == 1 {
                Err(AppError::classified(
                    ErrorClass::Transient,
                    "provider.rate_limited",
                    "rate limited",
                ))
            } else {
                Ok("success")
            }
        });

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 2);
        assert_eq!(delays.borrow().len(), 1);
    }

    #[test]
    fn honours_retry_after_ms() {
        let policy = RetryPolicy::default();
        let delays = Rc::new(RefCell::new(Vec::new()));
        let sleeper = TestSleeper {
            delays: delays.clone(),
        };

        let mut attempts = 0;
        let result: AppResult<()> = retry_with_backoff(&policy, &sleeper, || {
            attempts += 1;
            if attempts == 1 {
                Err(AppError::Classified {
                    class: ErrorClass::Transient,
                    code: "provider.rate_limited",
                    message: "rate limited".to_string(),
                    retry_after_ms: Some(5000),
                    http_status: Some(429),
                })
            } else {
                Ok(())
            }
        });

        assert!(result.is_ok());
        // Should have slept exactly 5000ms (the retry_after value), not a jittered backoff
        assert_eq!(delays.borrow().as_slice(), &[5000]);
    }
}
