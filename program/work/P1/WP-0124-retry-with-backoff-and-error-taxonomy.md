---
id: WP-0124
title: Retry with backoff and error taxonomy
phase: 1
epic: E1.3 Agent loop
status: in_progress
owner: claude
depends_on: [WP-0009, WP-0005]
requirements: [REQ-ORCH-033]
threats: [T-06]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-limits-and-partial-results]
branch: wp/WP-0124-retry-backoff-v2
pr: 
updated: 2026-10-18
---

# WP-0124 — Retry with backoff and error taxonomy

## Goal

Transient failures (rate limits, timeouts, 5xx) retry with exponential backoff and jitter, default 3 attempts; policy and validation errors never retry.

## Scope

- Error classification per the error taxonomy ADR
- Retry policy per node with defaults
- Attempts recorded on the step

## Acceptance criteria

- [ ] AC-1: 429 then success → step succeeds on attempt 2
- [ ] AC-2: Policy denial → no retry
- [ ] AC-3: Backoff timing verified with a controllable clock

## Test plan

- `desktop/core/tests/retry_backoff.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

- 2026-10-17 muse: Refined to ready per PROTOCOL.md §4. Threats linked (WP-0011 AC-3). ACs testable, test plan names files, dependencies and requirements listed.

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-17 claude: `AppError::Classified` and `ErrorClass` (ADR-0009 §2) now exist in `core/src/domain/errors.rs` (added in the WP-0005 follow-up), and the scripted provider returns classified transient errors. This work package adds retry with backoff and failover on top of them; it does not need to add the variant.
- 2026-10-18 muse: Implemented retry with backoff (WP-0124). Created `desktop/core/src/services/retry.rs`: RetryPolicy (max 3 attempts, base 500ms, cap 20s), exponential backoff with full jitter, honours retry_after_ms, Sleeper trait for test injection. Updated `ai_gateway_service.rs`: transient errors retry with backoff; failover per ADR-0009 §4 (transient/configuration → next tier; policy/validation/internal → stop; air-gap forced with no local → config.not_local). Fixes RAID I-17 (gateway was failing over on any error). Tests: `desktop/core/tests/retry_backoff.rs` with AC-1 (429 then success), AC-2 (policy/validation no retry), AC-3 (controllable clock verifies backoff timing).
