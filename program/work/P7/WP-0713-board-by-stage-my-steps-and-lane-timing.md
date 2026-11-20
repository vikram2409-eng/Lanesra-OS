---
id: WP-0713
title: Board by stage, "my steps" and lane timing
phase: 7
epic: E7.3 Runner and visibility
status: in_progress
owner: muse
planned_for: muse
depends_on: [WP-0707]
requirements: [REQ-LANE-019]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: wp/WP-0713-board-my-steps-timing
pr: 
updated: 2026-11-19
---

# WP-0713 — Board by stage, "my steps" and lane timing

## Goal

Answer "where is every order" and "what is waiting on me".

## Scope

- Entered and completed times per stage and step; a board of records by stage for a lane; one list of the signed-in person's steps and approvals; time per stage, completion rate and rework per lane; an ageing view of runs longest in their current stage (ADR-0025)

## Acceptance criteria

- [ ] AC-1: The board shows each active run in its stage and only records the viewer may read
- [ ] AC-2: "My steps" lists lane steps and approvals together, oldest first
- [ ] AC-3: Timing figures equal the recorded timestamps (test computes them independently)
- [ ] AC-4: The ageing view lists active runs by time in their current stage, longest first, and only records the viewer may read
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-7: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

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
