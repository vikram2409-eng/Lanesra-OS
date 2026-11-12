---
id: WP-0640
title: Builder: undo, redo, copy, paste and keyboard
phase: 6
epic: E6.5 Builder experience
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0611]
requirements: [REQ-EXP-014]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [experience-studio-pages]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0640 — Builder: undo, redo, copy, paste and keyboard

## Goal

Make building comfortable and accessible.

## Scope

- Undo and redo history; copy and paste within and across pages; keyboard alternatives for every drag action
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Every edit can be undone and redone
- [ ] AC-2: A page can be built from start to publish with the keyboard only
- [ ] AC-3: Pasting into a page of another object clears bindings that do not exist there and says so
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest; Playwright keyboard-only E2E; axe checks
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
