---
id: WP-0707
title: Lane runner, first cut: the straight path on desktop and Team Workspace
phase: 7
epic: E7.3 Runner and visibility
status: in_progress
owner: muse
planned_for: claude
depends_on: [WP-0704, WP-0705]
requirements: [REQ-LANE-014, REQ-LANE-015, REQ-LANE-003]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: wp/WP-0707-lane-runner
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/779
updated: 2026-11-14
---

# WP-0707 — Lane runner, first cut: the straight path on desktop and Team Workspace

## Goal

The one screen a person works on, in desktop and Team Workspace, matching the demo.

## Scope

- Stage bar, step list, step work area, record and run summary, Back and Complete step; phone layout; start from a record and from a list; Object 360 shows runs and their audit trail
- First cut (ADR-0025): form, record and check steps on the straight path. Approval, branch, pause, handoff and cancel screens are WP-0720

## Acceptance criteria

- [ ] AC-1: A user completes a published lane made of form, record and check steps end to end on the runner without opening another screen; Receive Shipment for a fully received order is the reference case
- [ ] AC-2: The stage bar always shows every stage and marks the current and completed ones; a run that reaches an approval or a branch shows a plain "continue in the approvals inbox" state until WP-0720 ships
- [ ] AC-3: Object 360 of the anchor record lists its runs and each run's advances and writes with actor and time
- [ ] AC-4: The runner is keyboard operable, passes axe checks and stacks correctly at phone width
- [ ] AC-5: The demo runner from WP-0702 is aligned with this screen (same parts, same wording)
- [ ] AC-6: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-7: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest; Playwright E2E in both modes with the scripted provider; axe checks
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Uses the Path and Form page components if WP-0621 has shipped; otherwise its own, to be replaced later (record which in the Progress log).
- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

### 2026-11-14 muse
Claim: branch wp/WP-0707-lane-runner from main 1a3b90a; draft PR to open. Taking over from claude (planned_for claude; out of session tokens, owner directed the batch to Muse AI — Claude's handoff 2026-11-14).
Readiness: deps WP-0704 (#770) + WP-0705 (#775) merged; WP-0706 (#777) merged. ACs testable as written; no refinement needed beyond the WP-0621 note below.
WP-0621 (Path/Form components) still proposed — runner builds its own stage-bar and form components, to be replaced when WP-0621 ships.
Engine surface verified on main: get_run_view, submit_person_step, skip_step in desktop/core/src/services/process_lane_service.rs + graph_runtime_service.rs; Tauri commands in desktop/src-tauri/src/commands/process_lane_commands.rs; server dispatch in desktop/server/src/dispatch.rs. Use them, don't re-implement.
Tracker closeout folded in: WP-0704, WP-0705, WP-0706 set done (PRs merged; delivered [desktop, workspace]).
Next: map the existing runner surface + demo runner (app.js, WP-0702) + Object 360; write tests first (vitest + Playwright E2E both modes + axe), then implement.
