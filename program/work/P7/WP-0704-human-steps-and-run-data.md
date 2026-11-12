---
id: WP-0704
title: Human steps and run data
phase: 7
epic: E7.1 Lane foundation
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0703]
requirements: [REQ-LANE-004, REQ-LANE-005, REQ-LANE-011]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: core runtime; shown to users by WP-0707
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-12
---

# WP-0704 — Human steps and run data

## Goal

Let a run wait for a person and keep what they enter.

## Scope

- A human step node in the graph runtime with the form step kind; a durable wait; run data with a declared schema and mapping to record fields; role assignment, locked view and access-limited content

## Acceptance criteria

- [ ] AC-1: A run waits at a form step across a restart and resumes when the step is completed
- [ ] AC-2: Every value entered is stored in run data or the mapped record field; none is dropped (test enumerates the schema)
- [ ] AC-3: A viewer without the step's role sees its name and who it waits on and cannot complete it; fields they cannot read are not sent to them
- [ ] AC-4: The trace shows the human step with its wait time
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_human_step.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- WP-0560 (human seats) builds on this node instead of adding its own.
- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
