---
id: WP-0319
title: Budgets at four levels and cost forecast
phase: 3
epic: E3.4 Identity, budget and memory
status: proposed
owner: unassigned
depends_on: [WP-0304]
requirements: [REQ-GOV-013, REQ-GOV-014, REQ-NFR-004]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0319 — Budgets at four levels and cost forecast

## Goal

Forecast before each run; workspace/team/run/step budgets.

## Scope

- Default monthly budget (owner decision, RAID Q-09)

## Acceptance criteria

- [ ] AC-1: 100% of runs show a forecast

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Default monthly workspace budget: **USD 100** per workspace (owner decision 2026-10-16, RAID Q-09). Applies to new and upgraded workspaces; only an AI Admin can raise it. Test that a fresh workspace starts at USD 100 and that a non-admin cannot change it.
- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
