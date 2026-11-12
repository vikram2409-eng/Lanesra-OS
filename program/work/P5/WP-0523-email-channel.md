---
id: WP-0523
title: Email channel
phase: 5
epic: E5.3 Channels
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0520]
requirements: [REQ-CHAN-005]
estimate: M
editions: [workspace]
delivered: []
editions_reason: needs a reachable server; the desktop app has no public endpoint
docs: [channels]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0523 — Email channel

## Goal

Let a Team work an inbox.

## Scope

- Inbound by IMAP poll or webhook; threading; attachments as untrusted content; outbound through SMTP connections and the approval policy

## Acceptance criteria

- [ ] AC-1: A new mail starts a run and a reply continues the same conversation
- [ ] AC-2: Attachments and bodies are fenced as untrusted
- [ ] AC-3: Outbound mail to an outside domain follows the approval policy

## Test plan

- `desktop/core/tests/channel_email.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
