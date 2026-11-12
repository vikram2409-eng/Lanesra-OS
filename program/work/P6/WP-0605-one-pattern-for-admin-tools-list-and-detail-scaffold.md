---
id: WP-0605
title: One pattern for admin tools: list and detail scaffold
phase: 6
epic: E6.1 Admin console
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0602]
requirements: [REQ-ADM-007]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [admin-console]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0605 — One pattern for admin tools: list and detail scaffold

## Goal

Give every tool the same shape so learning one teaches the rest.

## Scope

- Shared list scaffold (search, filters, lifecycle pill, bulk actions) and detail scaffold (Overview, Configure, Test, History)
- Applied to the Build areas: Data and ontology, Experience Studio, Automation, AI Studio
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Tools in the four Build areas use the scaffolds with no loss of function (existing tests pass unchanged)
- [ ] AC-2: History shows the audit trail of the item; Test opens the tool's existing dry run where one exists
- [ ] AC-3: Scaffolds use only design tokens and pass axe checks
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest for scaffolds; existing tool tests; E2E smoke per area
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
