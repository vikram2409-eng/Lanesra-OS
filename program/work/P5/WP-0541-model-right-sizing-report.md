---
id: WP-0541
title: Model right-sizing report
phase: 5
epic: E5.5 Agent optimizer
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0540, WP-0146]
requirements: [REQ-OPT-003]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-optimizer]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0541 — Model right-sizing report

## Goal

Cut run cost with evidence.

## Scope

- Per-step report of the cheapest model that passes the suite; respects data boundaries and provider allow-lists

## Acceptance criteria

- [ ] AC-1: The report lists cost and score per candidate model per step
- [ ] AC-2: Models barred by a data boundary are never suggested
- [ ] AC-3: Applying a suggestion creates a draft version

## Test plan

- `desktop/core/tests/model_right_sizing.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
