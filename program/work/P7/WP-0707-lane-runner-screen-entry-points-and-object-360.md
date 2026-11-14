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

### 2026-11-14 muse
Mapping done (three parallel read-only maps, decisive claims spot-checked against the code). Findings:
- Engine surface exists at all three layers with no Tauri/HTTP gaps: get_run_view (sync, dispatch.rs:3260), submit_person_step + skip_step (async, admin_actions.rs:101/105), start/resolve/cancel/handoff. No pause entry point (WP-0720 scope). Frontend wrappers + LaneRunView/LaneStepView/FormFieldView types are MISSING from desktop/src/lib/api.ts and lib/types.ts — this WP adds them.
- LaneRunView carries only the CURRENT step (current: Option<LaneStepView>). Stage bar + step list must be composed from the lane blueprint (api.getLaneBlueprint exists) + GraphRun.nodes (stage_id, status, started/finished). No "writes so far" summary — derivable from completed action nodes' output_json (cancel_lane_run already does this, process_lane_service.rs:1705-1716). No anchor-record summary — read through existing record APIs.
- GAPS this WP must close: (1) list-runs-for-record — nothing exists in core (only the repo-private one-active-run guard find_active_run_for_lane_record, process_lane_repo.rs:223); new service fn + Tauri command + dispatch arm + api wrapper. The demo already has it (laneRunsForRecord, app.js:24148). (2) No audit event is recorded for a plain step submit/advance (only cancel/handoff); AC-3's "advances and writes with actor and time" needs audit-on-submit (matching the existing cancel/handoff pattern) and/or exposing run nodes. Decision during implementation; leaning to audit-on-submit. (3) Frontend GraphRun type is stale (no anchor fields, no waiting_person) — runner uses its own LaneRunView types.
- Object 360: desktop/src/components/Object360Modal.tsx, zero lane references today. AC-3 adds a "Lane runs" inspector group + a laneRuns query alongside the existing 8. Audit side is half-ready: cancel/handoff events already flow into the existing Activity group via listAuditEvents.
- Demo runner (app.js:24136-25320): one screen, stage bar (every stage, ✓/numbers joined by ›), step list (current stage only), step work area, right rail (record summary + "What this run has written" + Audit trail), Back/Complete step/Pause/Resume, stacks below 1100px. Wording: "Complete step" (form), "Post and complete step" (record), "Steps — {stage}", "Audit trail". Entry points: dashboard cards, record page "Process Lane" card, agent row, Object 360 inspector.
- DIVERGENCE (RAID Q-37, ANSWERED 2026-11-14 by the owner): the demo keeps its inline approval UI; AC-5 alignment covers the shared first-cut parts/wording only. Desktop first cut shows the inbox state per AC-2. Do not touch the demo's approval flow, branches, Pause/Resume, or seeded runs.
- WP-0621 still proposed — runner builds its own stage-bar and form components.
Next: tests first — vitest component tests for the runner (mocked api, Covers tags), Playwright E2E in both modes (server; no separate Tauri E2E project), axe checks (src/__tests__/a11y.test.tsx pattern). Then implement: api wrappers + types, list-runs-for-record (core), runner screen in features/lanes/, Object 360 group, demo alignment, help + sitemap/llms/Releases/Roadmap.

### 2026-11-14 muse (coordinator)
Fan-out: three parallel subagents on branch wp/WP-0707-lane-runner (PR #779 draft, head 5ae626a). (1) Rust core: LaneRunView extension (steps/writes/waiting) + list-runs-for-record (service + Tauri command + dispatch arm) + audit-on-submit. (2) React frontend: api wrappers + types, runner screen in features/lanes/, entry points, Object 360 "Lane runs" group, axe checks. (3) E2E + demo + docs: Playwright E2E (server mode), app.js alignment of shared first-cut parts/wording only (DO NOT touch demo approval flow/branches/Pause/Resume/seeded runs; Q-37 open), /help/process-lanes + sitemap/llms/Releases/Roadmap. File ownership partitioned; coordinator commits/pushes only. Contract shared by Rust + frontend: list_lane_runs_for_record {object_key, record_id} → {runs: [LaneRunListItem]}; extended LaneRunView adds steps/writes/waiting (snake_case).
Next: assemble subagent results, verify wiring (frontend ↔ Rust contract), run programctl validate + FINISH-CHECKLIST, set in_review, mark #779 ready.

### 2026-11-14 muse (coordinator)
Frontend piece DONE (worker 1/3): types + api wrappers (getLaneRunView/listLaneRunsForRecord/start/submit/skip), LaneRunner.tsx (stage bar, current-stage step list, form/record/check work, Back, inbox state, own stage-bar/form components — WP-0621 still proposed), StartLaneRun entry points (record header + ListTable row action, open-existing instead of second), Object 360 "Lane runs" group. Tests first (7 failed as expected): +35 tests, full suite 110 passed (baseline 75); tsc clean; build pass. Verified against landed Rust code: list_lane_runs_for_record args objectKey/recordId → {runs}; LaneRunView steps/writes/waiting match; NO lane key/name on the view, so page resolves blueprint via run list (test proves it). OPEN for assembly: (1) live wiring unproven (E2E is the check); (2) record steps may never be "current" live — engine returns current only when waiting_person, record steps auto-post; confirm with Rust piece; (3) per-run audit trail needs Rust audit-on-submit. Rust core + E2E/demo/docs still running.
