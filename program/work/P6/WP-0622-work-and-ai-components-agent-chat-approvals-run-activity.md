---
id: WP-0622
title: Work and AI components: agent chat, approvals, run activity
phase: 6
epic: E6.3 Components and blocks
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0615, WP-0218, WP-0202]
requirements: [REQ-EXP-009]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [page-components]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0622 — Work and AI components: agent chat, approvals, run activity

## Goal

Put agents and their oversight on the screens where people work.

## Scope

- Agent chat bound to a published agent or Team with the page's record as context; approvals list; run activity feed
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: A chat call runs as the signed-in user through the firewall, approvals and budgets
- [ ] AC-2: The approvals component shows only what the viewer may approve
- [ ] AC-3: Run activity shows only runs the viewer may see
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-6: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/page_ai_components.rs`; E2E with the scripted provider
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
