---
id: WP-0308
title: Round Table mode, speaker rules and seat visibility
phase: 3
epic: E3.2 Modes
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0305]
requirements: [REQ-ORCH-009, REQ-ORCH-038, REQ-ORCH-040]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0308 — Round Table mode, speaker rules and seat visibility

## Goal

Turn-based discussion.

## Scope

- ai_seat_visibility

## Acceptance criteria

- [ ] AC-1: Selector rules tested

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
