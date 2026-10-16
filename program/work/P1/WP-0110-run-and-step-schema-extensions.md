---
id: WP-0110
title: Run and step schema extensions
phase: 1
epic: E1.2 Run model and trace
status: proposed
owner: unassigned
depends_on: [WP-0009, WP-0007, WP-0100]
requirements: [REQ-CORE-008, REQ-API-001]
estimate: M
editions: [desktop, workspace]
delivered: []
editions_reason: internal engine work with no user-visible surface of its own; the online demo's simulated runs reflect it through the Playground work packages
docs: []
docs_reason: internal run storage
branch: 
pr: 
updated: 2026-10-16
---

# WP-0110 — Run and step schema extensions

## Goal

Extend `ai_runs` and `ai_run_nodes` (and the graph run tables) with the fields every later feature needs, following the state machine ADR.

## Scope

- Run: status set per ADR, `stop_reason`, `parent_run_id`, `fork_of_step_id`, `budget_json`, `cost_usd`
- Step: `attempt`, `tokens_in`, `tokens_out`, `cost_usd`, `context_receipt_json`, `guardrail_verdicts_json`, timings, `completed_partial`
- Repository functions and model structs; state transition guard function

## Acceptance criteria

- [ ] AC-1: Migration upgrade test passes with populated fixtures
- [ ] AC-2: Illegal state transitions are rejected by the guard (table-driven test of every pair)
- [ ] AC-3: Existing run history still reads correctly

## Test plan

- `desktop/core/tests/run_state_machine.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
