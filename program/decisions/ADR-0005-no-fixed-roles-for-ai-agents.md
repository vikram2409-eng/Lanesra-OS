# ADR-0005: Work is not pre-assigned to an AI agent

- Status: Accepted (program owner, 2026-10-16)
- Amends: ADR-0004 decisions 1 and 2 (roles and reviewers)

## Context

ADR-0004 named Claude Code and Muse AI as the builders, and a first version of the tracker tagged
each work package with the agent it suited (Claude Code for core, Muse AI for UI). The owner
rejected that: each AI has a weekly usage limit, and when one runs out the owner moves to the other.
Fixed roles would stall half the plan whenever one agent is unavailable.

## Decision

1. No work package is assigned to an AI in advance. There is no `agent` field; `programctl validate`
   rejects one if it reappears.
2. Whichever AI has usage left takes the next unclaimed item from STATUS.md "Ready to start", or
   continues the work package in progress.
3. Switching AI in the middle of a work package is a normal step, done through the Progress log and
   `programctl brief` on the same branch and pull request (AGENT-PROFILES.md, "Switching").
4. The agents differ only in mechanics (Muse cannot run tests or git), handled by CI (WP-0003) and
   the tracker-upkeep workflow (WP-0018); Muse reads the repo and opens PRs itself (owner, 2026-10-16).
5. Review: by an agent other than the author when available, otherwise by the owner.
6. **The two agents work in sequence, never in parallel** (owner, 2026-10-16). Claim branches are
   therefore optional; the work package file's `owner:` and `branch:` are the claim. Because the
   dashboard is the hand-over point between them, it must always be current (REQ-PRE-017, WP-0019).

## Consequences

- Handoff quality decides the pace: every session must end pushed, with a precise Progress log entry.
- WP-0003 and WP-0018 matter more: without them, a week with only Muse AI cannot test or land work.
- Forecasts use points per session across both agents combined (ROADMAP "Forecasting").
