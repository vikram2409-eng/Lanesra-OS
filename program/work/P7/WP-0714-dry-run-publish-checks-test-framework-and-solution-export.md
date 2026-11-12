---
id: WP-0714
title: Dry run, publish checks, test framework and Solution export
phase: 7
epic: E7.5 Authoring
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0705, WP-0706]
requirements: [REQ-LANE-021]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-12
---

# WP-0714 — Dry run, publish checks, test framework and Solution export

## Goal

Make a lane safe to publish and portable.

## Scope

- Dry run with writes simulated; publish checks for bindings, roles, branch exits and replaced workflows; lane cases in the Unified Test Run; Solution export and import

## Acceptance criteria

- [ ] AC-1: A dry run walks the lane and writes nothing
- [ ] AC-2: Publish is refused with a named reason for each failed check
- [ ] AC-3: A lane exports and imports in a Solution with its versions
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_publish.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
