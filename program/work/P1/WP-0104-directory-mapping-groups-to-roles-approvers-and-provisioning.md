---
id: WP-0104
title: Directory mapping: groups to roles, approvers and provisioning
phase: 1
epic: E1.1 Enterprise sign-in
status: proposed
owner: unassigned
depends_on: [WP-0102, WP-0103]
requirements: [REQ-GOV-001]
estimate: M
editions: [workspace]
delivered: []
editions_reason: directory mapping happens on the Team Workspace server
docs: [single-sign-on, access-roles-and-record-scopes]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0104 — Directory mapping: groups to roles, approvers and provisioning

## Goal

Map directory users and groups onto Lanesra users, Access Roles, Work Teams and approver eligibility, so users, approvers and (later) Agent Identities all come from the customer's directory.

## Scope

- Just-in-time provisioning on first sign-in
- Group claim → Access Role mapping table, admin UI
- Deprovisioning: disabled in IdP → session revoked on next check

## Acceptance criteria

- [ ] AC-1: Mapped group grants the configured role; removing the group removes it at next sign-in
- [ ] AC-2: Approver routing can target a directory group

## Test plan

- `desktop/server/tests/directory_mapping.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
