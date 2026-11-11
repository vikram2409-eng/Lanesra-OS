---
id: WP-0540
title: Agent optimizer proposals
phase: 5
epic: E5.5 Agent optimizer
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0212, WP-0130]
requirements: [REQ-OPT-001, REQ-OPT-002]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-optimizer]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0540 — Agent optimizer proposals

## Goal

Turn evaluation results into concrete, reviewable improvements.

## Scope

- Proposal generation from failing cases; diff view; re-score on suite and held-out set; apply as draft version; budget cap per optimisation

## Acceptance criteria

- [ ] AC-1: A proposal shows its diff, evidence cases and before/after scores
- [ ] AC-2: A proposal that lowers the held-out score is marked not recommended
- [ ] AC-3: Applying creates a draft version; nothing is published without the normal gate
- [ ] AC-4: An optimisation stops at its budget cap

## Test plan

- `desktop/core/tests/agent_optimizer.rs` with the scripted provider
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
