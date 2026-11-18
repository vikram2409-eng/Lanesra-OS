---
id: WP-0709
title: Lanes and workflows: declared replacement, and starting a lane from a workflow
phase: 7
epic: E7.2 Steps and rules
status: in_progress
owner: claude
planned_for: claude
depends_on: [WP-0705]
requirements: [REQ-LANE-013]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: wp/WP-0709-lanes-workflows-interplay
pr: 
updated: 2026-11-18
---

# WP-0709 — Lanes and workflows: declared replacement, and starting a lane from a workflow

## Goal

No duplicate work and no hidden suppression.

## Scope

- A step declares the workflow actions it replaces; writes carry their origin (run and step); workflows skip only declared actions for those writes; a workflow action that starts a lane run; both editors and the Dependency Explorer show the link

## Acceptance criteria

- [ ] AC-1: A lane-made change fires every workflow except the actions the step declares it replaces
- [ ] AC-2: The same change made outside a lane fires the workflow in full
- [ ] AC-3: The workflow screen shows which lane steps replace an action, and the lane shows which actions each step replaces
- [ ] AC-4: A workflow starts a lane run on the triggering record
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-7: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_workflow_interplay.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-18 claude: Built the declared-replacement engine (AC-1, AC-2, AC-5). The schema (`LaneStep.replaces: [{workflow_id, action_index}]`) and the P-13 publish check already existed; this adds the runtime. A thread-local `workflow_service::LaneWriteOrigin` (mirroring the existing `DepthGuard`) names the `(workflow_id, action_index)` pairs to skip; the lane runtime enters it around a step's own writes — the record-step action arm in `graph_runtime_service` (via `lane_replaces_from_config`) and the form-step write in `submit_person_step` (via `replaces_suppress_set`). `run_workflow` now enumerates actions and skips exactly the declared ones, recording each skip in the run summary (visible, not hidden). A non-lane write leaves the origin empty, so every action fires. Tests: new `core/tests/process_lane_workflow_interplay.rs` (2: AC-1 skips only the replaced action and the skip shows on the run; AC-2 a direct change fires in full). Local: interplay 2 passed; no regressions (workflow_automation 23, workflow_action_expansion 10, workflow_studio_v2 7, process_lane_writes 6, human_step 5, runner 5, repeat_conditions 6); clippy + fmt clean. Next (mine): AC-4 — a workflow action that starts a lane run on the triggering record (needs async pending-run plumbing, since `apply_action` is sync without a master key). Then AC-3 (editor links), AC-6 (demo), AC-7 (docs) are muse's.
