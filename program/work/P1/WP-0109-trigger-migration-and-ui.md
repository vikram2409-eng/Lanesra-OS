---
id: WP-0109
title: Trigger migration and Agent Identity UI
phase: 1
epic: E1.0 Identity foundation
status: proposed
owner: unassigned
depends_on: [WP-0107, WP-0108]
requirements: [REQ-GOV-003]
threats: [T-03]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [access-roles-and-record-scopes]
branch: 
pr: 
updated: 2026-10-18
---


# WP-0109 — Trigger migration and Agent Identity UI


## Goal


Migrate existing unattended triggers to use Agent Identities, and build the Agent Identity UI. Split from WP-0100 (XL) part (c).


## Scope


- Existing unattended triggers (schedules, webhooks, workflow actions) are migrated: each is assigned an Agent Identity created with the narrowest role that keeps it working, and the admin is shown what changed
- Every run, step, tool call and audit entry records both the agent and the effective principal (feeds WP-0110 and WP-0111)
- UI: an Agent Identities screen and an identity picker on agents and triggers, in the eight-group editor (WP-0160)
- Gap audit: Webhook trigger routes must check their `ai_agent_triggers` row and run as that trigger's Agent Identity; today any `agents.trigger` client runs any agent as the client's owner (I-14)


## Out of scope


- Principal model: WP-0100A
- Agent Identity backend: WP-0100B


## Acceptance criteria


- [ ] AC-1: Every tool call's audit entry names the agent and the effective principal (conformance test across all run entry points)
- [ ] AC-2: Existing triggers keep working after upgrade, each with an Agent Identity no broader than before (migration fixture test)


## Test plan


- Upgrade fixture for trigger migration (WP-0007 harness)
- UI tests for Agent Identity screen and picker
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it


## Notes


- Split from WP-0100 (XL) on 2026-10-17. Original AC-3, AC-5.
- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0012 feature flags.
- Threats: privilege escalation through unattended runs (threat model, WP-0011).


## Progress log


<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-18 claude: Owner decision 2026-10-18: Muse builds this work package following ADR-0016 (Accepted); Claude reviews the PR before the owner merges it. Refine to ready first (Muse batch 5).
