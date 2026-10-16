---
id: WP-0153
title: Standing approvals
phase: 1
epic: E1.6 Approvals
status: proposed
owner: unassigned
agent: claude
depends_on: [WP-0152]
requirements: [REQ-GOV-020]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0153 — Standing approvals

## Goal

Always allow a tool, or a tool with given arguments, scoped to run, team or workspace, expiring (default 30 days); listed, audited and revocable in one place.

## Scope

- `ai_approval_rules`
- Matching at approval time
- Admin list and revoke UI

## Acceptance criteria

- [ ] AC-1: Matching rule auto-approves and is audited
- [ ] AC-2: Expired or revoked rule no longer matches

## Test plan

- `desktop/core/tests/standing_approvals.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
