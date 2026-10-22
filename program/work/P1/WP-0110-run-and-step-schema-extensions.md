---
id: WP-0110
title: Run and step schema extensions
phase: 1
epic: E1.2 Run model and trace
status: in_progress
owner: claude
depends_on: [WP-0009, WP-0007, WP-0107]
requirements: [REQ-CORE-008, REQ-API-001]
estimate: M
editions: [desktop, workspace]
delivered: [desktop, workspace]
editions_reason: internal engine work with no user-visible surface of its own; the online demo's simulated runs reflect it through the Playground work packages
docs: []
docs_reason: internal run storage
branch: claude/enterprise-build-planning-jxy2kq
pr: 
updated: 2026-10-22
---

# WP-0110 — Run and step schema extensions

## Goal

Extend `ai_runs` and `ai_run_nodes` (and the graph run tables) with the fields every later feature needs, following the state machine ADR.

## Scope

- Run: status set per ADR-0007 §1, `stop_reason`, `parent_run_id`, `fork_of_step_id`, `budget_json`, `cost_usd`, and the worker lease `lease_owner` / `lease_expires_at` (ADR-0007 §5)
- Step: `tokens_in`, `tokens_out`, `cost_usd`, `context_receipt_json`, `guardrail_verdicts_json`. `attempt` and the timings (`started_at`, `finished_at`) already exist; `completed_partial` is a step status (ADR-0007 §2), not a column
- Retired `waiting_tool` / `waiting_agent` rows become `paused` with `stop_reason = interrupted`
- Repository functions and model structs; one guard, `run_state::transition`, that every run status change in the graph runtime goes through (ADR-0007 §4), taking and renewing the worker lease
- Out of scope: run events for each transition (WP-0112), the interrupted-run sweep (WP-0141), pipeline runs (`ai_agent_runs`, converged by WP-0150), writing the new step fields (WP-0111, WP-0126)

## Acceptance criteria

- [x] AC-1: Migration upgrade test passes with populated fixtures (`0082_runs.db`: the new columns exist, retired statuses are parked as `paused`/`interrupted`, no row is left in a status outside ADR-0007 §1)
- [x] AC-2: Illegal state transitions are rejected by the guard (table-driven test of every pair: all 144 status pairs against ADR-0007 §3 written out separately, on a real database; a run that already moved is not moved again; stop reasons, leases and finish times follow the status; the runtime moves runs only through the guard)
- [x] AC-3: Existing run history still reads correctly (fixture runs, their steps, context and errors read back; a due delayed run and a parked run resume and finish through the guard)

## Test plan

- `desktop/core/tests/run_state_machine.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-22 claude: Refined to ready against ADR-0007 (precise ACs, scope named, out-of-scope items pointed at their work packages) and built. Migration 0083 adds the run and step fields, the lease columns and an index for the sweep, and parks retired statuses. `services::run_state` holds the transition table and the one guarded `UPDATE ... WHERE status = from`; `graph_runtime_service` now moves runs only through it (start, waiting, resume, approval, cancel, fail, complete), and `graph_run_repo` no longer writes status. A run that becomes `running` takes a lease (2 minutes, renewed at every checkpoint) that is dropped when it leaves `running`. The step limit now records `stop_reason = limit_rounds`. Desktop types updated (`GraphRunStatus`, `RunNode`, `GraphRun`). +6 tests in `core/tests/run_state_machine.rs` and 2 unit tests in `run_state`; new upgrade fixture `0082_runs.db` (`gen_fixtures.py --with-runs`).
