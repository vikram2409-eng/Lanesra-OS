---
id: WP-0109
title: Trigger migration and Agent Identity UI
phase: 1
epic: E1.0 Identity foundation
status: in_review
owner: claude
depends_on: [WP-0107, WP-0108]
requirements: [REQ-GOV-003]
threats: [T-03]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [agent-identities, access-roles-and-record-scopes]
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/700
updated: 2026-10-22
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


- [x] AC-1: Every tool call's audit entry names the agent and the effective principal (conformance test across all run entry points)
- [x] AC-2: Existing triggers keep working after upgrade, each with an Agent Identity no broader than before (migration fixture test)


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
- 2026-10-22 claude: Built at the owner's request of 2026-10-21 ("finish the outstanding pieces"), which supersedes the 2026-10-18 split (Muse builds, Claude reviews); Muse reviews instead.
  - Migration 0082: every schedule and webhook trigger gets its own Agent Identity, owned by its creator or else the first Administrator, not an Administrator and with no Access Roles (no broader than before: a drained run had no actor, and since WP-0107 a webhook never borrows its client owner's rights). Each assignment is audited (`agent_identity_assigned_on_upgrade`). Identity names use the full trigger id: the upgrade fixture caught a UNIQUE (workspace_id, name) collision with an 8-character prefix.
  - Runs: the drain resolves the queued item's identity (a disabled or deleted one fails the run, recorded); a webhook call needs an active webhook trigger and runs as its identity with the API client owner as record actor (RAID I-14 closed); a Workflow Automation action may name an identity; approving a paused step resumes as the approving Administrator. The run's identity is held for the run's thread (`principals::enter_run`), so delegation and nested calls inherit it.
  - AC-1: every `ai_agent_tool_call` audit entry carries `agent_id`, `tool` and `effective_principal`. `core/tests/trigger_identities.rs` covers manual run, schedule drain, webhook, workflow enqueue, chat, delegation and graph agent node.
  - AC-2: fixture `0081_triggers.db` (`gen_fixtures.py --with-triggers`) proves owners, no rights, audit rows, and that the triggers still run after upgrade.
  - UI: Admin → AI → Agent Identities (identities, credentials issued once, rotate, revoke) and a "runs as" picker on each trigger, behind the `building` flag `agents.identities_ui` (removed by WP-0160). Commands in Tauri and the server dispatch; vitest and a server HTTP test.
  - Not in this package: the identity picker on agents (needs the eight-group editor, WP-0160) and record-level enforcement for identities (RAID I-20). Online demo: building flags are off in every release and the demo does not simulate triggers, so its delivery ships with the flag's graduation in WP-0160. Help: `access-roles-and-record-scopes` gains "Who scheduled and webhook runs act as".
- 2026-10-22 claude: Help article `agent-identities` added (app.js, sitemap, docs index, llms files, Releases v0.97.0 entry). Demo edition: RAID Q-21.
- 2026-10-22 claude: PR #700 merged by the owner (cfc4a38). Delivered on desktop and Team Workspace. Stays `in_review` only for the online demo edition, which ships with the flag in WP-0160 (RAID Q-21).
- 2026-10-22 claude: Online demo edition built (RAID Q-21). The demo's unattended run is the Workflow Automation "Run AI agent" action (it has no schedule or webhook triggers, so there is nothing to migrate): the action carries an Agent Identity, chosen with a picker shown only while `agents.identities_ui` is on and kept when the picker is hidden. Every simulated tool call writes an `ai_agent_tool_call` entry naming the agent, the tool and the effective principal, shown in a new Agent audit trail under AI Agents. Verified in headless Chromium.
