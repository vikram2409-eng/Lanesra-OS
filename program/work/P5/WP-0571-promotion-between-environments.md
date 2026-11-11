---
id: WP-0571
title: Promotion between environments
phase: 5
epic: E5.8 Developer surface
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0570, WP-0181]
requirements: [REQ-DEV-002]
estimate: M
editions: [workspace]
delivered: []
editions_reason: needs several workspaces on one server
docs: [teams-as-code]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0571 — Promotion between environments

## Goal

A controlled path from development to production.

## Scope

- Environment labels on workspaces; promote command; suite run at the target; promotion record

## Acceptance criteria

- [ ] AC-1: A promotion runs the suite at the target and is refused below the gate
- [ ] AC-2: The record shows who promoted which version from where
- [ ] AC-3: A Builder without rights on the target cannot promote

## Test plan

- `desktop/server/tests/promotion.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
