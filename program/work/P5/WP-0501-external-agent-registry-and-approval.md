---
id: WP-0501
title: External agent registry and approval
phase: 5
epic: E5.1 Agent control plane
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0114, WP-0301]
requirements: [REQ-FLEET-002]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-control-plane]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0501 — External agent registry and approval

## Goal

Give an organisation one place to record every agent it runs, including those built on other frameworks, so they can be approved, owned and later governed.

## Scope

- Registry model and screens: name, owner, framework, endpoint, protocol (A2A, MCP, trace-only), data classes, risk tier, lifecycle state
- Approval step by an AI Admin before an external agent is callable
- Agent card fetch and display for A2A peers
- Solutions export and import of registry entries without secrets

## Out of scope

- Calling external agents (WP-0503); ingest (WP-0504)

## Acceptance criteria

- [ ] AC-1: An AI Admin registers, edits, approves and disables an external agent; every change is audited
- [ ] AC-2: A Builder cannot approve; an unapproved or disabled entry is not offered in any seat or tool picker
- [ ] AC-3: Registry entries export and import in a Solution with endpoints but without credentials
- [ ] AC-4: The registry is isolated per workspace

## Test plan

- `desktop/core/tests/external_agent_registry.rs`: allow and deny paths
- vitest and E2E for the registry screen; axe checks
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR from WP-0114. Feature flag `agent_control_plane`.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
