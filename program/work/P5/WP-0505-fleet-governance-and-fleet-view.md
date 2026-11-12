---
id: WP-0505
title: Fleet governance and fleet view
phase: 5
epic: E5.1 Agent control plane
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0503, WP-0504, WP-0405, WP-0319]
requirements: [REQ-FLEET-005, REQ-FLEET-006]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-control-plane]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0505 — Fleet governance and fleet view

## Goal

Apply the same controls to every agent whatever built it, and give an AI Admin one screen for the whole estate.

## Scope

- Policies, data boundaries, budgets and the kill switch cover external agents
- Fleet view with owner, version, state, risk tier, last run, success rate, cost and open findings; filters and export
- Findings: no owner, no recent evaluation, over budget, unapproved definition change

## Out of scope

- Compliance mappings (WP-0591)

## Acceptance criteria

- [ ] AC-1: Pausing an external agent or the workspace refuses its calls at the firewall and at the A2A endpoint; resuming restores them
- [ ] AC-2: An external agent over its budget is refused and the run waits in `waiting_budget`
- [ ] AC-3: The fleet view lists Lanesra and external agents with the same columns and passes axe checks
- [ ] AC-4: A Viewer sees the fleet view read-only; only an AI Admin can pause

## Test plan

- `desktop/core/tests/fleet_governance.rs`; E2E for the fleet view
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Depends on the kill switch (WP-0405) and budgets (WP-0319).
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
