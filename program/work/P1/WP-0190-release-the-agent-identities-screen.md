---
id: WP-0190
title: "Release the Agent Identities screen and trigger picker ahead of WP-0160"
phase: 1
epic: E1.0 Identity foundation
status: in_review
owner: claude
depends_on: [WP-0109]
requirements: [REQ-GOV-003, REQ-GOV-029, REQ-PRE-021]
threats: [T-03, T-12]
estimate: S
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [agent-identities, access-roles-and-record-scopes]
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/705
updated: 2026-10-22
---

# WP-0190 — Release the Agent Identities screen and trigger picker ahead of WP-0160

## Goal

Admins cannot give an Agent Identity the Administrator role today: the only screen for it sits behind the `building` flag `agents.identities_ui` until WP-0160. Since WP-0107, an agent that uses admin tools runs on a schedule or webhook only as an identity holding Administrator, so such agents cannot be made to run at all. The owner decided to release the screen and the trigger's Runs as picker now (RAID Q-24).

## Scope

- `agents.identities_ui` goes to `ga`, on by default in desktop and Team Workspace, and is listed in the demo's `DEMO_RELEASED_FEATURES` (owner exception to ADR-0012 §5 step 2 for this flag)
- Help articles and Releases say where to manage identities; the 'not switched on yet' wording goes
- The flag stays in the registry until WP-0160 removes it with the old code path, so an admin can still switch the screen off per workspace

## Out of scope

- The identity picker in the agent editor (WP-0160)
- Record-level enforcement of an identity's Access Roles (RAID I-20)

## Acceptance criteria

- [x] AC-1: The flag is `ga`, on by default in both editions, listed outside developer mode, and an override can still turn it off (Rust test)
- [x] AC-2: The demo shows the Agent Identities screen and the action's Runs as picker (`check_feature_flags.py` passes with the flag released)
- [x] AC-3: The help articles no longer say the screen is unavailable

## Test plan

- `desktop/core/tests/feature_flags.rs`
- `python3 program/tools/check_feature_flags.py`
- `desktop/src/__tests__/agentIdentities.test.tsx` (screen and picker with the flag on and off)
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-22 claude: Written after the owner approved the recommendations in RAID Q-24.
- 2026-10-22 claude: Started, then paused: the session's automatic permission check refused the edit that adds the flag to the demo's `DEMO_RELEASED_FEATURES`, as a feature-flag change. Shipping the desktop and Team Workspace half alone would break REQ-PRE-021 (same release in every edition), so nothing is committed. The flag change, its Rust test and the help-text edits are ready to apply as soon as the owner allows it.
- 2026-10-22 claude: The owner allowed the flag change. Built: `agents.identities_ui` is `ga`, on by default in desktop and Team Workspace and still switchable off per workspace until WP-0160 removes it; the demo lists it in `DEMO_RELEASED_FEATURES`, so the Agent Identities screen and the action's Runs as picker show there too. Help (`agent-identities`, `access-roles-and-record-scopes`), Releases and llms files updated. The Feature flags admin screen now appears for Administrators, because this flag can be switched. Test: `the_agent_identities_screen_is_released_on_in_every_edition`; the existing `agentIdentities.test.tsx` covers the screen and picker with the flag on and off.
- 2026-10-22 claude: In review in #705.
