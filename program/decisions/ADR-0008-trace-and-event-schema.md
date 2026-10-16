# ADR-0008: Trace spans and run events share one schema and one channel

- Status: Proposed. Accepted when the owner merges the WP-0009 pull request.
- Date: 2026-10-16
- Work package: WP-0009
- Requirements: REQ-PRE-007, REQ-OBS-003, REQ-LOOP-005, REQ-API-005, REQ-LOOP-021, REQ-PG-018
- Constrains: WP-0111, WP-0112, WP-0113, WP-0121, WP-0128, WP-0145, and every Playground and dashboard work package

## Context

Today nothing streams run progress:

- The only SSE route, `GET /api/v1/events/stream`, polls `integration_executions` once a second.
- That route accepts only API-client Bearer keys, and no screen consumes it (RAID I-21).
- The desktop has no event channel at all.
- Runs export OTLP after the fact (`export_ai_agent_run_otlp`), but nothing records a span tree while the run executes.

Phase 1 needs a live trace in the Playground and a full trace for every run (the Phase 1 gate). Phase 2 adds replay, fork and resumable streams. Those features fit together only if spans, events and their transport are defined once.

## Decision

### 1. Span tree

Every run is one trace. Spans nest as follows:

```
run                       (root; one per run)
└─ step                   (one per node attempt; a delegation is a child step)
   ├─ context             (context assembly: what was included and why)
   ├─ model_call          (one per round)
   ├─ tool_call           (one per tool invocation, child of the round's model_call)
   │  └─ policy           (firewall verdict for that call)
   ├─ guardrail           (one per output check)
   └─ approval            (time spent waiting for a person)
```

- **IDs:** `trace_id` is 32 lowercase hex characters, set once per run. `span_id` is 16 hex characters. These are the OpenTelemetry sizes, so OTLP export needs no mapping.
- **Storage:** spans are kept in a new table, `ai_run_spans`, by WP-0111. Each span has a parent, a kind, a status (`ok`, `error` or `cancelled`), start and end times, and `attributes_json`.
- **Attribute names:** Lanesra's own attributes use the `lanesra.` prefix (for example `lanesra.agent.version_id`, `lanesra.policy.verdict`, `lanesra.context.receipt_hash`). Model-call attributes use the public OpenTelemetry GenAI names (`gen_ai.request.model`, `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens`) so that any collector understands them. Cost is `lanesra.cost_usd`.

### 2. One event envelope

Every live event uses this envelope, on every transport:

```json
{
  "schema": "lanesra.run-event",
  "v": 1,
  "seq": 42,
  "type": "tool.called",
  "at": "2026-10-16T14:03:07.412Z",
  "workspace_id": "ws_…",
  "run_id": "run_…",
  "step_id": "step_…",
  "span_id": "a1b2c3d4e5f60718",
  "parent_span_id": "0f1e2d3c4b5a6978",
  "data": { "tool": "get_invoice", "risk": "read", "verdict": "allow", "duration_ms": 184 }
}
```

- **`seq`:** gapless and increasing within one run, assigned in the same transaction that writes the span or status. It defines the order of events.
- **`type`:** taken from the list in REQ-API-005: `run.started`, `step.started`, `round.completed`, `tool.called`, `tool.denied`, `approval.requested`, `plan.proposed` (reserved), `step.completed`, `run.completed`, `run.failed`. Every status change made through ADR-0007 also emits `run.status_changed`.
- **`data`:** each type has a JSON Schema in `program/spec/schemas/events/<type>.v1.json` (WP-0112). CI validates the examples against these schemas.
- **Versioning:** adding an optional field is allowed within `v: 1`. Removing or renaming a field, or changing its meaning, needs `v: 2` plus a release in which both versions are sent.

### 3. Delivery guarantees

- **Order:** events are delivered in `seq` order within one run. There is no ordering across runs.
- **At least once:** clients drop duplicates by `(run_id, seq)`.
- **Replay:** events are read from the stored spans and status rows, not from a memory buffer. A client that reconnects with SSE `Last-Event-ID: <seq>`, or `?after_seq=`, receives every event after that point. This survives a closed tab and a server restart (REQ-PG-018).
- **Stream levels (REQ-PG-018):**

| Level | Includes |
|---|---|
| `summary` | `run.*` and `step.*` events only |
| `steps` (default) | Adds `round.completed`, `tool.*` and `approval.*` |
| `tokens` | Adds `token.delta` events, which are not stored and never replayed |
| `debug` | Adds full redacted attribute payloads |

### 4. Size limits, bodies and redaction

- An event's `data` must not exceed 16 KiB. Larger content, such as prompts, model outputs or tool results, is stored once and referenced by a `body_ref`; the client fetches it with a separate, permission-checked call.
- **Bodies are off by default, and admins can change it (owner decision, ADR-0013 #2).** The setting is `trace_bodies`:
  - `off` (default): prompt and output bodies are neither stored in the trace nor exported.
  - `redacted`: bodies are stored and exported after the redaction hook below.
  - `full`: bodies are stored and exported as-is, still without secrets or do-not-persist fields.

  It is set per workspace, with an optional override per Team. Only an AI Admin can change it, and every change is audited. The setting applies equally to storage, live events and OTLP export. Each trace records the setting that was in force, so a reader knows whether a missing body was never kept.
- **Redaction:** one redaction hook runs before anything is stored or emitted. It handles secrets (the existing secret redaction), the DLP classes the workspace has set to redact (RAID T-02), and do-not-persist fields (REQ-LOOP-021, which are never written at all).

### 5. One bus, three transports

- An in-process event bus in `lanesra-core` publishes each event after its transaction commits. Nothing polls a table for live events.
- **Team Workspace:** `GET /api/v1/runs/{id}/events` for one run and `GET /api/v1/events/stream?run_id=` for several. The UI authenticates with its session cookie and API clients with Bearer keys (`events.read` scope).
- **Authorisation:** a client receives events only for runs it is allowed to see (WP-0100 record scope).
- **Desktop:** a Tauri event channel named `run-event` carries the identical envelope.
- **Online demo:** the demo emits the same envelope from its simulator, so the same Playground code renders all three editions.
- The existing integration events stay on the current route, under their own schema name (`lanesra.integration-event`).

## Example

Tool call denied by the firewall in round 2 of step `lookup`:

```json
{"schema":"lanesra.run-event","v":1,"seq":17,"type":"tool.denied","at":"2026-10-16T14:03:09.002Z",
 "workspace_id":"ws_1","run_id":"run_9","step_id":"step_3","span_id":"7a6b5c4d3e2f1a0b","parent_span_id":"1a2b3c4d5e6f7a8b",
 "data":{"tool":"delete_record","risk":"destructive","verdict":"deny","reason":"blocked by policy 'No deletes'","round":2}}
```

## Consequences

- The Playground, the dashboards, the OTLP export and the API all read one stored span tree, so a trace looks the same everywhere and replay is exact.
- Assigning `seq` adds one counter update to each event's transaction. This is acceptable at SQLite scale; the worker pool in Phase 4 keeps one writer per run.
- The old integration SSE route stays as it is, so existing API clients are unaffected.
