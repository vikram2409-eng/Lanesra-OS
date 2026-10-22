---
id: WP-0192
title: "Two-level outbound allow-list: operator bound, workspace narrows"
phase: 1
epic: E1.9 Team Workspace platform
status: proposed
owner: unassigned
depends_on: [WP-0181, WP-0143]
requirements: [REQ-PRE-022, REQ-PRE-019]
threats: [T-12, T-13]
estimate: M
editions: [workspace]
editions_reason: Only Team Workspace hosts several workspaces and refuses internal addresses
delivered: []
docs: [set-up-team-workspace]
branch: 
pr: 
updated: 2026-10-22
---

# WP-0192 — Two-level outbound allow-list: operator bound, workspace narrows

## Goal

Once one server hosts several workspaces (WP-0181), each workspace admin should be able to narrow which internal services their workspace may reach, but never widen past the operator's `LANESRA_OUTBOUND_ALLOW` (RAID Q-23).

## Scope

- A per-workspace outbound allow-list, stored and audited, editable by that workspace's Administrators
- Effective list = operator list intersected with the workspace list; an empty workspace list means the operator list

## Out of scope

- Changing the operator-level setting from the UI

## Acceptance criteria

- [ ] AC-1: A workspace entry outside the operator's list is rejected on save
- [ ] AC-2: A workspace can reach an address only when both lists allow it (connect-time test)
- [ ] AC-3: Every change is audited

## Test plan

- `desktop/core/tests/ssrf_guard.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-22 claude: Written after the owner approved the recommendations in RAID Q-23.
