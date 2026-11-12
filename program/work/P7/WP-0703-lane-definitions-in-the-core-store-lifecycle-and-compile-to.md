---
id: WP-0703
title: Lane definitions in the core: store, lifecycle and compile to a graph
phase: 7
epic: E7.1 Lane foundation
status: in_review
owner: muse
planned_for: muse
depends_on: [WP-0701]
requirements: [REQ-LANE-001, REQ-LANE-002]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: core storage and API; the demo already has its definition from WP-0702
docs: [process-lanes]
branch: wp/WP-0703-lane-definitions-in-core
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/756
updated: 2026-11-12
---

# WP-0703 — Lane definitions in the core: store, lifecycle and compile to a graph

## Goal

Give desktop and Team Workspace the lane definition, on the engine that already runs Workflows and Teams.

## Scope

- Lane tables and API; Draft → Published with versions; compile to an execution graph with source kind `lane`; stage as grouping on nodes; a run keeps its version; Solution export hook

## Acceptance criteria

- [ ] AC-1: A published lane compiles to a graph that passes `validate_for_publish`
- [ ] AC-2: A run started before a new version is published finishes on its own version
- [ ] AC-3: A lane definition from the demo imports unchanged (same schema)
- [ ] AC-4: The upgrade test passes with the new tables
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_definition.rs`; upgrade test
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-11-12 muse: Claimed — the owner reassigned this WP from claude to muse ("build WP-0702 and WP-0703, assign it yourself"). Status proposed → in_progress on branch wp/WP-0703-lane-definitions-in-core. ACs reviewed and stand as written. Next: read ADR-0023, ADR-0024 (§3 compile mapping, §13 demo limits), SOLUTION-DESIGN.md §17–18, execution_graph.rs and migrate.rs, then build the core store + compile. Note: this branch stacks on the WP-0701 branch (PR #755, program check green, CI/security running) — rebase onto main after #755 merges. No local Rust toolchain here: keep changes small, CI is the compiler, expect fmt/clippy fix rounds.
- 2026-11-12 muse: Built (unpushed at this entry; see commits below). Migration 0095 (`process_lanes`, `process_lane_versions`, anchor + run-data columns on `ai_runs`, `stage_id` on `ai_run_nodes`); `models/process_lane.rs` (definition model + validation, unchanged from WP-0701 recon); `repositories/process_lane_repo.rs` + `graph_run_repo` anchor/stage helpers; `services/process_lane_service.rs` (import/publish/create-version, `compile_lane` per ADR-0024 §3, `start_lane_run` with start-condition evaluation and the one-active-run + excludes guards); `human_step` added to NODE_TYPES and `lane` to GRAPH_SOURCE_KINDS; Tauri commands + server dispatch/admin route (same service, both editions); Solution export arm (latest published definition into the manifest; import is a follow-up). Tests: 13 in `core/tests/process_lane_definition.rs` + 1 in `server/tests/process_lane_http.rs`, all tagged Covers REQ-LANE-001/002. Help: `/help/process-lanes` (wps WP-0703) + sitemap + llms.txt + llms-full.txt + Releases v0.99.0 bullet + Roadmap "Proceed" epic; `programctl validate` 0 errors, `docs --check` 65/65, `node --check app.js` passes. NO AC TICKED YET: no Rust test has run anywhere (no local toolchain; CI is the proof) — tick AC-1..AC-6 only when the PR's CI run proves them. NOTE on status: the brief said set `in_review` with `pr:` blank, but `programctl validate` refuses `in_review` without a pr link, so the status stays `in_progress` until the parent opens the PR — then set `in_review` with the link (as WP-0701 did). Decisions recorded: (1) one draft at a time; a new draft supersedes only after the previous one publishes (create_version refuses while a draft exists); (2) the row status is the lifecycle truth, the definition JSON is stored verbatim (imported published definitions keep their inner status field); (3) loop edges are labelled body/exit per the runtime's loop arm, `max_iterations` 1000 safety bound until WP-0710; (4) check/stage-condition runtime shapes derive only literal $record/$run_data conditions with ≤1 OR-group level - expressions, $item refs and compare_field stay verbatim in lane_check/lane_condition for WP-0710, and the node compiles with empty conditions; (5) a run whose first step is a human step fails at that step until WP-0704 - the AC-2 test uses a check-only lane and the one-active-run test uses an approval lane (waiting_approval); (6) start-condition expressions are refused until WP-0710; (7) run-node stage_id is stored via a repo seam (`set_run_node_stage`) for WP-0704 to call - the runtime does not set it yet; (8) the sweep exclusion for the new waiting statuses is WP-0704's, so `waiting_person`/`waiting_child` are NOT added to RUN_STATUSES here. Next: parent opens the PR; watch CI (expect fmt/clippy rounds - read the exact `cargo fmt --check` diff from the log and apply verbatim); then tick ACs from the green run.
