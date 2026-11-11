---
id: WP-0641
title: Visibility rules per component
phase: 6
epic: E6.5 Builder experience
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0611]
requirements: [REQ-EXP-015]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [experience-studio-pages]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0641 — Visibility rules per component

## Goal

Show the right parts of a page to the right people without building many pages.

## Scope

- Rules by role, by field condition (Business Rules conditions) and by device size, evaluated on the server for role and field conditions
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: A hidden component's data is not sent to the client
- [ ] AC-2: A rule never grants access: a field the user cannot read stays unreadable when a rule shows its component
- [ ] AC-3: Rules are listed in the pre-publish preview
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/page_visibility_rules.rs`
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Align with the shared expression language when WP-0312 lands.
- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
