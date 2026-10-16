---
id: WP-0127
title: Do-not-persist state fields
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0111]
requirements: [REQ-LOOP-021]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0127 — Do-not-persist state fields

## Goal

Fields marked memory-only are never written to checkpoints, traces or audit, and are re-fetched on resume.

## Scope

- Field marking in state and bindings
- Redaction at every persistence hook
- Re-fetch on resume

## Acceptance criteria

- [ ] AC-1: A planted secret value never appears in any table after a run (scan test)
- [ ] AC-2: Resume after restart re-fetches the field

## Test plan

- `desktop/core/tests/do_not_persist.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
