---
id: WP-0716
title: Child lanes
phase: 7
epic: E7.2 Steps and rules
status: in_progress
owner: claude
planned_for: muse
depends_on: [WP-0706]
requirements: [REQ-LANE-023]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: wp/WP-0716-child-lanes
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/783
updated: 2026-11-15
---

# WP-0716 — Child lanes

## Goal

Let one process hand part of the job to another and wait.

## Scope

- A step that starts a lane on a related record and waits for its outcome; depth limit; the parent shows the child's stage

## Acceptance criteria

- [x] AC-1: A parent run waits for its child and continues on the child's outcome — the parent sits in `waiting_child` while the child runs, shows the child's stage, and when the child finishes (completed, completed_partial, failed or cancelled) records the outcome in its run data and advances past the step. A child that finishes at once (no waiting step) never parks the parent.
- [x] AC-2: The depth limit is enforced with a readable refusal — a run at depth 3 cannot start a child; the refusal names the limit (3) and the lane chain, is recorded on the parent step, and leaves the parent waiting on the step (retryable, cancellable).
- [x] AC-3: Cancelling the parent offers to cancel or keep the child — cancelling with "cancel child" cancels the child run too; with "keep child" the child keeps running and its finish is a no-op for the cancelled parent. A cancelled child resumes the parent with outcome `cancelled`.
- [x] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [x] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_child.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-11-15 muse: Claimed (branch wp/WP-0716-child-lanes, from main 2a7380d); ACs refined (see above). Design reading, recorded here so the build does not re-litigate it: `child_lane` is a FIELD on a step (ADR-0024 §9, shared schema, model `ChildLane`, publish check P-11) — not a sixth step kind. A step that carries it must be kind `form` with no fields (it compiles to a `human_step` node, the only pausing node the lane service can intercept); `validate_definition` enforces this. The runtime is never touched: the lane service drives the child from its own advance paths (start/submit/resolve/skip post-hooks) — a parent that lands `waiting_person` on a child step starts the child and moves to `waiting_child`; a child that ends terminal resumes its parent through the same hooks. Depth limit is 3 (a top-level run is depth 0; a run at depth 3 cannot start a child); the limit also bounds cycles. Adding `waiting_child` needs two `is_legal` entries in `run_state.rs` (`waiting_person` ↔ `waiting_child`) plus the status itself in `graph_run.rs` — that is this WP's named scope (ADR-0024 §6), not a runtime change. No new publish check: a child lane that cannot start is refused at the step with a readable reason (runtime refusal is the required behaviour); a publish-time child-exists check (P-18) is a follow-up, not this WP. Next: build to the ACs; test file `desktop/core/tests/process_lane_child.rs`, every test tagged `Covers: REQ-LANE-023 (WP-0716 AC-n)`.
- 2026-11-15 claude: Built the core engine (owner handed WP-0716 to Claude as the Rust-heavy piece). `waiting_child` added to `graph_run.rs` + `run_state.rs` (two transitions `waiting_person` <-> `waiting_child`) + the `run_state_machine` oracle (14 statuses, 196 pairs, 31 legal). `validate_definition`: a `child_lane` step must be a `form` step with no fields. The lane service drives children from its own advance hooks (`drive_child_lanes`, called at the end of start/submit/resolve/skip and after cancel): a run waiting at a child step starts the child (`start_child_for`), parks in `waiting_child`, and is resumed (`resume_parent_after_child`) when the child ends; reserved run-data keys `__depth` / `__parent` / `__child` / `__child_outcome`. Depth limit 3 refused with a readable, audited reason naming the limit and the lane chain (AC-2). `cancel_lane_run` gains `master_key` + `cancel_child` (AC-3): cancel-or-keep the child subtree; a cancelled child resumes its parent with outcome `cancelled`. `get_run_view` gains a `child` field (lane + stage) for the waiting parent (AC-1). The runtime (graph_runtime_service/execution_graph) is untouched. Tests: new `desktop/core/tests/process_lane_child.rs` (5, AC-1..AC-3); adjacent suites green (run_state_machine 6, human_step 5, branches 5, writes 6, definition 13, blueprint 6, execution_graph_runtime 18, idempotent_resume 5); `cargo clippy -p lanesra-core --all-targets` clean; core + server compile; touched files rustfmt-clean. Docs (AC-6): /help/process-lanes "Child lanes" section + WP-0716 in `wps`; `llms.txt`/`llms-full.txt`, Releases (v0.99.8) and the Roadmap entry updated (process-lanes already in sitemap). **AC-5 (online demo + demo E2E) is the one AC left** — pure JS on the now-shipped demo runner, no Rust; to be done next (this WP stays `in_progress` until it lands). Not built locally: the Tauri shell (no GTK) and the full Rust suite — CI covers both.
