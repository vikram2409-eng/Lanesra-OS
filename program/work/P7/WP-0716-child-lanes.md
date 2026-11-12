---
id: WP-0716
title: Child lanes
phase: 7
epic: E7.2 Steps and rules
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0706]
requirements: [REQ-LANE-023]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-12
---

# WP-0716 — Child lanes

## Goal

Let one process hand part of the job to another and wait.

## Scope

- A step that starts a lane on a related record and waits for its outcome; depth limit; the parent shows the child's stage

## Acceptance criteria

- [ ] AC-1: A parent run waits for its child and continues on the child's outcome
- [ ] AC-2: The depth limit is enforced with a readable refusal
- [ ] AC-3: Cancelling the parent offers to cancel or keep the child
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_child.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
