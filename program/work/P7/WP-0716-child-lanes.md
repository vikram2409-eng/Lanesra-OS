---
id: WP-0716
title: Child lanes
phase: 7
epic: E7.2 Steps and rules
status: done
owner: claude
planned_for: muse
depends_on: [WP-0706]
requirements: [REQ-LANE-023]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
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
- [x] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
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
- 2026-11-15 muse: AC-5 done (online demo + demo E2E, pure JS, no Rust touched). New example lane `lane-wh-supplier-check` (`program/spec/schemas/examples/lane-wh-supplier-check.json`, on `supplier`, 2 stages / 3 steps, draft): confirm-contact form (contact name must match the record), check-active, mark-verified record step posting `run_data.set` — chosen so P-12 refuses it by name as the brief requires. Receiving lane gains one child step (1.3 `verify-supplier`, after `match-delivery-note`, `child_lane` on `$record.supplier_id` → Supplier Check), 12 → 13 steps. Demo runner (`app.js`): `laneStartChildRun` (depth ≥ 3 refusal naming the limit and chain, re-park if a child is already active, resume-if-finished, at-once finish never parks the parent), `laneResumeParentAfterChild` (records the child's terminal status as `__child_outcome` — `completed`/`cancelled`, as the engine records — clears `__child`, switches the view back to the parent), `laneCancelRun(run, cancelChild)` (cancels the parent first, then the child subtree when asked; a cancelled child resumes its parent), hooks in `laneEnterStep` / `laneSubmitFormStep` (retryable refusal) / `laneFinishRun` / `laneResumeRun` (paused parents pick up a finished child); waiting banner "waiting on Supplier Check — <stage>" with an Open-child-run button, Cancel button + cancel-or-keep prompt (`Cancel child too` / `Keep child running`); dry-run walker reports the child it would start and starts nothing. Ripple: dashboard card 13 steps + Supplier Check row, "four warehouse lanes" → "five" (3 sites + `llms.txt`/`llms-full.txt`), dry-run default anchor `sup-01` + `contact_name` fixture. Proof: `node --check app.js` clean; schema check 5 examples validate; `programctl validate` 0 errors; harness-lane-fixes 86/86; headless child probe 43/43; demo E2E green — new `lane-wh-child.spec.ts` 4/4 (happy path to done with stock 88/52, cancel-child, keep-child, cancel-child-resumes-parent), `warehouse-lane.spec.ts` 5/5 (AC-1/AC-2 now walk through the child step), `warehouse-lane-blueprint.spec.ts` 1/1, `lane-publish-dryrun.spec.ts` 6/6 (new Supplier Check blueprint + dry-run test, Receiving dry run asserts the child line); server-mode regression `lane-runner.spec.ts` 2/2. WP set `in_review`; all six ACs ticked. Next: parent pushes, reviews, merges PR #783 when CI is green.
- 2026-11-15 muse: Test fix (separate commit, per repo AGENTS.md — a wrong test is fixed, never weakened): the Program tracker CI job failed on the AC-5 head because two programctl self-tests asserted the Receiving lane has 12 steps — `test_lane_blueprint_in_demo.py` and `test_lane_definition_in_demo.py`. The AC-5 child step makes it 13, so both now assert 13 and additionally assert the child step itself (id `verify-supplier`, `child_lane` lane `lane-wh-supplier-check` on `$record.supplier_id`). Proof: `python3 -m unittest discover -s program/tools -p 'test_*.py'` 85/85 OK (was 2 failures); `programctl validate` 0 errors; schema check 5 examples validate. Next: push, then merge #783 when CI is green.
- 2026-11-15 muse: Rust test fix (separate commit, per repo AGENTS.md): CI failed on the test-fix head 1204cacd — three Rust tests still asserted the Receiving lane has 12 steps: `process_lane_blueprint.rs` (core, steps.len), `process_lane_blueprint_http.rs` (server, shown_steps.len), `process_lane_definition.rs` (compiled graph nodes 19 / edges 25, plus the verify-stage node count 3). All now assert the 13-step shape (nodes 20, edges 26, verify stage 4 nodes) and additionally assert the child step itself (verify-supplier node is human_step; blueprint tests assert its child_lane lane/anchor). Proof: the three test targets green locally (6/6 + 13/13 + 1/1), fmt clean, clippy 0 warnings. Next: push, then merge #783 when CI is green.
- 2026-11-15 muse: Publish-check test fix (separate commit, per repo AGENTS.md): CI failed on the Rust-fix head 2301317a — `warehouse_lanes_refuse_publish_with_named_p12_reasons` failed P-11 on the child step: the anchor `$record.supplier_id` did not resolve because the test's `seed_warehouse` modelled purchase_order with only `status` + `expected_date`. The real warehouse purchase_order object has `supplier_id` (demo seed: sup-01), so the seed now creates it. The lane and the P-11 check were both right; the fixture was incomplete. (Note: the gate's paraphrase of the P-11/P-12/P-13 reasons was wrong — verified against the real failure output, the only new refusal was the single P-11 entry on verify-supplier.) Proof: `process_lane_publish` 20/20 locally. Next: push, then merge #783 when CI is green.
- 2026-11-15 muse: Merged (PR #783, merge cdfcc07; CI + Program tracker all green on head 8fcfdd81, mergeable clean). Status in_review → done, delivered [desktop, workspace, demo]. Closeout folded into WP-0718's claim commit.
