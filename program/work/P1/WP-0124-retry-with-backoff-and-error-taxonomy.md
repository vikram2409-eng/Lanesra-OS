---
id: WP-0124
title: Retry with backoff and error taxonomy
phase: 1
epic: E1.3 Agent loop
status: in_review
owner: claude
depends_on: [WP-0009, WP-0005]
requirements: [REQ-ORCH-033]
threats: [T-06]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [agent-limits-and-partial-results]
branch: wp/WP-0124-retry-backoff-v2
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/678
updated: 2026-10-22
---

# WP-0124 — Retry with backoff and error taxonomy

## Goal

Transient failures (rate limits, timeouts, 5xx) retry with exponential backoff and jitter, default 3 attempts; policy and validation errors never retry.

## Scope

- Error classification per the error taxonomy ADR
- Retry policy per node with defaults
- Attempts recorded on the step

## Acceptance criteria

- [x] AC-1: 429 then success → step succeeds on attempt 2
- [x] AC-2: Policy denial → no retry
- [x] AC-3: Backoff timing verified with a controllable clock

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
- 2026-10-20 claude: Rescued from Muse's handoff. Restarted `ai_service.rs` from main and classified provider failures per ADR-0009 §2 in all six completion paths (`provider_status_error`, `provider_request_error`, `retry_after_ms`): 429 → `provider.rate_limited` with Retry-After, 5xx → `provider.unavailable`, timeout → `provider.timeout`, other network failure → `provider.network`, 401/403 → `config.missing_key`, 404 → `config.model_not_found`, other 4xx → `validation.arguments`. Replaced Muse's unused sync retry helper with one async `retry::retry` and a `RetryClock` (sleep and jitter), used by both the gateway tier loop and the unrouted `complete_with_tools` path. Retry-After is capped at 20 s. `resolve_tier` errors are now configuration errors, so failover still happens on a missing key or disabled provider. Removed the claim that this fixes RAID I-17 (it does not; that stays with WP-0123). AC-1 is proven on a real run (429 then 200, two requests); AC-2 and AC-3 with the controllable clock. Still open from Scope: a retry policy per node and attempts recorded on the step need the step table from WP-0110; proposed as follow-up there.
- 2026-10-22 claude: PR #678 merged. Retry and error classification run in the shared core (desktop and Team Workspace). Still open before `done`: the help article `agent-limits-and-partial-results` (shared with WP-0121, WP-0125, WP-0126) and the online demo edition (RAID Q-21).
- 2026-10-22 claude: Online demo edition built (RAID Q-21): Admin → LLM & MCP → Gateway → Retries and failure handling simulates a provider failure with the same policy as `retry.rs` (3 attempts, 500 ms window doubling to a 20 s cap, Retry-After capped at 20 s, only transient errors retried, configuration errors fail over), on a simulated clock. Help article `agent-limits-and-partial-results` written (error classes, retries, failover; step limits, compaction and budgets named as still being built for WP-0121, WP-0125 and WP-0126) and added to the sitemap, llms files and Releases. Verified in headless Chromium, including the backoff windows against the Rust unit test values.
