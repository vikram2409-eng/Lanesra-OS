---
id: WP-0624
title: Blocks: reusable components
phase: 6
epic: E6.3 Components and blocks
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0611]
requirements: [REQ-EXP-010]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [reusable-blocks]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0624 — Blocks: reusable components

## Goal

Let an admin configure a component once and reuse it across screens.

## Scope

- Save a component or group as a block with inputs; insert linked or as a copy; versions with Draft → Published; where-used list; Solution export; Dependency Explorer
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Publishing a new version of a linked block changes every page that uses it; a copy is unaffected
- [ ] AC-2: A block cannot be deleted while a published page uses it; the admin sees where
- [ ] AC-3: Blocks export and import in a Solution with their versions
- [ ] AC-4: A block cannot contain itself (cycle refused)
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-7: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/page_blocks.rs`; upgrade test; vitest
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
