---
id: WP-0201
title: Roles: Viewer, Builder, Approver, AI Admin
phase: 2
epic: E2.1 Playground core
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0104]
requirements: [REQ-GOV-004]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0201 — Roles: Viewer, Builder, Approver, AI Admin

## Goal

Introduce the AI roles the Playground needs.

## Scope

- Role definitions and permission checks on every AI endpoint

## Acceptance criteria

- [ ] AC-1: Permission matrix test covers every endpoint and role

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
