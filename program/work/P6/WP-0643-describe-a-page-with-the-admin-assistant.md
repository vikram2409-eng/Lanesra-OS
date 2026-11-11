---
id: WP-0643
title: Describe a page with the Admin Assistant
phase: 6
epic: E6.5 Builder experience
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0615, WP-0624]
requirements: [REQ-EXP-017]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [experience-studio-pages]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0643 — Describe a page with the Admin Assistant

## Goal

A faster start for people who would rather describe than drag.

## Scope

- Assistant tool that drafts a page definition from a description, validated against the component registry
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: The result is always a draft and always valid against the registry
- [ ] AC-2: The assistant cannot publish or assign a page
- [ ] AC-3: The prompt is in the prompt catalog and tested with the scripted provider
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/assistant_draft_page.rs`
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
