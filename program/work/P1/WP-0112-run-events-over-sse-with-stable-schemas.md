---
id: WP-0112
title: Run events over SSE with stable schemas
phase: 1
epic: E1.2 Run model and trace
status: in_review
owner: claude
depends_on: [WP-0111]
requirements: [REQ-LOOP-005, REQ-OBS-003, REQ-API-005]
estimate: L
editions: [desktop, workspace]
delivered: [desktop, workspace]
editions_reason: a server event stream; the online demo generates the same events client-side
docs: [api-access-and-webhooks, run-traces]
branch: claude/enterprise-build-planning-jxy2kq
pr: 728
updated: 2026-10-30
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

- [x] AC-1: Each event validates against its JSON Schema in tests
- [x] AC-2: A user without access to a run receives none of its events (tested)
- [x] AC-3: Event order per run is monotonic by sequence number
- [x] AC-4: Publishing a run's events stays within the trace budget agreed for WP-0015: span writes and event publishing together add < 5 ms per round

## Test plan

- `desktop/core/tests/run_events.rs` and `desktop/server/tests/run_events.rs`, sharing the schema checker `desktop/core/tests/support/event_schema.rs`; `desktop/src/__tests__/runEvents.test.ts` for the browser client
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-30 claude: Refined proposed → ready and claimed. Dependency met (WP-0111 done); ADR-0007 and ADR-0008 Accepted. Decisions: events are stored in their own table (`ai_run_events`, migration 0087) written as spans open and close, so replay is exact and survives a restart (ADR-0008 §3 asks for replay from storage, not memory). Who may see a run: runs do not record who started them yet and both run kinds are Administrator-only to read today, so a signed-in user must be an Administrator; API clients need `events.read`. Record scope (WP-0100) widens this later. The `tokens` and `debug` stream levels wait for the Playground (Phase 2) and are refused with 400 until then. The online demo is out of scope (editions_reason).
- 2026-10-30 claude: Built. `trace_service` writes `run.started`, `step.started`/`step.completed`, `round.completed`, `tool.called`, `tool.denied`, `approval.requested` (firewall hold, graph Approval node, pipeline approval step), `run.completed` and `run.failed` with a gapless per-run `seq`, through the persist hooks (so do-not-persist values are scrubbed), and `run_event_service` publishes them on an in-process bus. Team Workspace: `GET /api/v1/runs/:id/events` and `GET /api/v1/events/stream?run_id=` (SSE, `Last-Event-ID`/`after_seq` replay, `level=summary|steps`, session cookie for Administrators or `events.read`), and the `list_run_events` command. Desktop: the `run-event` Tauri channel (forwarded while the signed-in user is an Administrator) and `list_run_events`. First consumer: the Agent Teams test run follows its events instead of polling every 2 s. JSON Schemas in `program/spec/schemas/events/`. Tests: 4 core (every type validates, gapless across pause and resume, bus order, < 5 ms per round, admin-only read, the checker refuses broken events), 4 server (live follow of a paused run, AC-2 refusals, replay and levels, multi-run stream), 3 frontend.
- 2026-10-30 claude: In review in #728.
