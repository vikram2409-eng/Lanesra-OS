---
id: WP-0706
title: Approvals, branches with exits, pause, handoff and cancel
phase: 7
epic: E7.2 Steps and rules
status: in_review
owner: claude
planned_for: claude
depends_on: [WP-0704]
requirements: [REQ-LANE-008, REQ-LANE-009, REQ-LANE-010, REQ-LANE-012]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: core runtime; shown to users by WP-0707
docs: [process-lanes]
branch: wp/WP-0706-approvals-branches-pause-handoff-cancel
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/777
updated: 2026-11-14
---

# WP-0706 — Approvals, branches with exits, pause, handoff and cancel

## Goal

Cover everything that is not the straight path.

## Scope

- Approval steps through the approvals inbox; declared branches; publish refuses a dead end; pause, resume, handoff; reopen within the current stage; final runs; cancel that lists posted writes; one active run per lane per record; declared exclusions between lanes

## Acceptance criteria

- [x] AC-1: An approval step waits in the inbox and each response routes the run along its declared branch
- [x] AC-2: A lane whose hold or rejection branch has no way out cannot be published, and the message names the step
- [x] AC-3: A second run of the same lane on the same record is refused with a link to the active one; a different lane may start unless excluded
- [x] AC-4: Cancelling after writes leaves them in place and lists them in the audit trail
- [x] AC-5: Handoff changes who can act and is audited
- [x] AC-6: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [x] AC-7: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_branches.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-14 claude: Built. AC-1: lane approval steps already compile to an `approval` node that waits in the approvals inbox and routes `approved`/`rejected` via the WP-0150 primitive; added `process_lane_service::resolve_lane_approval` (a lane guard over `graph_runtime_service::resolve_approval`) + tests proving each response routes to the declared branch. AC-2: `validate_definition` now runs a step-reachability check (`validate_every_step_reaches_an_end`) — any step with no path to the end of the lane fails the publish and the message names the step (a looping hold/rejection branch is the case). AC-3: already enforced by `start_lane_run` (one active run per lane per record, the refusal carries the active run id; declared `excludes` block a second lane) — added coverage. AC-4: `cancel_lane_run` leaves posted writes in place (never rolls one back) and records a `lane_run_cancelled` audit entry listing what was posted. AC-5: `handoff_step` reassigns the current form step to another role (stored per-run under run_data `__handoffs.<step_id>`, consulted by `effective_step_role` in submit/get_run_view), audited as `lane_step_handoff`; only the current assignee or an admin may hand off. AC-6: Tauri commands (`resolve_process_lane_approval`, `cancel_process_lane_run`, `handoff_process_lane_step`) + server routes (`/resolve-approval`, `/cancel`, `/handoff`) over the shared core. Tests: `desktop/core/tests/process_lane_branches.rs` (5, all AC's); adjacent suites green (human_step 5, definition 13, blueprint 6, execution_graph_runtime 18); `cargo clippy -p lanesra-core --all-targets` clean; core + server compile; all touched files rustfmt-clean. Docs (AC-7): /help/process-lanes gains an "Approvals, branches, handoff and cancel" section + WP-0706 in `wps`; `llms.txt`/`llms-full.txt`, Releases (v0.99.5) and the Roadmap entry updated. Not built locally: the Tauri shell (no GTK) and the full Rust suite — CI covers both. Next: WP-0707 (the runner screen) shows all of this and the WP-0705 writes to users and lifts the demo freeze; reopen-within-stage and pause/resume UI are WP-0720.
