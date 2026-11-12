---
id: WP-0717
title: Three more warehouse lanes and lane templates
phase: 7
epic: E7.6 Shipped lanes
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0707, WP-0710]
requirements: [REQ-LANE-025]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-12
---

# WP-0717 — Three more warehouse lanes and lane templates

## Goal

Show the feature is general, and give admins lanes to start from.

## Scope

- Pick and Pack (sales order), Count (cycle count) and Replenish (reorder rule) lanes on the warehouse app; each offered as a template; Receive Shipment brought to the editions from the demo definition

## Acceptance criteria

- [ ] AC-1: Each lane completes end to end with the records it should change
- [ ] AC-2: Each is available as a template that an admin can install and edit
- [ ] AC-3: The warehouse lanes run in desktop, Team Workspace and the online demo
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_warehouse.rs`; demo E2E
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
