---
id: WP-0532
title: Annotation queues and end-user feedback
phase: 5
epic: E5.4 Evaluation depth
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0212, WP-0408]
requirements: [REQ-EVAL-015, REQ-EVAL-016]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [evaluation]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0532 — Annotation queues and end-user feedback

## Goal

Bring human judgement into evaluation at scale and measure how far the automated judges can be trusted.

## Scope

- Queues fed by sampling rules, flags and feedback; rubric scoring; judge agreement report; one-click to golden case
- Thumbs and comments on answers in the app and channels

## Acceptance criteria

- [ ] AC-1: A reviewer scores a queued run and the judge agreement rate updates
- [ ] AC-2: A corrected run becomes a golden case marked corrected
- [ ] AC-3: Feedback is linked to run and step and can filter sampling
- [ ] AC-4: Only Builders and above see queues; feedback text follows the trace-bodies setting

## Test plan

- `desktop/core/tests/eval_annotation.rs`; vitest and E2E for the queue
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
