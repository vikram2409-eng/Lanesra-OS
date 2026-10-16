---
id: WP-0323
title: Organisation-aware approval routing, SLA and escalation
phase: 3
epic: E3.4 Identity, budget and memory
status: proposed
owner: unassigned
depends_on: [WP-0151, WP-0104]
requirements: [REQ-GOV-012, REQ-GOV-024]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0323 — Organisation-aware approval routing, SLA and escalation

## Goal

Route by owner, Org Unit and Work Team with out-of-office fallback.

## Scope


## Acceptance criteria

- [ ] AC-1: Escalation timer tested with controllable clock

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
