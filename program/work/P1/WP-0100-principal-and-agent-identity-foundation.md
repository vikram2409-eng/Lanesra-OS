---
id: WP-0100
title: Principal model, Agent Identities and agent credentials
phase: 1
epic: E1.0 Identity foundation
status: done
owner: unassigned
depends_on: [WP-0009]
requirements: [REQ-GOV-002, REQ-GOV-003, REQ-GOV-029]
estimate: XL
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-identities, access-roles-and-record-scopes]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0100 — Principal model, Agent Identities and agent credentials

**SPLIT on 2026-10-17 into WP-0100A, WP-0100B, WP-0100C (XL → 3×M). This file is retained for history; do not build.**

## Goal

Give agents an identity of their own before anything else in Phase 1 records "who acted"
(ADR-0006). Today an agent acts as the human who started it, or an unattended run acts as the
owner of the API client that triggered it, often an Administrator
(`desktop/core/src/services/ai_orchestration_service.rs:15-19`). Afterwards, humans, Agent
Identities and API clients are all principals: access is granted to them the same way (Access Role
and Record Scope), and every action is attributed to them.

## Scope

- One principal model: `human` (users, later signed in by password, OIDC or SAML), `agent_identity`, `api_client`; every principal has an Access Role and a Record Scope
- Agent Identity management: create, edit, disable, delete; an owning human; admin-only; audited (`ai_agent_identities`)
- Agent credentials: Lanesra-issued short-lived tokens bound to one Agent Identity and a scope; rotation, revocation, expiry; secret material encrypted at rest; never sent to a model or written to traces
- Effective identity rules: an interactive run acts as (starting human ∩ agent's ontology scope ∩ agent identity's permissions); an unattended run acts as its Agent Identity only, never as an Administrator by default; the existing "owner is an Administrator" path is replaced and migrated
- Every run, step, tool call and audit entry records both the agent and the effective principal (feeds WP-0110 and WP-0111)
- Existing unattended triggers (schedules, webhooks, workflow actions) are migrated: each is assigned an Agent Identity created with the narrowest role that keeps it working, and the admin is shown what changed
- UI: an Agent Identities screen and an identity picker on agents and triggers, in the eight-group editor (WP-0160)
- Split at refinement (XL): (a) principal model and effective-identity rules, (b) Agent Identities and credentials, (c) trigger migration and UI
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - Replace the `actor = None` defaults for AI and API-client writes, which currently skip Access Control (RAID I-20)
  - Webhook trigger routes must check their `ai_agent_triggers` row and run as that trigger's Agent Identity; today any `agents.trigger` client runs any agent as the client's owner (I-14)
- Owner decisions (ADR-0013, ADR-0014, 2026-10-16):
  - Several workspaces per server and a future SaaS edition (ADR-0014): design with WP-0181. Human accounts live in the server's control database and can belong to several workspaces; principals (human profile, Agent Identity, API client) stay inside one workspace

## Out of scope

- Team-level identity features (Team Owner and Auditor roles, team-wide identities): WP-0318
- Human SSO: WP-0101 to WP-0105, which map directory users onto the same principal model

## Acceptance criteria

- [ ] AC-1: An unattended run of an admin-scoped agent fails unless its Agent Identity holds the needed permission; it never inherits an Administrator's rights (test)
- [ ] AC-2: An interactive run never exceeds the starting user's access, even when the agent's identity has more (authorisation matrix test)
- [ ] AC-3: Every tool call's audit entry names the agent and the effective principal (conformance test across all run entry points)
- [ ] AC-4: An agent credential is never present in a prompt, trace, log or API response (scan test); a revoked or expired credential is refused immediately
- [ ] AC-5: Existing triggers keep working after upgrade, each with an Agent Identity no broader than before (migration fixture test)

## Test plan

- `desktop/core/tests/principals.rs`, `desktop/core/tests/agent_identity_credentials.rs`, `desktop/core/tests/effective_identity_matrix.rs`
- Upgrade fixture for trigger migration (WP-0007 harness)
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Threats: privilege escalation through unattended runs, credential leakage to models, confused deputy (threat model, WP-0011).
- Built before SSO and the run model so neither is reworked later (ADR-0006).

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
