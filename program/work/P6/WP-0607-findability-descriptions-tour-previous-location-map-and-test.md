---
id: WP-0607
title: Findability: descriptions, tour, previous-location map and test
phase: 6
epic: E6.1 Admin console
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0603]
requirements: [REQ-ADM-008, REQ-NFR-007]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [admin-console]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0607 — Findability: descriptions, tour, previous-location map and test

## Goal

Make sure people understand the grouping, especially AI split across three areas.

## Scope

- One-line descriptions on every area and tool; purpose labels on the three AI areas; cross-links from an agent or Team to its runs and policies
- First-visit tour; a "where did it move" list from the previous-names data; a findability test script with ten tasks
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Every area and tool shows its description (registry test)
- [ ] AC-2: Searching a previous name (for example "Page Builder", "Orchestration", "LLM & MCP") opens the new home
- [ ] AC-3: An agent's detail screen links to its runs in AI operations and its policies in Governance
- [ ] AC-4: The findability test is run with at least five first-time admins and its result recorded for the gate
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-7: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest for search and tour; test script and results in `program/gates/G6-experience-and-admin.md`
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
