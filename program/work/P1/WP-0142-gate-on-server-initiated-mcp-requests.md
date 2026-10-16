---
id: WP-0142
title: Gate on server-initiated MCP requests
phase: 1
epic: E1.5 Tools and firewall
status: proposed
owner: unassigned
depends_on: [WP-0150]
requirements: [REQ-TOOL-011]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [mcp-client-connect-external-server]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0142 — Gate on server-initiated MCP requests

## Goal

Deny external MCP servers' sampling and user-prompt requests by default; when allowed, route through the approvals inbox with per-server request and token caps.

## Scope

- Default-deny handler
- Admin allow per server
- Caps and approval routing

## Acceptance criteria

- [ ] AC-1: Default deny tested
- [ ] AC-2: Allowed request creates an approval and respects caps

## Test plan

- `desktop/core/tests/mcp_request_gate.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
