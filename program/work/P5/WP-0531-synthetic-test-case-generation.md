---
id: WP-0531
title: Synthetic test-case generation
phase: 5
epic: E5.4 Evaluation depth
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0211]
requirements: [REQ-EVAL-014]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [evaluation]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0531 — Synthetic test-case generation

## Goal

Make it quick to reach a useful suite size.

## Scope

- Draft cases from a knowledge collection, sample records or traces; review screen with accept, edit, reject; provenance on each case

## Acceptance criteria

- [ ] AC-1: Generated cases stay drafts until a person accepts them
- [ ] AC-2: Each accepted case records its source
- [ ] AC-3: Restricted data classes are never used as a source

## Test plan

- `desktop/core/tests/eval_synthetic_cases.rs`; vitest for the review screen
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
