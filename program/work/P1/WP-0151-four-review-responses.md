---
id: WP-0151
title: Four review responses
phase: 1
epic: E1.6 Approvals
status: proposed
owner: unassigned
depends_on: [WP-0150]
requirements: [REQ-GOV-022]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0151 — Four review responses

## Goal

Approve, Edit then approve, Reply with feedback (agent retries), Skip this step; the author chooses which are allowed per gate.

## Scope

- Response model
- Engine behaviour per response
- API and inbox UI buttons

## Acceptance criteria

- [ ] AC-1: Each response drives the run correctly
- [ ] AC-2: Disallowed response rejected by API

## Test plan

- `desktop/core/tests/review_responses.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
