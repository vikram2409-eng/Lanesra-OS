---
id: WP-0406
title: Content labels and untrusted-input defence
phase: 4
epic: E4.2 Compliance and governance
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0128, WP-0305]
requirements: [REQ-GOV-026, REQ-TOOL-014]
estimate: XL
branch: 
pr: 
updated: 2026-10-16
---

# WP-0406 — Content labels and untrusted-input defence

## Goal

Trust and confidentiality labels on every value, enforced in the firewall.

## Scope

- See RAID R-04 on pulling parts earlier

## Acceptance criteria

- [ ] AC-1: Injection corpus blocked

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Extends the Phase 1 minimal defence in WP-0145; do not build a second mechanism.
- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
