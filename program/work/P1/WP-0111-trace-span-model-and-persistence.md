---
id: WP-0111
title: Trace span model and persistence
phase: 1
epic: E1.2 Run model and trace
status: proposed
owner: unassigned
depends_on: [WP-0110]
requirements: [REQ-OBS-001, REQ-OBS-002]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: internal engine work with no user-visible surface of its own; the online demo's simulated runs reflect it through the Playground work packages
docs: [run-traces]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0111 — Trace span model and persistence

## Goal

Record a span tree for every run: run → step → round → model call and tool call, with delegations nested as child steps, carrying the attributes in the spec.

## Scope

- Span storage per the trace ADR (table or JSON per step), span ids, parent ids, timings
- Attributes: agent version, model, provider, tokens in/out, cost, latency, policy and guardrail verdicts, context receipt hash
- Instrumentation in chat_service agent loop, pipeline topologies and graph runtime agent nodes
- Performance budget from WP-0015

## Acceptance criteria

- [ ] AC-1: A pipeline run with delegation produces the expected nested tree (golden JSON test)
- [ ] AC-2: Span write overhead within budget
- [ ] AC-3: Do-not-persist hook point exists (consumed by WP-0127)

## Test plan

- `desktop/core/tests/trace_spans.rs` using the scripted provider
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
