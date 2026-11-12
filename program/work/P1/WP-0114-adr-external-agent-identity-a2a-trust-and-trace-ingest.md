---
id: WP-0114
title: ADR: external agent identity, A2A trust and trace ingest
phase: 1
epic: E1.0 Identity foundation
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0108, WP-0111]
requirements: [REQ-FLEET-001]
estimate: M
editions: []
delivered: []
branch: 
pr: 
updated: 2026-11-11
---

# WP-0114 — ADR: external agent identity, A2A trust and trace ingest

## Goal

Fix how an agent built outside Lanesra is identified, trusted and traced, before Phase 3 builds team tools and Phase 5 builds the control plane. Without this, Phase 3 handoff and team tools would assume every seat is a Lanesra agent.

## Scope

- An accepted ADR covering: external agent as a principal kind bound to an Agent Identity (ADR-0006, WP-0108); authentication to and from A2A peers; how an external reply is labelled untrusted (WP-0145); how ingested spans map onto ADR-0008 and are marked externally reported; what a Team seat needs so it can later hold an external agent
- A plain-language behaviour note on the A2A protocol in `program/research/` (clean room: behaviour only)
- Follow-up notes added to WP-0301, WP-0309 and WP-0501 where the ADR constrains them

## Out of scope

- Any product code (WP-0501 to WP-0505)

## Acceptance criteria

- [ ] AC-1: The ADR is Accepted by the owner and names the fields a seat and a run step need for an external agent
- [ ] AC-2: The ADR states the deny paths: unregistered endpoint, unapproved agent, over-budget, paused
- [ ] AC-3: WP-0301, WP-0309 and WP-0501 each cite the ADR in Notes
- [ ] AC-4: `programctl validate` passes

## Test plan

- Documentation only: `python3 program/tools/programctl.py validate`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Follows ADR-0006, ADR-0008, ADR-0016. Design section: SOLUTION-DESIGN.md §8.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
