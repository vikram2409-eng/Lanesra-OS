---
id: WP-0108
title: Agent Identities and credentials
phase: 1
epic: E1.0 Identity foundation
status: in_review
owner: claude
depends_on: [WP-0107]
requirements: [REQ-GOV-002, REQ-GOV-029]
threats: [T-03, T-09]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace]
docs: [agent-identities]
branch: wp/WP-0108-agent-identities
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/683
updated: 2026-10-22
---


# WP-0108 — Agent Identities and credentials


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


- [x] AC-1: An agent credential is never present in a prompt, trace, log or API response (scan test)
- [x] AC-2: A revoked or expired credential is refused immediately (test)


## Test plan


- `desktop/core/tests/agent_identity_credentials.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it


## Notes


- Split from WP-0100 (XL) on 2026-10-17. Original AC-4.
- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0012 feature flags.
- Threats: credential leakage to models (threat model, WP-0011).


## Progress log


<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-18 claude: Owner decision 2026-10-18: Muse builds this work package following ADR-0016 (Accepted); Claude reviews the PR before the owner merges it. Refine to ready first (Muse batch 5).
- 2026-10-20 claude: Rebuilt after review, on the reworked WP-0107. Migration 0081 stores `agent_identities` with Access Control v1 rights (linked Access Roles plus an Administrator flag, not free-text roles) and `agent_identity_credentials` holding only `sha256(token)`; no recoverable token copy. `agent_identity_service`: Administrator-only, audited create/update/enable/disable/delete; issue, rotate and revoke credentials (tokens from the OS random source, 1 to 168 hours); `authenticate` refuses revoked, expired, disabled or other-workspace credentials with `policy.access_denied`; `grant_for_run` feeds `EffectiveIdentity`. SQL lives in `agent_identity_repo`. AC-1 scan test covers every stored text column, the audit log, API models and the prompts sent to the provider; AC-2 tests revoke, rotate, expiry and disable. Not in this WP: running an admin-scoped agent unattended as an admin identity end to end (the inner `chat_service` gate still checks the record actor), API routes, UI and trigger assignment: WP-0109.
- 2026-10-22 claude: PR #683 merged. Backend in the shared core (desktop and Team Workspace); management screens arrive with WP-0109 behind a flag; help article `agent-identities` written (PR #700). Stays `in_review` only for the online demo edition: see RAID Q-21.
