---
id: WP-0416
title: Sandboxed code tool and notebook sessions
phase: 4
epic: E4.3 Advanced capabilities
status: proposed
owner: unassigned
depends_on: [WP-0331]
requirements: [REQ-TOOL-009, REQ-TOOL-012]
estimate: XL
editions: [desktop, workspace]
delivered: []
branch: 
pr: 
updated: 2026-10-16
---

# WP-0416 — Sandboxed code tool and notebook sessions

## Goal

Isolated Python/SQL, no network by default, Privileged tier.

## Scope

- Sandbox technology ADR first

## Acceptance criteria

- [ ] AC-1: Escape attempts fail

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
