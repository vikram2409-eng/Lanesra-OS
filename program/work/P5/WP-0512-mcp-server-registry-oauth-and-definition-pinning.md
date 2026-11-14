---
id: WP-0512
title: MCP server registry, OAuth and definition pinning
phase: 5
epic: E5.2 Delegated credentials and MCP gateway
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0147, WP-0142]
requirements: [REQ-CRED-005, REQ-CRED-006]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [mcp-gateway]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0512 — MCP server registry, OAuth and definition pinning

## Goal

Stop an MCP server from changing what a tool does after an admin approved it, and let MCP servers that need a user sign-in work.

## Scope

- Registry of approved MCP servers; tool definitions pinned by hash
- Change detection and quarantine with a diff for the admin
- Scan of tool descriptions for instruction-like content
- OAuth for MCP servers through the token vault

## Out of scope

- The gateway endpoint (WP-0513)

## Acceptance criteria

- [ ] AC-1: A tool whose definition changed is quarantined and not callable until approved again; the admin sees the diff
- [ ] AC-2: A description containing instruction-like content blocks approval by default and shows the text
- [ ] AC-3: An MCP server needing OAuth works with a user grant and is refused without one
- [ ] AC-4: Unapproved servers are not callable from any agent

## Test plan

- `desktop/core/tests/mcp_registry.rs` with a scripted MCP server that changes its definitions
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0026 (WP-0147): delegated credentials, the token vault and the MCP gateway — the registry and definition pinning this builds. Mitigates tool poisoning in the threat model.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
