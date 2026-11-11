---
id: WP-0585
title: High availability and disaster recovery
phase: 5
epic: E5.9 Enterprise operations
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0421, WP-0401]
requirements: [REQ-OPS-006]
estimate: L
editions: [workspace]
delivered: []
editions_reason: applies to the server on PostgreSQL
docs: [team-workspace-operations]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0585 — High availability and disaster recovery

## Goal

Remove the single server as a point of failure.

## Scope

- Stateless server replicas; run ownership by lease with takeover; recovery runbook with measured targets

## Acceptance criteria

- [ ] AC-1: Killing a replica mid-run loses no run; another replica resumes it from its checkpoint
- [ ] AC-2: A restore drill meets the documented recovery targets
- [ ] AC-3: Two replicas never execute the same step (idempotency and lease test)

## Test plan

- `desktop/server/tests/ha_takeover.rs`; drill notes in the gate file
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
