---
id: WP-0709
title: Lanes and workflows: declared replacement, and starting a lane from a workflow
phase: 7
epic: E7.2 Steps and rules
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0705]
requirements: [REQ-LANE-013]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-12
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
