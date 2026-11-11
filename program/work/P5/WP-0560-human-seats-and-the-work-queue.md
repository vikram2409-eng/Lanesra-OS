---
id: WP-0560
title: Human seats and the work queue
phase: 5
epic: E5.7 Humans as seats
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0301, WP-0323]
requirements: [REQ-HUM-001, REQ-HUM-002]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [human-seats]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0560 — Human seats and the work queue

## Goal

Model real processes where some steps are done by people, in the same Team and trace.

## Scope

- Seat kind person or Work Team; work items; queue with claim, reassign, return; form from output schema; validation and guardrails on submit

## Acceptance criteria

- [ ] AC-1: A step on a human seat creates a work item and the run waits durably across a restart
- [ ] AC-2: A submitted result that fails the schema or a guardrail is returned to the person with reasons
- [ ] AC-3: Only members of the seat can claim; reassignment is audited
- [ ] AC-4: The trace shows the human step with its wait time

## Test plan

- `desktop/core/tests/human_seats.rs`; E2E for the queue
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Builds on approvals (WP-0150) and approval routing (WP-0323).
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
