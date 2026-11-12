---
id: WP-0520
title: Channel framework and conversation sessions
phase: 5
epic: E5.3 Channels
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0411, WP-0325]
requirements: [REQ-CHAN-001, REQ-CHAN-002]
estimate: L
editions: [workspace, demo]
delivered: []
editions_reason: needs a reachable server; the demo simulates a channel conversation on sample data
docs: [channels]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0520 — Channel framework and conversation sessions

## Goal

One model for conversations that arrive from outside the app, so each channel is an adapter and not a new run path.

## Scope

- Channel, conversation and message model; conversation to run links; stop rules per conversation
- Principal mapping and guest policy; inbound signature verification and replay protection
- Adapter interface: receive, send, stream, render card, ask approval

## Out of scope

- Any concrete channel

## Acceptance criteria

- [ ] AC-1: A test adapter starts a conversation, continues it across runs and stops at its stop rule
- [ ] AC-2: An unmapped sender gets nothing by default and what the guest policy allows when set
- [ ] AC-3: A replayed or unsigned inbound request is refused
- [ ] AC-4: Conversations are isolated per workspace and appear in run history

## Test plan

- `desktop/core/tests/channel_sessions.rs`; `desktop/server/tests/channel_inbound.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Design section: SOLUTION-DESIGN.md §9.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
