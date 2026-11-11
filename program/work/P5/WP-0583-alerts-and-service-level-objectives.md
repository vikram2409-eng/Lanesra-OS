---
id: WP-0583
title: Alerts and service-level objectives
phase: 5
epic: E5.9 Enterprise operations
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0404]
requirements: [REQ-OPS-004]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [observability]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0583 — Alerts and service-level objectives

## Goal

Tell the right person when a Team degrades.

## Scope

- Rule model; evaluation job; notification routes; de-duplication and quiet hours; alert history

## Acceptance criteria

- [ ] AC-1: A rule fires once per incident and resolves
- [ ] AC-2: Quiet hours hold non-critical alerts
- [ ] AC-3: An SLO shows its remaining error budget

## Test plan

- `desktop/core/tests/alerts.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
