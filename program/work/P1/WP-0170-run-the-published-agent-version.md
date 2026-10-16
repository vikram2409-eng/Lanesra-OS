---
id: WP-0170
title: Run the published agent version
phase: 1
epic: E1.3 Agent loop
status: proposed
owner: unassigned
depends_on: [WP-0110]
requirements: [REQ-CORE-009]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [agent-versioning-and-approvals]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0170 — Run the published agent version

## Goal

Found by the gap audit (WP-0008, RAID I-18): a run reads only the published version's `output_schema`. The prompt, tools and model routing come from the live, editable `ai_agents` row (`desktop/core/src/services/chat_service.rs:1805-1816`). Editing a published agent therefore changes production behaviour without publishing, and evaluation results cannot be tied to the version that ran. Everything in the Playground, the publish gate and replay assumes runs are pinned to an immutable version.

## Scope

- At run start, resolve the agent's current Published version and run its snapshot: persona and instructions, tools (`action_names`), delegates, skills, model routing, output schema, guardrails
- Draft and Test versions run only from the editor or Playground, never from triggers, pipelines, graphs or delegation
- Record the version id on each run and step (with WP-0110), and on each eval run, so scores are per version
- Migration: give every existing agent without a published version a published snapshot of its current row, so behaviour does not change on upgrade
- Online demo: the same rule on its sample data (runs use the published snapshot; editing shows "unpublished changes")

## Acceptance criteria

- [ ] AC-1: Editing a published agent's prompt or tools does not change what a triggered run uses until the edit is published (test)
- [ ] AC-2: Every run, step and eval run records the agent version id (test)
- [ ] AC-3: The upgrade migration publishes a snapshot for agents that have none, and existing runs behave the same (migration fixture test)
- [ ] AC-4: Desktop, Team Workspace and the online demo behave the same

## Test plan

- New `desktop/core/tests/agent_published_version_runtime.rs`; update `ai_agent_versioning` tests as needed (none deleted)
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Refine at the Gate 0 review: confirm scope against the ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
