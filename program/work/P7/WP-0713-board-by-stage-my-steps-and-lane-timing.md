---
id: WP-0713
title: Board by stage, "my steps" and lane timing
phase: 7
epic: E7.3 Runner and visibility
status: in_review
owner: muse
planned_for: muse
depends_on: [WP-0707]
requirements: [REQ-LANE-019]
estimate: L
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [process-lanes]
branch: wp/WP-0713-board-my-steps-timing-r4
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/806
updated: 2026-11-20
---

# WP-0713 — Board by stage, "my steps" and lane timing

## Goal

Answer "where is every order" and "what is waiting on me".

## Scope

- Entered and completed times per stage and step; a board of records by stage for a lane; one list of the signed-in person's steps and approvals; time per stage, completion rate and rework per lane; an ageing view of runs longest in their current stage (ADR-0025)

## Acceptance criteria

- [x] AC-1: The board shows each active run in its stage and only records the viewer may read
- [x] AC-2: "My steps" lists lane steps and approvals together, oldest first
- [x] AC-3: Timing figures equal the recorded timestamps (test computes them independently)
- [x] AC-4: The ageing view lists active runs by time in their current stage, longest first, and only records the viewer may read
- [x] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [x] AC-6: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [x] AC-7: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_metrics.rs`; vitest
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Reuses the Board and Record list page components if WP-0620 has shipped.
- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
### 2026-11-19 muse (claim)
- Claimed (branch `wp/WP-0713-board-my-steps-timing`). WP-0711 done-status
  flip folds in here per the no-small-PRs rule.
- Design read before building: REQ-LANE-019; SOLUTION-DESIGN §17 (Board by
  stage shows where every record of a lane is; My steps lists the person's
  lane steps with their approvals). Per-step entered/completed times already
  exist on `ai_run_nodes` (`started_at` / `finished_at` + `stage_id`);
  stage times derive from node rows — no new table needed for timing.
  `list_runs_for_graph` and `list_lane_runs_for_record` exist; the board
  needs a new per-lane runs query. WP-0620 (Board/Record list components)
  is still proposed, NOT shipped, so the board/list UI is built here.
  Open design point: the AC-1/AC-4 access filter ("only records the viewer
  may read") — resolve against the record-level access rule at build time,
  do not guess. Full notes: `~/workspace/lanesra-build/wp0713-design-notes.md`.
### 2026-11-19 muse (build design, recorded before building)
- Access rule RESOLVED from code (not guessed): a run's record is readable
  iff `access_service::require_capability(..., Capability::Read,
  Some(record_id))` passes — the same call `get_run_view` uses
  (REQ-LANE-011). List-level views use its companion
  `access_service::filter_visible` (one evaluator, every caller);
  `None` viewer = system convention, nothing filtered.
- Timing needs no new table: `ai_run_nodes` already carries `stage_id` +
  `started_at` / `finished_at`. Stage entered = earliest node started_at
  in the stage; stage duration = last finished_at - first started_at,
  counted only when every node in the stage finished; rework = node rows
  with attempt > 1.
- Board spans lane versions (graph `source_id` `{lane}@v{n}`); stage
  order from the latest published definition; a run on an older version
  lands in a trailing column named for its stage id. Board shows ACTIVE
  runs only (terminal statuses excluded).
- "My steps": waiting_person runs where `actor_may_act_on_step` passes
  for the viewer (so a reassigned step shows only for its assignee) +
  waiting_approval runs where the viewer holds the approval step's role
  (or is admin; a role-less step is open to anyone, the `actor_may_act`
  convention). Approval attribution: the approval node's config carries
  `lane_step_id`; the approval row's `subject_id` is the run id. Oldest
  first by wait start (node started_at / approval created_at).
- New repo fns: `graph_run_repo::list_lane_runs_for_lane` (board/timing/
  ageing) and `list_lane_runs` (all lane runs in a workspace, for
  "my steps"). Service fns: `list_lane_board`, `list_my_steps`,
  `lane_timing`, `list_lane_ageing` — all in `process_lane_service.rs`,
  shared by desktop and Team Workspace from the same code (AC-5).
- Tests first: `desktop/core/tests/process_lane_metrics.rs` (AC-1..AC-4
  at core level); AC-5 HTTP test follows in `process_lane_http.rs`.
### 2026-11-19 muse (engine facts found while testing — read, not changed)
- Run-node rows carry `stage_id` ONLY for human steps (`set_run_node_stage`,
  WP-0704). Approval/check/record/agent rows have stage_id NULL; their
  stage lives only in the compiled graph node's config (`lane_stage_id`).
  Stage timing therefore resolves each row's stage through the graph
  (`node_stages`/`row_stage` in `process_lane_service.rs`), never from
  the row's own column. Engine untouched (Claude's lane).
- An approval node's row is never marked finished (the engine resumes
  from the next node), so stage duration is entered -> left, where "left"
  is the next stage's entered time or the run's `finished_at` when it is
  the last stage entered and the run is terminal. `run_state::transition`
  DOES set `finished_at` on terminal transitions.
- `user_service::create` eagerly assigns the default access role
  (Standard User, ordinary Read) at creation. A negative access test
  must first `remove_from_user` the Standard User role, then assign only
  the narrow role — otherwise the best scope across roles wins.
- `approval_service::decide` requires an admin actor (its internal `get`
  enforces it); the step-role gate on deciding is the inbox's, not the
  engine's. Tests decide approvals as admin.

### 2026-11-19 muse (AC-5, core + HTTP done, frontend next)
- Tauri commands added (`process_lane_commands.rs`, registered in
  `src-tauri/src/lib.rs`): `list_lane_board`, `list_my_lane_steps`,
  `get_lane_timing`, `list_lane_ageing` — thin wrappers over the same
  service fns the core test proves.
- Team Workspace admin routes (GET, `admin_actions.rs`):
  `/api/admin/process-lanes/:lane_key/board`,
  `/api/admin/process-lanes/my-steps`,
  `/api/admin/process-lanes/:lane_key/timing`,
  `/api/admin/process-lanes/:lane_key/ageing`. Timing is a lane-wide
  aggregate (not viewer-filtered) but still requires an authenticated
  member. `my-steps` is a literal segment; it cannot collide with the
  `:lane_key` routes (different segment counts).
- HTTP test `lane_board_my_steps_timing_ageing_work_over_http`
  (`process_lane_http.rs`, AC-5): check-only lane run completes at once,
  so the board/ageing views are empty by design (active runs only) while
  timing counts the run. Semantics (access filter, ordering) stay in the
  core test, where the fixture is cheap.
- Core green at AC-5: metrics 5/5, lib 56/56, lane suites 6/24/6,
  fmt + clippy clean. HTTP AC-5 test running; frontend views next.

### 2026-11-19 muse (AC-5 done, all layers green)
- HTTP test first failed on a wrong field name (`lane_key` vs the
  structs' `lane_id`) — test fixed, not the service. HTTP AC-5 test
  passes; full `process_lane_http` suite 4/4; clippy server+core clean.
- Frontend: types in `lib/types.ts`, `call` wrappers in `lib/api.ts`,
  `LaneBoard.tsx` (pure views + query sections: board, ageing, my
  steps, timing), wired into `LaneBlueprintPage` (my steps above the
  lane list, board + ageing + timing under the selected lane's
  blueprint) with run rows opening the runner. `tsc --noEmit` clean;
  vitest lanes 68/68; full frontend suite 228/228; `npm run build`
  clean. One tsc miss on the way: the new types were not imported in
  `api.ts` — fixed by adding them to its type import.

### 2026-11-19 muse (AC-6 done, demo green)
- Demo (`app.js`): `laneBoard`, `laneMySteps`, `laneTiming`,
  `laneAgeing`, `laneFormatDuration` — same entered→left rule as the
  core, over the demo's own run/step rows. The demo has one signed-in
  person, so there is no access filter (noted here, not hidden).
  Board page `laneBoardPage` (`current='laneBoard'`), "View board"
  buttons on all five dashboard lane rows, "My steps" across all
  lanes on the board page, run rows open the runner.
- E2E `lane-board.spec.ts` (port 4181) 2/2: Count board after starting
  the lane on CC-2026-001; Receiving board on the seeded PO-2026-005
  run waiting at manager review. Two failures on the way, both test
  mistakes, not product bugs: (1) my `laneAnchorLabel(run)` collided
  with the existing record-shaped `laneAnchorLabel(r)` and the page
  failed to load at all — renamed to `laneRunAnchorLabel`; lesson:
  `node --check` does NOT catch this, the browser load is the real
  check. (2) the count record id is `cc-2026-001`, not `CC-2026-001`
  (the count number). Full demo E2E regression batch green:
  lane-board 2/2, lane-wh-count 4/4, warehouse-lane 5/5,
  lane-anytime-actions 3/3, warehouse-lane-blueprint 1/1,
  lane-runner 2/2 (the runner spec needs its webServer: under
  `CI=true` the config starts no server and :3000 refuses — run it
  without CI after `cargo build -p lanesra-server`, with cargo on
  PATH for the spawned shell).

### 2026-11-19 muse (AC-7 done, docs smoke-verified)
- Help `/help/process-lanes`: new section "Board by stage, my steps
  and lane timing" after the WP-0711 section; summary and `wps:`
  updated (WP-0713 added). Releases: new head entry v0.99.14.
  Roadmap: Process Lanes paragraph gains the WP-0713 sentence.
  `llms.txt` (both Process Lanes bullets) and `llms-full.txt`
  (its Process Lanes paragraph) gain the WP-0713 sentence. Sitemap
  already carries /help/process-lanes — no change. Smoke-verified in
  the browser: help section and v0.99.14 both render.

### 2026-11-19 muse (close-out)
- All 7 ACs proven. Local proof: core lib 56/56, metrics 5/5, lane
  suites publish 6/6, repeat-conditions 24/24, anytime-actions 6/6;
  server HTTP 4/4; clippy core+server clean; fmt clean; tsc clean;
  vitest 228/228; `npm run build` clean; demo E2E regression green
  (see AC-6). No migration, no new Action Type, no engine change.
  Branch is stale (cut from 698d4f6; main has moved past ab1ad77) —
  close-out rebuilds fresh from main via the -r2 pattern.

### 2026-11-20 muse (close-out rebuild -r2)
- #804 (WP-0721 filing) merged as 17685cc; main tip is now 17685cc.
  Fresh branch `wp/WP-0713-board-my-steps-timing-r2` cut from it; all
  20 WP-0713 files copied from the old worktree (14 modified, 5 new,
  plus the WP-0711 done-status flip). WP file branch updated to the
  -r2 branch; pr: still points at #801 until the new PR opens.
- Disk-quota lesson: the full local Rust suite "failure" (exit 101)
  was the linker crashing on a full disk (`ld terminated with signal 7
  [Bus error]`, `Disk quota exceeded (os error 122)`), NOT the code.
  Freed 117G of stale `desktop/target` dirs from merged worktrees
  (builder-gaps 104G + wp0710-finish 13G); the failing test then
  linked and passed. Full re-run in progress; it gates the push.
- Full suite re-run (2026-11-20, after a runtime restart lost the
  first run's log): 1,421 passed, 2 failed — both in
  `core/tests/ssrf_guard.rs` (`an_allow_listed_address_is_reachable`,
  `a_redirect_to_a_refused_address_is_not_followed`; local server on
  127.0.0.3 unreachable in this VM). PROVEN PRE-EXISTING: the same
  target on a clean main checkout (17685cc, no WP-0713 changes) fails
  identically, 8 passed / 2 failed. WP-0713 touches no SSRF code.
  Push proceeds on the targeted green suites + this record.

### 2026-11-20 muse (close-out rebuild -r3)
- Main moved while the r2 push approval timed out: #805 (WP-0721
  Blueprint build, Claude) merged as 28b37ab. The r2 branch (base
  17685cc) is stale again, so this is a fresh -r3 rebuild from
  28b37ab. LESSON (caught by diffing, not by CI): a blind file copy
  from the stale branch would have silently reverted main — #805's
  app.js blueprint markup, #803's home video carousel, and #799's
  WP-0708 engine code in process_lane_service.rs. All three were
  re-applied as 3-way merges with base 698d4f6 (the true common
  ancestor): app.js (3 conflicts, resolved; carousel kept), LaneBlueprint.tsx (clean), llms.txt / llms-full.txt (both keep #805's
  new blueprint sentence + the WP-0713 board sentence),
  process_lane_service.rs (1 conflict, import list only).
- Release entry bumped: WP-0721 took v0.99.14 in #805, so the
  WP-0713 releases entry is now v0.99.15 (head), followed by
  #805's v0.99.14. Help topic summary + wps list carry both
  WP-0713 and WP-0721. pr: still points at #801 until the new PR
  opens.

### 2026-11-21 muse (close-out rebuild -r4)
- #806 (the r3 PR) went ALL GREEN on CI overnight (22 successful, 6
  neutral, 0 failed, 2026-11-21 08:23 PST) but main moved under it:
  #807 (WP-0720 engine, Claude) merged as 99f9a38, and the branch now
  conflicts with main in exactly one file — program/STATUS.md (a
  generated file; #807's regen vs this branch's regen). process_lane_service.rs did NOT conflict.
- So this is a fresh -r4 rebuild from 99f9a38. #807 changed 7 files;
  only process_lane_service.rs overlaps this change set, and the
  3-way merge from base 28b37ab was CLEAN (0 conflicts): #807's
  pause_run / resume_paused_run / reopen_step sit alongside the
  WP-0713 board/my-steps/timing/ageing fns, verified by diff (only
  WP-0713 additions vs new main; the single removed line is main's
  separate `use GraphRun` import, covered by the grouped import).
  STATUS.md is regenerated here, not copied. pr: still points at #806
  until the new PR opens.
