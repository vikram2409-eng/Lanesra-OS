---
id: WP-0582
title: SIEM export
phase: 5
epic: E5.9 Enterprise operations
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0402]
requirements: [REQ-OPS-003]
estimate: M
editions: [workspace]
delivered: []
editions_reason: needs a reachable server; the desktop app has no public endpoint
docs: [audit-and-compliance]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0582 — SIEM export

## Goal

Put agent activity in the security team's existing tools.

## Scope

- Syslog and HTTPS sinks; documented schema; at-least-once delivery; gap detection by hash chain

## Acceptance criteria

- [ ] AC-1: Events arrive in order with the documented schema
- [ ] AC-2: An outage is caught up without loss or reported as a gap
- [ ] AC-3: Bodies follow the trace-bodies setting

## Test plan

- `desktop/server/tests/siem_export.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
