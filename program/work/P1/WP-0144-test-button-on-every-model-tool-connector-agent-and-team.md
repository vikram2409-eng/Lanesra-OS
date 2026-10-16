---
id: WP-0144
title: Test button on every model, tool, connector, agent and team
phase: 1
epic: E1.5 Tools and firewall
status: proposed
owner: unassigned
depends_on: [WP-0010]
requirements: [REQ-TOOL-013]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [testing-agents-and-connections]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0144 — Test button on every model, tool, connector, agent and team

## Goal

One-click checks that report in plain words what worked and what did not; Guided Setup later uses them.

## Scope

- Check functions per object type
- API and UI button
- Plain-language result strings via prompt catalog/UX copy

## Acceptance criteria

- [ ] AC-1: Each object type has a passing and failing check test

## Test plan

- `desktop/core/tests/test_buttons.rs`; vitest for result rendering
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
