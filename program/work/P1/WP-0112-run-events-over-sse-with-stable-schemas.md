---
id: WP-0112
title: Run events over SSE with stable schemas
phase: 1
epic: E1.2 Run model and trace
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0111]
requirements: [REQ-LOOP-005, REQ-OBS-003, REQ-API-005]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0112 — Run events over SSE with stable schemas

## Goal

Stream run events live to the browser over the existing `/api/v1/events/stream`, with versioned, documented schemas, so the Playground (Phase 2) can be built on them.

## Scope

- Events: run.started, step.started, round.completed, tool.called, tool.denied, approval.requested, plan.proposed (reserved), step.completed, run.completed, run.failed
- JSON Schemas in `program/spec/schemas/events/`, envelope with schema version and sequence number
- Authorisation: a client only receives events for runs it may see
- Tauri equivalent for the desktop edition

## Acceptance criteria

- [ ] AC-1: Each event validates against its JSON Schema in tests
- [ ] AC-2: A user without access to a run receives none of its events (tested)
- [ ] AC-3: Event order per run is monotonic by sequence number

## Test plan

- `desktop/server/tests/run_events.rs`; schema validation helper shared with frontend tests
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
