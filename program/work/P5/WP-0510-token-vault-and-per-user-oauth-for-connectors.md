---
id: WP-0510
title: Token vault and per-user OAuth for connectors
phase: 5
epic: E5.2 Delegated credentials and MCP gateway
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0147, WP-0102]
requirements: [REQ-CRED-002, REQ-CRED-003]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: real OAuth needs real providers; the demo shows the connect screen with sample grants only
docs: [delegated-credentials]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0510 — Token vault and per-user OAuth for connectors

## Goal

Let each person connect their own account to an outside system, so agents act with that person's permissions there.

## Scope

- OAuth 2.1 authorisation code with PKCE for connectors; token vault under the workspace data key
- Refresh, rotation, revocation; My connections screen and an admin grants list
- Audit of every grant use without the token

## Out of scope

- Using grants in runs (WP-0511); MCP (WP-0512)

## Acceptance criteria

- [ ] AC-1: A user connects, sees the grant with scopes and expiry, and revokes it; the token is never readable again
- [ ] AC-2: An expired token refreshes before a call; a failed refresh marks the grant broken and tells the user
- [ ] AC-3: An admin can list and revoke any grant but cannot read or use it
- [ ] AC-4: Tokens never appear in traces, checkpoints, logs or model context (test scans all four)

## Test plan

- `desktop/core/tests/token_vault.rs` with a scripted OAuth provider
- Upgrade test for the new tables
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR from WP-0147; ADR-0014 for the data key.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
