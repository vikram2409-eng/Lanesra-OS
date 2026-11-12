---
id: WP-0513
title: MCP gateway endpoint
phase: 5
epic: E5.2 Delegated credentials and MCP gateway
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0512]
requirements: [REQ-CRED-007]
estimate: M
editions: [workspace]
delivered: []
editions_reason: needs a reachable server; the desktop app has no public endpoint
docs: [mcp-gateway]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0513 — MCP gateway endpoint

## Goal

Give outside MCP clients one governed endpoint for an organisation's approved tools.

## Scope

- One endpoint aggregating approved servers' tools with namespaced names
- Scoped keys, firewall, rate limits and audit as for agent calls

## Acceptance criteria

- [ ] AC-1: An MCP client lists and calls tools from two servers through the gateway
- [ ] AC-2: A key without the scope, a quarantined tool or a rate-limited client is refused
- [ ] AC-3: Every gateway call is in the tool audit with the client

## Test plan

- `desktop/server/tests/mcp_gateway.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR from WP-0147; ADR-0018.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
