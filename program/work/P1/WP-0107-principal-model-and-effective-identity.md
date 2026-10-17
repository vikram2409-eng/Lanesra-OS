---
id: WP-0107
title: Principal model and effective-identity rules
phase: 1
epic: E1.0 Identity foundation
status: proposed
owner: unassigned
depends_on: [WP-0009]
requirements: [REQ-GOV-002, REQ-GOV-003]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-identities, access-roles-and-record-scopes]
branch: 
pr: 
updated: 2026-10-17
---

# WP-0107 — Principal model and effective-identity rules

## Goal

Define the principal model (human, agent_identity, api_client) and the effective-identity rules that determine what permissions an agent run operates with. Split from WP-0100 (XL) part (a).

## Scope

- One principal model: `human` (users), `agent_identity`, `api_client`; every principal has an Access Role and a Record Scope
- Effective identity rules: an interactive run acts as (starting human ∩ agent's ontology scope ∩ agent identity's permissions); an unattended run acts as its Agent Identity only, never as an Administrator by default
- Replace the existing "owner is an Administrator" path (`desktop/core/src/services/ai_orchestration_service.rs:15-19`)
- Gap audit: Replace the `actor = None` defaults for AI and API-client writes, which currently skip Access Control (RAID I-20)

## Out of scope

- Agent Identity CRUD UI: WP-0100B
- Agent credentials (tokens): WP-0100B
- Trigger migration: WP-0100C

## Acceptance criteria

- [ ] AC-1: An unattended run of an admin-scoped agent fails unless its Agent Identity holds the needed permission; it never inherits an Administrator's rights (test)
- [ ] AC-2: An interactive run never exceeds the starting user's access, even when the agent's identity has more (authorisation matrix test)

## Test plan

- `desktop/core/tests/principals.rs`, `desktop/core/tests/effective_identity_matrix.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Split from WP-0100 (XL) on 2026-10-17. Original AC-1, AC-2.
- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0012 feature flags.
- Threats: privilege escalation through unattended runs, confused deputy (threat model, WP-0011).

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
