---
id: WP-0141
title: Idempotency keys and side-effect-safe resume
phase: 1
epic: E1.5 Tools and firewall
status: proposed
owner: unassigned
depends_on: [WP-0110]
requirements: [REQ-TOOL-005, REQ-GOV-023]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0141 — Idempotency keys and side-effect-safe resume

## Goal

Write tools receive an idempotency key per step attempt; on resume after a pause or crash, writes already made are recognised and not repeated.

## Scope

- Key derivation
- Write ledger
- Native writes, connector writes and MCP writes honour the key

## Acceptance criteria

- [ ] AC-1: Kill mid-step then resume → no duplicate record (test with process restart simulation)
- [ ] AC-2: Retry after a transient failure never double-writes

## Test plan

- `desktop/core/tests/idempotent_resume.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
