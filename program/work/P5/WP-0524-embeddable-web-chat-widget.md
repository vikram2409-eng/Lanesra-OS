---
id: WP-0524
title: Embeddable web chat widget
phase: 5
epic: E5.3 Channels
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0520]
requirements: [REQ-CHAN-006]
estimate: L
editions: [workspace, demo]
delivered: []
editions_reason: served by the server; the demo embeds a simulated widget
docs: [channels]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0524 — Embeddable web chat widget

## Goal

Let an organisation put a governed agent on its own site or portal.

## Scope

- Widget script and iframe; allowed origins; signed visitor tokens; per-visitor rate limits; Theme Studio styling; accessibility

## Acceptance criteria

- [ ] AC-1: The widget loads only on an allowed origin
- [ ] AC-2: A visitor token signed by the host maps to a principal; without one the guest policy applies
- [ ] AC-3: Rate limits hold per visitor
- [ ] AC-4: Keyboard and screen-reader use pass axe checks

## Test plan

- vitest and Playwright for the widget; `desktop/server/tests/channel_widget.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
