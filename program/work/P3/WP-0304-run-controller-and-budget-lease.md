---
id: WP-0304
title: Run controller and budget lease
phase: 3
epic: E3.1 Teams and compiler
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0303]
requirements: [REQ-ORCH-003, REQ-ORCH-016]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0304 — Run controller and budget lease

## Goal

Admit, start, pause, budget, replay and fork in one controller.

## Scope

- Takes over from ai_orchestration_service

## Acceptance criteria

- [ ] AC-1: waiting_budget on exhaustion

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
