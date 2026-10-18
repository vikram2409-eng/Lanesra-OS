//! Tests for retry with backoff and error taxonomy (WP-0124).
//!
//! Covers: REQ-ORCH-033 (WP-0124 AC-1, AC-2, AC-3)
//!
//! AC-1: 429 then success → step succeeds on attempt 2
//! AC-2: Policy denial → no retry
//! AC-3: Backoff timing verified with a controllable clock

use lanesra_core::domain::{AppError, ErrorClass};
use lanesra_core::services::retry::{
    backoff_delay_ms, is_retryable, retry_delay_for, RetryPolicy, Sleeper,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Test sleeper that records delays instead of sleeping (controllable clock).
/// Covers: REQ-ORCH-033 (WP-0124 AC-3)
struct TestSleeper {
    delays: Rc<RefCell<Vec<u64>>>,
}

impl Sleeper for TestSleeper {
    fn sleep_ms(&self, ms: u64) {
        self.delays.borrow_mut().push(ms);
    }
}

fn test_sleeper() -> (TestSleeper, Rc<RefCell<Vec<u64>>>) {
    let delays = Rc::new(RefCell::new(Vec::new()));
    let sleeper = TestSleeper {
        delays: delays.clone(),
    };
    (sleeper, delays)
}

/// AC-1: 429 then success → step succeeds on attempt 2
/// Covers: REQ-ORCH-033 (WP-0124 AC-1)
#[test]
fn ac1_rate_limited_then_success() {
    let policy = RetryPolicy::default();
    let (sleeper, delays) = test_sleeper();

    let mut attempts = 0;
    let result = lanesra_core::services::retry::retry_with_backoff(&policy, &sleeper, || {
        attempts += 1;
        if attempts == 1 {
            Err(AppError::Classified {
                class: ErrorClass::Transient,
                code: "provider.rate_limited",
                message: "Rate limited".to_string(),
                retry_after_ms: None,
                http_status: Some(429),
            })
        } else {
            Ok("success")
        }
    });

    assert_eq!(result.unwrap(), "success");
    assert_eq!(attempts, 2, "should succeed on attempt 2");
    assert_eq!(delays.borrow().len(), 1, "should sleep once between attempts");
}

/// AC-2: Policy denial → no retry
/// Covers: REQ-ORCH-033 (WP-0124 AC-2)
#[test]
fn ac2_policy_denial_no_retry() {
    let policy = RetryPolicy::default();
    let (sleeper, delays) = test_sleeper();

    let mut attempts = 0;
    let result: Result<(), _> =
        lanesra_core::services::retry::retry_with_backoff(&policy, &sleeper, || {
            attempts += 1;
            Err(AppError::classified(
                ErrorClass::Policy,
                "policy.denied",
                "Denied by policy",
            ))
        });

    assert!(result.is_err());
    assert_eq!(attempts, 1, "policy error should not be retried");
    assert!(
        delays.borrow().is_empty(),
        "should not sleep for non-retryable error"
    );
}

/// AC-2: Validation error → no retry
/// Covers: REQ-ORCH-033 (WP-0124 AC-2)
#[test]
fn ac2_validation_error_no_retry() {
    let policy = RetryPolicy::default();
    let (sleeper, delays) = test_sleeper();

    let mut attempts = 0;
    let result: Result<(), _> =
        lanesra_core::services::retry::retry_with_backoff(&policy, &sleeper, || {
            attempts += 1;
            Err(AppError::classified(
                ErrorClass::Validation,
                "validation.arguments",
                "Invalid arguments",
            ))
        });

    assert!(result.is_err());
    assert_eq!(attempts, 1, "validation error should not be retried");
    assert!(delays.borrow().is_empty());
}

/// AC-3: Backoff timing verified with controllable clock
/// Covers: REQ-ORCH-033 (WP-0124 AC-3)
#[test]
fn ac3_backoff_timing_with_controllable_clock() {
    let policy = RetryPolicy {
        max_attempts: 4,
        base_delay_ms: 500,
        max_delay_ms: 20_000,
    };
    let (sleeper, delays) = test_sleeper();

    let mut attempts = 0;
    let result: Result<(), _> =
        lanesra_core::services::retry::retry_with_backoff(&policy, &sleeper, || {
            attempts += 1;
            Err(AppError::classified(
                ErrorClass::Transient,
                "provider.unavailable",
                "Unavailable",
            ))
        });

    assert!(result.is_err());
    assert_eq!(attempts, 4);

    let recorded = delays.borrow();
    assert_eq!(recorded.len(), 3, "should sleep 3 times for 4 attempts");

    // Verify exponential backoff bounds (with jitter, delays are random but bounded)
    // Attempt 1 delay: 0..=500 (base * 2^0)
    assert!(
        recorded[0] <= 500,
        "first delay {} exceeds max 500",
        recorded[0]
    );
    // Attempt 2 delay: 0..=1000 (base * 2^1)
    assert!(
        recorded[1] <= 1000,
        "second delay {} exceeds max 1000",
        recorded[1]
    );
    // Attempt 3 delay: 0..=2000 (base * 2^2)
    assert!(
        recorded[2] <= 2000,
        "third delay {} exceeds max 2000",
        recorded[2]
    );
}

/// Backoff respects the maximum cap
/// Covers: REQ-ORCH-033 (WP-0124 AC-3)
#[test]
fn backoff_respects_cap() {
    for _ in 0..20 {
        let delay = backoff_delay_ms(10, 500, 20_000);
        assert!(
            delay <= 20_000,
            "delay {delay} exceeds cap of 20000ms"
        );
    }
}

/// Retry-After header is honoured
/// Covers: REQ-ORCH-033 (WP-0124 AC-1)
#[test]
fn honours_retry_after_header() {
    let policy = RetryPolicy::default();
    let (sleeper, delays) = test_sleeper();

    let mut attempts = 0;
    let result = lanesra_core::services::retry::retry_with_backoff(&policy, &sleeper, || {
        attempts += 1;
        if attempts == 1 {
            Err(AppError::Classified {
                class: ErrorClass::Transient,
                code: "provider.rate_limited",
                message: "Rate limited".to_string(),
                retry_after_ms: Some(5000),
                http_status: Some(429),
            })
        } else {
            Ok("success")
        }
    });

    assert_eq!(result.unwrap(), "success");
    // Should sleep exactly 5000ms (the Retry-After value), not a jittered backoff
    assert_eq!(delays.borrow().as_slice(), &[5000]);
}

/// Transient errors are retryable, others are not
/// Covers: REQ-ORCH-033 (WP-0124)
#[test]
fn error_class_retryability() {
    let transient = AppError::classified(ErrorClass::Transient, "provider.timeout", "timeout");
    let policy_err = AppError::classified(ErrorClass::Policy, "policy.denied", "denied");
    let validation_err =
        AppError::classified(ErrorClass::Validation, "validation.arguments", "bad args");
    let config_err = AppError::classified(ErrorClass::Configuration, "config.missing_key", "no key");
    let internal_err = AppError::classified(ErrorClass::Internal, "internal", "bug");

    assert!(is_retryable(&transient), "transient should be retryable");
    assert!(!is_retryable(&policy_err), "policy should not be retryable");
    assert!(
        !is_retryable(&validation_err),
        "validation should not be retryable"
    );
    assert!(
        !is_retryable(&config_err),
        "configuration should not be retryable"
    );
    assert!(
        !is_retryable(&internal_err),
        "internal should not be retryable"
    );
}

/// Retry delay is None for non-transient errors
/// Covers: REQ-ORCH-033 (WP-0124)
#[test]
fn retry_delay_none_for_non_transient() {
    let policy = RetryPolicy::default();
    let policy_err = AppError::classified(ErrorClass::Policy, "policy.denied", "denied");

    assert_eq!(retry_delay_for(&policy_err, 0, &policy), None);
}
