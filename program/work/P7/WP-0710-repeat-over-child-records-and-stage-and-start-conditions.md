---
id: WP-0710
title: Repeat over child records, and stage and start conditions
phase: 7
epic: E7.2 Steps and rules
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0705]
requirements: [REQ-LANE-017]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-14
---

# WP-0710 — Repeat over child records, and stage and start conditions

## Goal

Model line-by-line work and conditional stages instead of hand-building them.

## Scope

- Repeating step over a relationship of the anchor record with per-item validation; stage entry and skip conditions; lane start condition
- Items of a repeating step can be assigned to different people and worked at the same time (ADR-0025 decision 6); the step completes when every item is done

## Acceptance criteria

- [ ] AC-1: A repeating step shows one entry per child record and validates each; two people complete different items of the same step at the same time without overwriting each other
- [ ] AC-2: A stage whose skip condition holds is skipped and the audit trail says why
- [ ] AC-3: A lane cannot start on a record that fails its start condition, and the reason is shown
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_repeat_conditions.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
