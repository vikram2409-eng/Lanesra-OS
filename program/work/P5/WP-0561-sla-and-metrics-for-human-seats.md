---
id: WP-0561
title: SLA and metrics for human seats
phase: 5
epic: E5.7 Humans as seats
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0560, WP-0329]
requirements: [REQ-HUM-003]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [human-seats]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0561 — SLA and metrics for human seats

## Goal

Keep hybrid processes moving and measurable.

## Scope

- SLA timers, escalation, out-of-office routing; metrics split by human and agent

## Acceptance criteria

- [ ] AC-1: An overdue item escalates to the backup
- [ ] AC-2: An out-of-office member is skipped
- [ ] AC-3: Team metrics show human and agent time and rework separately

## Test plan

- `desktop/core/tests/human_seat_sla.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
