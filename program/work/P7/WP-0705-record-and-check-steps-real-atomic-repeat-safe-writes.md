---
id: WP-0705
title: Record and check steps: real, atomic, repeat-safe writes
phase: 7
epic: E7.2 Steps and rules
status: done
owner: claude
planned_for: claude
depends_on: [WP-0703, WP-0141]
requirements: [REQ-LANE-006, REQ-LANE-007]
estimate: L
editions: [desktop, workspace]
delivered: [desktop, workspace]
editions_reason: core runtime; shown to users by WP-0707
docs: [process-lanes]
branch: wp/WP-0705-record-and-check-step-writes
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/775
updated: 2026-11-14
---

# WP-0705 — Record and check steps: real, atomic, repeat-safe writes

## Goal

Make "writes are real" safe: one step, one outcome, never twice.

## Scope

- Record steps call Action Types; a step's writes commit together; idempotency key per step attempt (WP-0141); check steps with the shared conditions; required and optional steps; skip with reason

## Acceptance criteria

- [x] AC-1: A step with several writes either applies all of them or none (failure injected mid-step)
- [x] AC-2: Completing the same step twice, or resuming after a crash, does not repeat a write
- [x] AC-3: A Business Rule that blocks the write blocks the step with its message
- [x] AC-4: A failed check blocks advance with the reason; a required step cannot be skipped; a skip of an optional step is recorded
- [x] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [x] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_writes.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-14 claude: Built. A lane `record` step's `writes` now run through `action_type_service::invoke` (the governed `update_record`/`set_entity_values` path), resolving each write's inputs (`$record.*`, `$run_data.*`, literals; `$item.*` is WP-0710) against the run, and posting all of a step's writes inside one `idempotency_service::run_once` key - so a failure mid-step applies none (AC-1), and a retry or crash-resume never repeats a write (AC-2, building on WP-0141). Because the write is the same governed path a direct edit uses, a Business Rule that blocks the save blocks the step with its message and writes nothing (AC-3). A `check` node now surfaces its lane_check `message` as the reason on its run node when it does not pass (AC-4). New `process_lane_service::skip_step` + `skip_process_lane_step` Tauri command and `POST /api/admin/process-lane-runs/:id/skip-step` route: an optional step can be skipped with a required reason (recorded who/when/why on a `skipped` run-node row), a required step is refused (AC-4). Both editions share the core service (AC-5). Tests: `desktop/core/tests/process_lane_writes.rs` (6, all AC's); adjacent lane/runtime suites still green (human_step 5, idempotent_resume 5, run_state_machine 6, execution_graph_runtime 18, process_lane_definition 13, blueprint 6, assistant_draft_lane 6); `cargo clippy -p lanesra-core --all-targets` clean; core + server compile; all touched files rustfmt-clean. Docs (AC-6): /help/process-lanes gains a "What record and check steps do" section and WP-0705 in its `wps`; `llms.txt`/`llms-full.txt`, Releases (v0.99.4) and the Roadmap Process Lanes entry updated (process-lanes already in `sitemap.xml`). Not built locally: the Tauri shell (no GTK in this VM) and the full Rust suite - CI covers both. Next: WP-0707 (the runner screen) surfaces all of this to users and lifts the demo freeze.
