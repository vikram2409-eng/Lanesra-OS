---
id: WP-0533
title: Red-team suite in the publish gate
phase: 5
epic: E5.4 Evaluation depth
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0213, WP-0406]
requirements: [REQ-EVAL-017]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [evaluation, ai-security]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0533 — Red-team suite in the publish gate

## Goal

Find out how an agent fails under attack before an attacker does.

## Scope

- Versioned probe library written fresh; runner with writes simulated; attack success rate per category; publish-gate threshold; findings linked to guardrails

## Acceptance criteria

- [ ] AC-1: A probe run reports success rate per category and never performs a real write
- [ ] AC-2: Publish is blocked above the threshold with the failing probes listed
- [ ] AC-3: Probe library changes are versioned and diffable
- [ ] AC-4: Results are reproducible from recordings

## Test plan

- `desktop/core/tests/eval_red_team.rs` with the scripted provider
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Clean room: probes are written from behaviour descriptions, not copied from any tool.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
