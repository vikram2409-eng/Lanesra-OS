---
id: WP-0123
title: Per-step model routing and data boundaries
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0121]
requirements: [REQ-LOOP-003, REQ-GOV-008]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0123 — Per-step model routing and data boundaries

## Goal

Route each step to a model by tier or exact model, through the gateway's fallback chain, and enforce which data classes may reach which provider (sensitive data forced local).

## Scope

- Model tier mapping (Fast, Balanced, Best, Local only)
- Data-class → provider allow-list policy
- Routing decision recorded on the span

## Acceptance criteria

- [ ] AC-1: A step bound to sensitive data is never sent to a non-local provider (tested)
- [ ] AC-2: Fallback chain order is honoured and recorded

## Test plan

- `desktop/core/tests/model_routing.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
