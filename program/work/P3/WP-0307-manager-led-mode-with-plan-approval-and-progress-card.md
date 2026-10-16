---
id: WP-0307
title: Manager-led mode with plan approval and progress card
phase: 3
epic: E3.2 Modes
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0305, WP-0319]
requirements: [REQ-ORCH-006, REQ-ORCH-007, REQ-ORCH-008, REQ-PG-005, REQ-GOV-006, REQ-API-004]
estimate: XL
branch: 
pr: 
updated: 2026-10-16
---

# WP-0307 — Manager-led mode with plan approval and progress card

## Goal

Plan, preview and approve, execute, check and re-plan, finish.

## Scope

- ai_run_plans

## Acceptance criteria

- [ ] AC-1: Re-plan cap and stall rules tested

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
