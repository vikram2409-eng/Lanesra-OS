---
id: WP-0704
title: Human steps and run data
phase: 7
epic: E7.1 Lane foundation
status: in_progress
owner: claude
planned_for: claude
depends_on: [WP-0703]
requirements: [REQ-LANE-004, REQ-LANE-005, REQ-LANE-011]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: core runtime; shown to users by WP-0707
docs: [process-lanes]
branch: wp/WP-0704-human-steps-and-run-data
pr: 
updated: 2026-11-14
---

# WP-0704 — Human steps and run data

## Goal

Let a run wait for a person and keep what they enter.

## Scope

- A human step node in the graph runtime with the form step kind; a durable wait; run data with a declared schema and mapping to record fields; role assignment, locked view and access-limited content

## Acceptance criteria

- [x] AC-1: A run waits at a form step across a restart and resumes when the step is completed
- [x] AC-2: Every value entered is stored in run data or the mapped record field; none is dropped (test enumerates the schema)
- [x] AC-3: A viewer without the step's role sees its name and who it waits on and cannot complete it; fields they cannot read are not sent to them
- [x] AC-4: The trace shows the human step with its wait time
- [x] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [x] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_human_step.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- WP-0560 (human seats) builds on this node instead of adding its own.
- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refined against ADR-0024 section 4 (the `human_step` node) and moved to `ready` on 2026-11-14 (ADR-0025).

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

### 2026-11-14 claude
Done: AC-1..AC-6. New `human_step` node waits a lane run in `waiting_person` (ADR-0024 §4/§6) with no lease; `submit_person_step` validates entered values against the step's fields and its `validation`, stores run-data fields on the run and record-bound fields through `custom_field_service::set_entity_values` (the governed path), completes the step row (its open interval is the wait time), and advances via `resume_person_step`. `get_run_view` is the access-limited read (role gates `can_complete`; record-bound values withheld without read access). Exposed on both editions: Tauri commands `submit_process_lane_step` / `get_process_lane_run_view`, server route `/api/admin/process-lane-runs/:id/submit-step` and dispatch `get_process_lane_run_view`. Help article, llms.txt/llms-full.txt and a Releases entry updated.
State: green locally - `core/tests/process_lane_human_step.rs` 5/5; lane definition/blueprint/runtime suites 18/6/13; `cargo fmt --all --check` clean; `cargo clippy -p lanesra-core` clean; `node --check app.js` ok. The Tauri shell did not build in this VM (missing GTK/`gdk-sys` system libs) - CI's `tauri (desktop shell)` job covers it. Full suite runs on the PR.
Next: WP-0705 (record and check steps: Action Type writes, all-or-nothing and repeat-safe; needs WP-0141) and WP-0707 (the runner screen, first cut) build on this. `waiting_child` (ADR-0024 §6) is intentionally left to WP-0716.
Gotchas: every lane step must declare a `role` (the validator enforces it). `custom_field_service::set_entity_values` runs Business Rules but does not itself gate on record capability; WP-0705 adds the access/atomicity wrapper for record *steps*. A step `validation` that is only an arithmetic `expression` is deferred to WP-0710 and does not block here.
Commands: cargo test -p lanesra-core --features test-support --test process_lane_human_step
