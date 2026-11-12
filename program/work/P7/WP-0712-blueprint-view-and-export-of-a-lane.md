---
id: WP-0712
title: Blueprint view and export of a lane
phase: 7
epic: E7.5 Authoring
status: in_progress
owner: muse
planned_for: muse
depends_on: [WP-0703]
requirements: [REQ-LANE-020]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: wp/WP-0712-blueprint-view-and-export
pr: 
updated: 2026-11-12
---

# WP-0712 — Blueprint view and export of a lane

## Goal

Give business owners a page they can read and sign off.

## Scope

- Read-only page: purpose, stage chevrons with their steps (kind, role, writes), run data fields and mapped record fields; export to PDF

## Acceptance criteria

- [ ] AC-1: The view renders any valid lane definition, including branches
- [ ] AC-2: The export matches the view
- [ ] AC-3: The view is readable by a Viewer and exposes no record data
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest for the view; a snapshot of the Receive Shipment blueprint
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-11-12 muse: Claimed — the owner assigned the next two packages to muse ("assign next two work packages to yourself and start building it"). Status proposed → in_progress on branch wp/WP-0712-blueprint-view-and-export. ACs reviewed and stand as written. Next: read ADR-0023, ADR-0024, SOLUTION-DESIGN.md §17, then build the blueprint view + PDF export. Note: this branch stacks on the WP-0703 branch (PR #756, CI red on fmt + one compile error + two clippy errors, fixes in progress) — rebase onto main after #756 merges.
