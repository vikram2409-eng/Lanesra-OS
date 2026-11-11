---
id: WP-0530
title: Simulated users for multi-turn evaluation
phase: 5
epic: E5.4 Evaluation depth
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0212]
requirements: [REQ-EVAL-013]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [evaluation]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0530 — Simulated users for multi-turn evaluation

## Goal

Test conversations, not only single answers.

## Scope

- Persona and goal definition; scripted or model-played user; conversation graders; turn and cost caps; recorded-run replay

## Acceptance criteria

- [ ] AC-1: A simulated conversation runs to its goal or cap and is scored as a whole
- [ ] AC-2: Results are identical when replayed from a recording
- [ ] AC-3: A persona cannot exceed its turn or cost cap
- [ ] AC-4: Scores feed the publish gate like other graders

## Test plan

- `desktop/core/tests/eval_simulated_user.rs` with the scripted provider
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Prompts written fresh in the prompt catalog (ADR-0011).
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
