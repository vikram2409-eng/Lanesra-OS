---
id: WP-0404
title: OTLP export, orchestration dashboard and cost reports
phase: 4
epic: E4.2 Compliance and governance
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0111]
requirements: [REQ-OBS-004, REQ-OBS-005, REQ-GOV-015]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0404 — OTLP export, orchestration dashboard and cost reports

## Goal

Admin-configured export; dashboards; monthly reports.

## Scope


## Acceptance criteria

- [ ] AC-1: Bodies excluded by default

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
