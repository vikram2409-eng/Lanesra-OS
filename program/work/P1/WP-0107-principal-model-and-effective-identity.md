---
id: WP-0107
title: Principal model and effective-identity rules
phase: 1
epic: E1.0 Identity foundation
status: in_review
owner: claude
depends_on: [WP-0009]
requirements: [REQ-GOV-002, REQ-GOV-003]
threats: [T-03, T-12]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [agent-identities, access-roles-and-record-scopes]
branch: wp/WP-0107-principal-model
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/682
updated: 2026-10-22
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

- [x] AC-1: An unattended run of an admin-scoped agent fails unless its Agent Identity holds the needed permission; it never inherits an Administrator's rights (test)
- [x] AC-2: An interactive run never exceeds the starting user's access, even when the agent's identity has more (authorisation matrix test)

## Test plan

- `desktop/core/tests/principals.rs`, `desktop/core/tests/effective_identity_matrix.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Split from WP-0100 (XL) on 2026-10-17. Original AC-1, AC-2.
- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0012 feature flags.
- Threats: privilege escalation through unattended runs, confused deputy (threat model, WP-0011).

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-18 claude: Owner decision 2026-10-18: Muse builds this work package following ADR-0016 (Accepted); Claude reviews the PR before the owner merges it. Refine to ready first (Muse batch 5).
- 2026-10-20 claude: Reworked after review. `principals.rs` now builds on Access Control v1 (the `Administrator` user role via `user_service::require_admin`) instead of a separate role ranking. Every run carries an `EffectiveIdentity`: `run_internal` refuses an admin-scoped step unless every principal holds Administrator (`policy.access_denied`), and records the refused run as failed. The webhook route no longer lends the API client owner's Administrator rights (the owner-is-Administrator path); the owner's Access Roles still limit record access. AC-1 and AC-2 proven on real runs in `tests/principals.rs` and by the matrix in `tests/effective_identity_matrix.rs`. Not in this WP: an admin Agent Identity running unattended end to end needs WP-0108's stored identities (the inner `chat_service` gate still checks the record actor, so it fails closed until then); unattributed record writes on unattended runs (RAID I-20) move to WP-0109, when triggers run as Agent Identities.
- 2026-10-22 claude: PR #682 merged. The rules are enforced in the shared core, so desktop and Team Workspace are delivered; help article `agent-identities` written (with WP-0109, PR #700). Stays `in_review` only for the online demo edition: see RAID Q-21.
- 2026-10-22 claude: Online demo edition built (RAID Q-21). The demo's runs follow the same effective-identity rules: a run a person starts acts as that person, limited by an Agent Identity when it names one, and never exceeds their access; a workflow-started run acts as its Agent Identity only, and an agent that uses admin tools is refused unless that identity holds Administrator. Verified in headless Chromium against the demo (interactive Viewer refused even with an Administrator identity; unattended runs refused without one; disabled and deleted identities refused).
