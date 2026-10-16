---
id: WP-0331
title: Phase 3 security review (gate)
phase: 3
epic: E3.6 Security gate
status: proposed
owner: unassigned
depends_on: [WP-0307, WP-0308, WP-0309, WP-0318, WP-0324]
requirements: [REQ-PRE-009]
estimate: L
editions: []
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0331 — Phase 3 security review (gate)

## Goal

Updated threat model, authz matrix review, prompt-injection test corpus run against every mode, findings fixed.

## Scope

- External or second-AI review

## Acceptance criteria

- [ ] AC-1: Sign-off recorded in gates/G3

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
