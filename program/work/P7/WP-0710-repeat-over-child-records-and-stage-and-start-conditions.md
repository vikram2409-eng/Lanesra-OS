---
id: WP-0710
title: Repeat over child records, and stage and start conditions
phase: 7
epic: E7.2 Steps and rules
status: done
owner: muse
planned_for: claude
depends_on: [WP-0705]
requirements: [REQ-LANE-017]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [process-lanes]
branch: wp/WP-0710-demo-and-docs-r2
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/796
updated: 2026-11-18
---

# WP-0710 — Repeat over child records, and stage and start conditions

## Goal

Model line-by-line work and conditional stages instead of hand-building them.

## Scope

- Repeating step over a relationship of the anchor record with per-item validation; stage entry and skip conditions; lane start condition
- Items of a repeating step can be assigned to different people and worked at the same time (ADR-0025 decision 6); the step completes when every item is done

## Acceptance criteria

- [x] AC-1: A repeating step shows one entry per child record and validates each; two people complete different items of the same step at the same time without overwriting each other
- [x] AC-2: A stage whose skip condition holds is skipped and the audit trail says why
- [x] AC-3: A lane cannot start on a record that fails its start condition, and the reason is shown
- [x] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [x] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [x] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_repeat_conditions.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.
- Schema-extension proposal for RAID Q-33..Q-36: `program/spec/schema-extension-proposal-q33-q36.md` (prep track, 2026-11-14). Q-33 proposes `repeat_over.assign` (per-item assignment — this WP implements it on acceptance); Q-34 proposes `skip_when` on a repeating step (this WP implements it on acceptance). Q-35 (`resolve` lookup) is WP-0705's; Q-36 (`role_wh_counter`) is a role-model / seed change, not a schema change. No schema change until the owner accepts each item separately.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-14 muse: Schema-extension proposal for RAID Q-33..Q-36 written at program/spec/schema-extension-proposal-q33-q36.md (prep track, separate PR). Q-33 (`repeat_over.assign`) and Q-34 (`skip_when`) are this WP's to implement on owner acceptance; Q-35 is WP-0705's, Q-36 is a role-model change. This WP stays proposed and unassigned — the proposal settles the design so the engine can build it cleanly; it does not start the build.
- 2026-11-16 claude: AC-1 built. Per-item execution on the lane service: a repeating step (`repeat_over` over a relationship of the anchor) is worked one item per child record. `submit_person_step_item` validates the item's form fields, writes its values to the child record (`record.<coll>[].<field>`, governed path) or a per-child run-data map (`run_data.<key>` → `{item_id: value}`, the dry run's shape), marks the item done under `__items.<step_id>`, and hands off to `drive_child_lanes` which jumps the run past the loop (via the new `graph_runtime_service::resume_lane_run_at_node`) once every item is done; an empty or all-done repeating step advances instead of parking. Per-item assignment (Q-33 `repeat_over.assign` by `child_field`) reads each item's assignee from a child field and gates completion on it. Concurrency-safe: each submit writes only its own item's keys, so two people never overwrite each other. Wired as the `submit_process_lane_step_item` Tauri command and `POST /api/admin/process-lane-runs/:id/submit-item`. Tests: new `core/tests/process_lane_repeat_conditions.rs` (4). Next (mine): AC-2 stage `skip_when` + audit, AC-3 lane start condition + reason. AC-5 demo + AC-6 docs are muse's lane. Do not merge — report when the engine ACs are verified.
- 2026-11-16 claude: AC-2 and AC-3 built. AC-2: a lane stage gate node now carries `lane_gate` (entry/skip) and `lane_stage_name`; the runtime `condition` arm writes a `lane_stage_skipped` audit entry — naming the stage and why — when a skip condition holds or an entry condition fails (keyed on the run's source entity, since the anchor columns are only filled once the initial drive parks). AC-3: `start_lane_run` already refuses a start on a record that fails the lane `start_condition` with a reason; added its test. Tests now 6 in `process_lane_repeat_conditions.rs` (AC-1 ×4, AC-2, AC-3). Remaining: AC-4 is structural (desktop + Team Workspace run the same core; the server route is wired and the HTTP path mirrors `submit-step`), AC-5 demo (`app.js`) and AC-6 docs are muse's lane. Engine ACs (1–3) verified; not merging — reporting.
- 2026-11-18 muse: Claimed the finish (AC-5 demo + AC-6 docs) on branch wp/WP-0710-demo-and-docs from main 68bda93. Engine merged as #789 (e49364c); this branch finishes the WP. DESIGN (read of the engine + demo, 2026-11-18): the demo runner already enforces the start condition (laneStartRun refuses with a toast naming the reason) and already validates per item (per-line form rows, per-line validation, per-item checks), but it (a) submits a repeating form step for ALL lines at once — no per-item completion, (b) does not honour `repeat_over.assign` (builder says "once WP-0710 lands") or `skip_when`, (c) does not evaluate stage entry/skip conditions on stage entry (only the dry run does). No sample lane carries a stage condition, and every stage a sample path reaches is needed on that path — so a stage skip cannot fire on the existing lanes without redesigning one. VEHICLE: the Count lane (lane-wh-count.json), the Q-33/Q-34 proposal's own vehicle — make it runnable in the demo (LANE_WH_COUNT literal + entry point + cycle_count.set_status Action Type + cycle_count approval branch), with `repeat_over.assign: {by: 'child_field', field: 'counted_by'}` on count-lines (seed: counted_by per CC-2026-001 line), `skip_when: {field: '$item.variance', operator: 'equals', value: 0}` on post-count-adjustments, and a `skip_condition` on the Post adjustments stage that fires on the no-variance path (check-variances PASS retargeted to post-adjustments; at entry the count is still COUNTING, so `skip_condition: {field: '$record.status', operator: 'equals', value: 'COUNTING'}` skips the stage, while the approved-variance path enters it with status APPROVED and posts). Per-item completion: each line is its own entry with its own submit; state in runData.__items.<step_id> (mirrors the engine); the step completes when every non-skipped item is done. Assignment is shown per item; the demo is single-user, so the engine (not the demo) enforces the assignee gate — the demo shows who each item is assigned to and completes items independently (the two-people concurrency is proven by the engine tests). Start-condition refusal gains a persistent inline note where the Start button is (the toast alone is transient). AC-6: /help/process-lanes article + sitemap/llms.txt/llms-full.txt/Releases/Roadmap. E2E: extend the lane specs (per-item completion, assign display, skip_when, stage skip, start refusal). Next: build the runner changes in app.js, then the Count lane definition + seed, then E2E, then AC-6 docs.
- 2026-11-18 muse: AC-5 + AC-6 built and proven locally. AC-5: Count lane runnable in the demo (LANE_WH_COUNT literal + dashboard/record entry points, cycle_count.set_status Action Type, warehouse_count_variance approval branch, COUNT_ADJUSTMENT stock moves incl. negative variances); runner honors repeat_over.assign per item (shown as "Assigned to"), skip_when on record steps, stage entry/skip conditions on stage entry (audit wording matches the dry run / engine), per-item Mark done with __items state mirroring the engine, persistent start-refusal card on the record page; Count definition now ships assign (counted_by) + skip_when (variance==0) + Post adjustments skip_condition ($record.status == COUNTING), PASS retargeted to post-adjustments; seed CC-2026-011 (no-variance) added. E2E: new lane-wh-count.spec.ts 4/4 (AC-1 per-item, AC-5 approve-posts-only-nonzero, AC-2 stage skip, AC-3 refusal) + dry-run Count test; regression 41/41 (warehouse-lane, child, blueprint, builder-demo, dry-run) — total 45/45. Two builder-demo tests failed first (they asserted the pre-WP-0710 Count definition with no filter): updated to the shipped definition (filter present; remove/re-add cycle exercises the warning) + restored a "since WP-0710" note in the builder. AC-6: help article gains "Try it: the Count lane" + "Per-item work, stage conditions and start conditions" sections, WP-0710 in wps; Releases v0.99.11; Roadmap Process Lanes entry updated; llms.txt/llms-full.txt "runner honors assign/skip_when since WP-0710"; sitemap already lists /help/process-lanes (no change). AC-4: new server HTTP test per_item_submit_works_over_http_the_same_as_on_desktop (desktop/server/tests/process_lane_http.rs) — submit-item over the admin route, run stays waiting_person after the first item, completes after the last, values land per child. Local proof: core repeat-conditions 6/6, server HTTP 2/2, demo self-test 6/6, E2E 45/45, programctl validate 0/0, node --check OK, cargo fmt clean. WP-0715/WP-0719 already done in their files and in STATUS.md (no fold-in needed). Next: push, open the draft PR, CI is the final proof.
- 2026-11-18 muse: PR #794 came back mergeable_state dirty — only program/STATUS.md conflicts (Claude's WP-0709 merged to main as #792 while the branch was up). Rebuilt fresh from main cb375f3 as wp/WP-0710-demo-and-docs-r2 with the identical tree; re-ran core 6/6, server HTTP 2/2, demo self-test 6/6, node --check OK on the r2 tree. #794 to be closed with a pointer to the new PR.
- 2026-11-18 muse: Draft PR #796 opened (https://github.com/vikram2409-eng/Lanesra-OS/pull/796) from the r2 branch. #794 closed with a pointer to #796.
- 2026-11-18 muse: Draft PR #794 opened (https://github.com/vikram2409-eng/Lanesra-OS/pull/794). Status in_progress → in_review. CI is the final proof.
- 2026-11-18 muse: CI on #796 (head d52935f): 19 successful, 6 neutral, 2 failing — rust (tests 4/4) and the aggregate rust (tests). One test failed: dry_run_count_flags_the_two_zero_variance_lines (desktop/core/tests/process_lane_publish.rs) — expected 1 zero-quantity flag, got 0. Cause (this WP's definition, not a flaky test): the Count lane's new Post adjustments skip_condition ($record.status == COUNTING) fires in the CORE dry run on the approved path too, because the core dry run never applied simulated set_status writes to the simulated record — the approval's cycle_count.set_status → APPROVED was recorded as a write but the anchor status stayed COUNTING, so the stage was skipped and post-count-adjustments was never simulated. The online demo's dry run already applies that rule (app.js laneDryRun: a .set_status write moves simRecord.status). Fix (simulator parity, not a test change): simulate_record_step in desktop/core/src/services/process_lane_publish.rs now applies a non-repeating step's .set_status write to state.record_fields["status"] (anchor only; repeating steps act on their children). Local proof: the failing test passes; full process_lane_publish target 24/24. Next: push the fix to #796; CI re-run is the final proof.
