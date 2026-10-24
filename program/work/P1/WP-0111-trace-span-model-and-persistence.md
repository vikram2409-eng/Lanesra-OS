---
id: WP-0111
title: Trace span model and persistence
phase: 1
epic: E1.2 Run model and trace
status: done
owner: claude
depends_on: [WP-0110]
requirements: [REQ-OBS-001, REQ-OBS-002]
estimate: L
editions: [desktop, workspace]
delivered: [desktop, workspace]
editions_reason: internal engine work with no user-visible surface of its own; the online demo's simulated runs reflect it through the Playground work packages
docs: [run-traces]
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/720
updated: 2026-10-24
---

# WP-0111 — Trace span model and persistence

## Goal

Record a span tree for every run: run → step → round → model call and tool call, with delegations nested as child steps, carrying the attributes in the spec.

## Scope

- Span storage per the trace ADR (table or JSON per step), span ids, parent ids, timings
- Attributes: agent version, model, provider, tokens in/out, cost, latency, policy and guardrail verdicts, context receipt hash
- Instrumentation in chat_service agent loop, pipeline topologies and graph runtime agent nodes
- Performance budget from WP-0015
- Owner decisions (ADR-0013, ADR-0014, 2026-10-16):
  - Admin setting `trace_bodies`: `off` (default), `redacted` or `full`, per workspace with a per-Team override; applied to storage, live events and OTLP export; AI Admin only; audited; each trace records the setting in force (ADR-0008 §4)

## Acceptance criteria

- [x] AC-1: A pipeline run with delegation produces the expected nested tree (golden JSON test): run > step > model_call > tool_call > policy, with the delegation as a child step of its tool call
- [x] AC-2: Writing a run's spans adds less than 5 ms per round (the budget agreed by the owner on 2026-10-23, from WP-0015), measured by the time spent in span writes over an 8-round run
- [x] AC-3: Do-not-persist hook point exists (consumed by WP-0127): every span's attributes and bodies pass through registered hooks before they are written

## Test plan

- `desktop/core/tests/trace_spans.rs` using the scripted provider
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-23 claude: Refined proposed → ready, then claimed. The owner agreed WP-0015's proposed budget (trace writes add < 5 ms per round), so AC-2 is now testable; AC-1 and AC-3 made precise. Dependencies met (WP-0110 done); ADR-0008, ADR-0010, ADR-0012 and ADR-0013 Accepted. Scope note: ADR-0013's per-Team override of `trace_bodies` needs Teams, which do not exist yet; it is split into WP-0198 (proposed).
- 2026-10-24 claude: Built.
  - Migration 0085: `ai_run_spans` (trace and span ids at OpenTelemetry sizes, parent, run kind and id, seq, kind, name, status, times, attributes), `ai_run_span_bodies`, and `ai_settings.trace_bodies` (default `off`).
  - `services::trace_service`: `begin_run` opens or reopens a run's trace; `start`/`Span::end` write child spans under the innermost open span, kept on a thread-local stack (a run and its delegations execute on one thread, as `principals` already relies on). Tracing never fails a run. A waiting run keeps its root open and resumes in the same trace; a cancelled graph run closes it as cancelled.
  - Instrumented: pipeline runs (`run_internal`, resumes, refused queued runs) and steps; graph runs (`run_from`) and every node attempt; each round's model call (GenAI names for provider, model and tokens; latency; tier); each tool call (outcome ok, error, blocked or malformed); the firewall verdict as a policy span; the output-schema check as a guardrail span; delegation as a child step. No cost attribute yet: prices come with the model catalog (WP-0146).
  - `trace_bodies` (ADR-0008 §4): off keeps no bodies; redacted applies DLP redaction; full keeps them as-is; agent credentials are removed in every mode; bodies cut at 64,000 characters; set by an Administrator only and audited; each root records the mode in force. Bodies are read through an Administrator-only call.
  - Readers: `spans_for_run`, `span_bodies`; Tauri commands and server dispatch `set_ai_trace_bodies`, `list_run_spans`, `get_run_span_bodies`; the setting in Admin → LLM & MCP → Gateway → Observability.
  - Tests: `core/tests/trace_spans.rs` (7): the golden delegation tree, graph steps with a waiting run resumed in the same trace and a cancelled one closed, the overhead budget (about 0.7 ms per round in a debug build), the persist hook, the three body modes, admin-only audited setting, and no spans outside a run.
  - Help article `run-traces` added.
- 2026-10-24 claude: In review in #720.
- 2026-10-24 claude: Merged in #720. Done.
