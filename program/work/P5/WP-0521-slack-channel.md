---
id: WP-0521
title: Slack channel
phase: 5
epic: E5.3 Channels
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0520]
requirements: [REQ-CHAN-003]
estimate: L
editions: [workspace]
delivered: []
editions_reason: needs a reachable server; the desktop app has no public endpoint
docs: [channels]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0521 — Slack channel

## Goal

Put Teams and agents where people already work.

## Scope

- Slack adapter: mentions, direct messages, threads; streaming updates; output cards as blocks; user mapping by email

## Acceptance criteria

- [ ] AC-1: A mention starts a conversation and a thread reply continues it
- [ ] AC-2: A card action goes through the normal write path and firewall
- [ ] AC-3: A user who is not mapped gets the guest policy
- [ ] AC-4: Signature and timestamp checks reject forged events

## Test plan

- `desktop/server/tests/channel_slack.rs` with recorded event fixtures
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
