---
id: WP-0113
title: Full-trace conformance suite (Phase 1 gate test)
phase: 1
epic: E1.2 Run model and trace
status: proposed
owner: unassigned
depends_on: [WP-0112, WP-0122, WP-0150, WP-0100]
requirements: [REQ-OBS-003, REQ-NFR-003]
estimate: M
editions: []
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0113 — Full-trace conformance suite (Phase 1 gate test)

## Goal

Prove the Phase 1 gate - every run produces a full trace - and that 100% of tool calls have a complete audit trail, across every way a run can start.

## Scope

- Matrix: chat agent, pipeline (sequential, consensus, peer review), workflow agent action, graph agent node, schedule trigger, webhook trigger, MCP-initiated, CLI-initiated
- For each: trace completeness check and tool audit completeness check

## Acceptance criteria

- [ ] AC-1: Every matrix cell passes
- [ ] AC-2: Suite is part of the regression gate

## Test plan

- `desktop/core/tests/trace_conformance.rs`, `desktop/server/tests/trace_conformance.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
