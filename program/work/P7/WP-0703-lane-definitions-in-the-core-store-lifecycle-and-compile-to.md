---
id: WP-0703
title: Lane definitions in the core: store, lifecycle and compile to a graph
phase: 7
epic: E7.1 Lane foundation
status: in_progress
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
pr: 
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
