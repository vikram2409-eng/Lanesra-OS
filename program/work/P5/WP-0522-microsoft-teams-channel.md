---
id: WP-0522
title: Microsoft Teams channel
phase: 5
epic: E5.3 Channels
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0520]
requirements: [REQ-CHAN-004]
estimate: L
editions: [workspace]
delivered: []
editions_reason: needs a reachable server; the desktop app has no public endpoint
docs: [channels]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0522 — Microsoft Teams channel

## Goal

The same for organisations on Microsoft 365.

## Scope

- Adapter for personal, group and channel conversations; adaptive cards; user mapping through Entra ID

## Acceptance criteria

- [ ] AC-1: A message starts a conversation and a reply continues it in each of the three scopes
- [ ] AC-2: A card action goes through the normal write path and firewall
- [ ] AC-3: Token validation rejects forged activities

## Test plan

- `desktop/server/tests/channel_msteams.rs` with recorded activity fixtures
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
