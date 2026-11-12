---
id: WP-0580
title: SCIM 2.0 provisioning
phase: 5
epic: E5.9 Enterprise operations
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0104]
requirements: [REQ-OPS-001]
estimate: L
editions: [workspace]
delivered: []
editions_reason: directory provisioning applies to the server
docs: [enterprise-sign-in]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0580 — SCIM 2.0 provisioning

## Goal

Keep accounts in step with the directory, including leavers.

## Scope

- SCIM users and groups endpoints behind a provisioning token; deactivation cascade

## Acceptance criteria

- [ ] AC-1: Create, update, deactivate and group changes apply as the directory sends them
- [ ] AC-2: Deactivation ends sessions, revokes grants and reassigns approvals and work items
- [ ] AC-3: A bad token is refused and counted

## Test plan

- `desktop/server/tests/scim.rs` with recorded directory requests
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0016.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
