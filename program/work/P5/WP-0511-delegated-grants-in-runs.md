---
id: WP-0511
title: Delegated grants in runs
phase: 5
epic: E5.2 Delegated credentials and MCP gateway
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0510]
requirements: [REQ-CRED-004]
estimate: M
editions: [desktop, workspace]
delivered: []
editions_reason: as WP-0510
docs: [delegated-credentials]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0511 — Delegated grants in runs

## Goal

Make the effective identity of a run choose the right credential, and refuse cleanly when there is none.

## Scope

- Grant selection by effective identity at the connector layer
- Refusal messages that tell the user how to connect
- Trace attribute naming the grant owner kind (user or Agent Identity), never the token

## Acceptance criteria

- [ ] AC-1: An interactive run calls a delegated connector with the starting user's grant
- [ ] AC-2: An unattended run uses the Agent Identity's grant, or is refused with a readable reason when it has none
- [ ] AC-3: No path lets a run use another user's grant, including delegation, handoff and nested Teams
- [ ] AC-4: A revoked grant is refused on the next call inside a running step

## Test plan

- `desktop/core/tests/delegated_grant_selection.rs`: decision table from the ADR, one test per row
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR from WP-0147; principal rules from WP-0107.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
