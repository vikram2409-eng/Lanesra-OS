---
id: WP-0152
title: Approvals bound to exact arguments
phase: 1
epic: E1.6 Approvals
status: proposed
owner: unassigned
depends_on: [WP-0150]
requirements: [REQ-GOV-021]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0152 — Approvals bound to exact arguments

## Goal

An approval covers exactly the arguments shown; any change by retry, guardrail or policy needs approval again.

## Scope

- Argument hash stored with approval
- Check at execute time

## Acceptance criteria

- [ ] AC-1: Changed argument after approval → new approval required (tested)

## Test plan

- `desktop/core/tests/approval_binding.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
