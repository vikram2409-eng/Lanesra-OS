---
id: WP-0715
title: Describe a process: the Admin Assistant drafts a lane
phase: 7
epic: E7.5 Authoring
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0703, WP-0712]
requirements: [REQ-LANE-022]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-12
---

# WP-0715 — Describe a process: the Admin Assistant drafts a lane

## Goal

A fast start: describe the job, review a drafted lane.

## Scope

- Assistant tool that drafts a lane definition (stages, steps, data fields) from a description, validated against the schema, opened in the blueprint view

## Acceptance criteria

- [ ] AC-1: The result is always a draft and always valid against the schema
- [ ] AC-2: The assistant cannot publish a lane
- [ ] AC-3: The prompt is in the prompt catalog and tested with the scripted provider
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/assistant_draft_lane.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
