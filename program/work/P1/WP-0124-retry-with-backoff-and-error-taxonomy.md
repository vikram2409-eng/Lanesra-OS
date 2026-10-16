---
id: WP-0124
title: Retry with backoff and error taxonomy
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
depends_on: [WP-0009, WP-0005]
requirements: [REQ-ORCH-033]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-limits-and-partial-results]
branch: 
pr: 
updated: 2026-10-16
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

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
