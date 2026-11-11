---
id: WP-0621
title: Form and content components
phase: 6
epic: E6.3 Components and blocks
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0611]
requirements: [REQ-EXP-008]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [page-components]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0621 — Form and content components

## Goal

Forms, steps and content anywhere on a page.

## Scope

- Form (single or multi-step, object field rules), image, link list, path, embed with an allowed-domains list
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: A form component creates or edits a record through the normal write path with Business Rules applied
- [ ] AC-2: An embed to a domain that is not allowed is refused at save time
- [ ] AC-3: All five are keyboard operable and pass axe checks
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest per component; `desktop/core/tests/page_form_component.rs`
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
