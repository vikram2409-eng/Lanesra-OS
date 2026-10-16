---
id: WP-0160
title: Agent editor in eight settings groups
phase: 1
epic: E1.7 Agent setup experience
status: proposed
owner: unassigned
agent: muse
depends_on: [WP-0120, WP-0014]
requirements: [REQ-SETUP-001, REQ-SETUP-002, REQ-SETUP-003, REQ-SETUP-004, REQ-SETUP-005]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0160 — Agent editor in eight settings groups

## Goal

Rebuild the agent editor around the eight plain-language groups with essentials, folded More options, effect notes, live risk and cost badges, and 'Using defaults' states.

## Scope

- Group components and layout per mockup 2 (without the guide panel, which is Phase 2)
- Three-field minimum agent
- One home per setting with read-only links elsewhere

## Acceptance criteria

- [ ] AC-1: A new agent with name, job and one tool runs
- [ ] AC-2: Effect note appears for a threshold-crossing change
- [ ] AC-3: Axe passes in light and dark

## Test plan

- vitest + RTL for each group; E2E create-and-run agent
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
