---
id: WP-0112
title: Run events over SSE with stable schemas
phase: 1
epic: E1.2 Run model and trace
status: proposed
owner: unassigned
depends_on: [WP-0111]
requirements: [REQ-LOOP-005, REQ-OBS-003, REQ-API-005]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: a server event stream; the online demo generates the same events client-side
docs: [api-access-and-webhooks, run-traces]
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
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - No usable channel exists today: the SSE stream is API-key only, polls `integration_executions` and has no consumer (I-21). Build an in-process event bus, session-cookie auth for the UI, a Tauri event channel for desktop, and the first frontend consumer (the UI polls with react-query today)
- Owner decisions (ADR-0013, ADR-0014, 2026-10-16):
  - Team Workspace is used only through the browser (ADR-0013 #5). The Tauri channel serves the desktop edition's own runs; the desktop never connects to a server

## Acceptance criteria

- [ ] AC-1: Each event validates against its JSON Schema in tests
- [ ] AC-2: A user without access to a run receives none of its events (tested)
- [ ] AC-3: Event order per run is monotonic by sequence number
- [ ] AC-4: Publishing a run's events stays within the trace budget agreed for WP-0015: span writes and event publishing together add < 5 ms per round

## Test plan

- `desktop/server/tests/run_events.rs`; schema validation helper shared with frontend tests
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
