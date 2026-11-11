---
id: WP-0599
title: Phase 5 gate: platform readiness
phase: 5
epic: E5.11 Platform gate
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0505, WP-0513, WP-0525, WP-0533, WP-0585, WP-0592]
requirements: [REQ-NFR-006]
estimate: M
editions: []
delivered: []
branch: 
pr: 
updated: 2026-11-11
---

# WP-0599 — Phase 5 gate: platform readiness

## Goal

Prove the platform extension works end to end before it is announced.

## Scope

- Timed walkthrough: register an external agent, govern it, see its run
- Security review of the new surfaces (A2A, channels, token vault, MCP gateway, custom tools)
- Gate file G5 filled with evidence

## Acceptance criteria

- [ ] AC-1: The walkthrough completes in under 30 minutes for a new admin
- [ ] AC-2: The security review has no open high findings
- [ ] AC-3: `programctl trace --fail-on-gaps` passes for Phase 5 requirements

## Test plan

- Evidence recorded in `program/gates/G5-platform-release.md`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
