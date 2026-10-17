---
id: WP-0100B
title: Agent Identities and credentials
phase: 1
epic: E1.0 Identity foundation
status: proposed
owner: unassigned
depends_on: [WP-0100A]
requirements: [REQ-GOV-002, REQ-GOV-029]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-identities]
branch: 
pr: 
updated: 2026-10-17
---

# WP-0100B — Agent Identities and credentials

## Goal

Implement Agent Identity management (CRUD) and Lanesra-issued agent credentials (short-lived tokens). Split from WP-0100 (XL) part (b).

## Scope

- Agent Identity management: create, edit, disable, delete; an owning human; admin-only; audited (`ai_agent_identities`)
- Agent credentials: Lanesra-issued short-lived tokens bound to one Agent Identity and a scope; rotation, revocation, expiry; secret material encrypted at rest; never sent to a model or written to traces
- Owner decisions (ADR-0013, ADR-0014, 2026-10-16): Human accounts live in the server's control database and can belong to several workspaces; principals (human profile, Agent Identity, API client) stay inside one workspace

## Out of scope

- Principal model: WP-0100A
- Trigger migration: WP-0100C
- Team-level identity features: WP-0318
- Human SSO: WP-0101 to WP-0105

## Acceptance criteria

- [ ] AC-1: An agent credential is never present in a prompt, trace, log or API response (scan test)
- [ ] AC-2: A revoked or expired credential is refused immediately (test)

## Test plan

- `desktop/core/tests/agent_identity_credentials.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Split from WP-0100 (XL) on 2026-10-17. Original AC-4.
- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0012 feature flags.
- Threats: credential leakage to models (threat model, WP-0011).

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
