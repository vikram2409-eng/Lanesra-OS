# WP-0124 Handoff for Claude

**Date:** 2026-10-18  
**PR:** #678 (branch `wp/WP-0124-retry-backoff-v2`) — OPEN, CI RED  
**Previous PR:** #677 (branch `wp/WP-0124-retry-backoff`) — CLOSED (too many fix commits)

## What's Done

1. **New file `desktop/core/src/services/retry.rs`** (~300 lines):
   - `RetryPolicy { max_attempts: 3, base_delay_ms: 500, max_delay_ms: 20_000 }`
   - `backoff_delay_ms()` — exponential backoff with full jitter (SystemTime-nanos LCG, no `rand` crate)
   - `is_retryable()` — transient only
   - `retry_delay_for()` — honours `retry_after_ms`
   - `Sleeper` trait + `ThreadSleeper`, sync `retry_with_backoff()`
   - Unit tests in module

2. **`desktop/core/src/services/mod.rs`**: Added `pub mod retry;`

3. **`desktop/core/src/services/ai_gateway_service.rs`**:
   - Tier loop retries transient errors in-place with `tokio::time::sleep` backoff
   - Failover per ADR-0009 §4: Transient|Configuration → next tier; Policy|Validation|Internal → return immediately
   - Forced air-gap + Configuration → `config.not_local` fail-closed
   - Fixes RAID I-17

4. **`desktop/core/tests/retry_backoff.rs`** (~240 lines):
   - `TestSleeper` recording delays
   - Tests tagged `Covers: REQ-ORCH-033 (WP-0124 AC-n)`
   - AC-1 (429 then success), AC-2 (policy/validation no retry), AC-3 (backoff timing with controllable clock)

5. **`desktop/core/src/services/ai_service.rs`** — HTTP error classification per ADR-0009 §2:
   - `classify_provider_status(status, retry_after, body)` helper
   - 429 → Transient, `provider.rate_limited` (Retry-After → `retry_after_ms`)
   - 500-599 → Transient, `provider.unavailable`
   - 401/403 → Configuration, `config.missing_key`
   - Other 4xx → Validation, `validation.arguments`
   - `classify_request_error()` for timeout/connect → Transient, `provider.timeout`
   - Unit tests for each row in `classify_tests` module

## What's Broken

**`ai_service.rs` has syntax errors from manual edits.** The file is 1547 lines. My Python scripts that inserted the `retry_after` header extraction line introduced extra `}` delimiters.

Current error (CI run on commit 57902b1):
```
error: unexpected closing delimiter: `}` 
--> desktop/core/src/services/ai_service.rs:535:1
```

Previous errors were at lines 481, 450 — each fix revealed the next one. There are likely more.

## Root Cause

I cannot run `cargo fmt` or `cargo check` locally (no Rust toolchain). I'm editing a 1500+ line file via Python scripts and manual edits without compiler feedback. Each fix is guesswork.

## Recommended Fix

1. Start from main's `ai_service.rs` (clean)
2. Apply the HTTP classification changes carefully, ONE function at a time
3. Use an actual Rust toolchain to verify, OR make minimal surgical edits

The logic is sound — the implementation just has brace-matching errors from automated editing.

## Files on Branch

All 6 files are on `wp/WP-0124-retry-backoff-v2`:
- `desktop/core/src/services/retry.rs` (new, clean)
- `desktop/core/src/services/mod.rs` (clean)
- `desktop/core/src/services/ai_gateway_service.rs` (clean)
- `desktop/core/src/services/ai_service.rs` (**BROKEN** - syntax errors)
- `desktop/core/tests/retry_backoff.rs` (new, clean)
- `program/work/P1/WP-0124-retry-with-backoff-and-error-taxonomy.md` (tracker, `status: in_progress`)

## Tracker Status

WP-0124: `status: in_progress`, `owner: muse`, `branch: wp/WP-0124-retry-backoff-v2`

Do NOT tick ACs — no test has passed yet.
