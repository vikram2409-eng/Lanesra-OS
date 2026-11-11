---
id: WP-0573
title: TypeScript SDK
phase: 5
epic: E5.8 Developer surface
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0572]
requirements: [REQ-DEV-004]
estimate: M
editions: [desktop, workspace]
delivered: []
editions_reason: as WP-0572
docs: [developer-sdk]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0573 — TypeScript SDK

## Goal

The same for web and Node developers.

## Scope

- Typed client with the same coverage and tests as the Python SDK

## Acceptance criteria

- [ ] AC-1: Coverage matches the Python SDK method for method
- [ ] AC-2: A streamed run yields the documented events in order

## Test plan

- `sdk/typescript/tests/`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
