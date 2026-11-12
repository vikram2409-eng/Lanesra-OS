---
id: WP-0581
title: External key management and secrets managers
phase: 5
epic: E5.9 Enterprise operations
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0180, WP-0181]
requirements: [REQ-OPS-002]
estimate: L
editions: [workspace]
delivered: []
editions_reason: key management services apply to the server
docs: [team-workspace-operations]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0581 — External key management and secrets managers

## Goal

Meet the common requirement that keys stay in the customer's own key service.

## Scope

- Master key provider interface with four providers; secret references for connectors; rotation

## Acceptance criteria

- [ ] AC-1: The server starts with its master key in each provider (contract tests)
- [ ] AC-2: Rotation re-wraps workspace data keys with no downtime
- [ ] AC-3: A provider outage fails closed with a clear health status

## Test plan

- `desktop/core/tests/key_providers.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0014.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
