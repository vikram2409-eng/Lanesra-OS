---
id: WP-0504
title: Trace ingest for external agent runs
phase: 5
epic: E5.1 Agent control plane
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0501, WP-0404]
requirements: [REQ-FLEET-004]
estimate: L
editions: [workspace]
delivered: []
editions_reason: needs a reachable server; the desktop app has no public endpoint
docs: [agent-control-plane, observability]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0504 — Trace ingest for external agent runs

## Goal

Show what external agents did in the same run history as Lanesra agents, so cost and behaviour are reviewed in one place.

## Scope

- OTLP receiver (HTTP) behind a scoped key per registered agent
- Mapping of generative-AI span conventions onto run, step, round, model call and tool call (ADR-0008)
- Ingested runs marked externally reported and read-only; bodies follow the trace-bodies setting
- Size, rate and retention limits per agent

## Out of scope

- Replay or fork of ingested runs

## Acceptance criteria

- [ ] AC-1: A posted trace appears as a run with steps, tokens and cost under the registered agent
- [ ] AC-2: A trace for an unknown agent, with a bad key or over the size limit is refused and counted
- [ ] AC-3: Ingested runs cannot be replayed, forked or edited and say why
- [ ] AC-4: Body redaction applies to ingested spans

## Test plan

- `desktop/server/tests/trace_ingest.rs` with recorded span fixtures
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR from WP-0114; ADR-0008.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
