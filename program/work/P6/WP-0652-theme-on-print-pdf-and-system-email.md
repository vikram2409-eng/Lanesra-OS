---
id: WP-0652
title: Theme on print, PDF and system email
phase: 6
epic: E6.6 Navigation and theme
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0651]
requirements: [REQ-EXP-020]
estimate: M
editions: [desktop, workspace]
delivered: []
editions_reason: print and email are produced by the desktop app and the server; the demo has neither
docs: [theme-studio]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0652 — Theme on print, PDF and system email

## Goal

One brand everywhere a customer sees it.

## Scope

- Published theme tokens applied to quote, order and invoice print and PDF output and to system email

## Acceptance criteria

- [ ] AC-1: Printed documents and system email use the published theme's colours and type
- [ ] AC-2: A draft theme never reaches print or email
- [ ] AC-3: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-4: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/theme_print_email.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
