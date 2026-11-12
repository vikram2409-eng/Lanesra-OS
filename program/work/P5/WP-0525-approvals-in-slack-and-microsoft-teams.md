---
id: WP-0525
title: Approvals in Slack and Microsoft Teams
phase: 5
epic: E5.3 Channels
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0521, WP-0522, WP-0218, WP-0152]
requirements: [REQ-CHAN-007]
estimate: M
editions: [workspace]
delivered: []
editions_reason: needs a reachable server; the desktop app has no public endpoint
docs: [channels, approvals]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0525 — Approvals in Slack and Microsoft Teams

## Goal

Shorten approval waits by letting approvers answer where they are.

## Scope

- Approval cards with the diff and the allowed responses; decision bound to exact arguments and mapped identity

## Acceptance criteria

- [ ] AC-1: Each of the four responses given in a channel is recorded exactly as from the inbox
- [ ] AC-2: A person who is not an allowed approver, or the run's starter under the four-eyes rule, is refused
- [ ] AC-3: An argument change after the card was sent invalidates the card

## Test plan

- `desktop/server/tests/channel_approvals.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
