---
id: WP-0552
title: Permission-aware source sync
phase: 5
epic: E5.6 Ontology-grounded knowledge
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0510, WP-0550]
requirements: [REQ-KNOW-003]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: sync needs real connectors; the demo uses sample collections
docs: [knowledge-and-memory]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0552 — Permission-aware source sync

## Goal

Keep outside documents current without leaking them to people who cannot open them at the source.

## Scope

- Scheduled sync from connectors; per-document permission capture; retrieval filter by effective identity; deletion propagation

## Acceptance criteria

- [ ] AC-1: A changed or deleted source document is updated or removed at the next sync
- [ ] AC-2: A chunk is returned only to an identity allowed at the source
- [ ] AC-3: A sync failure is visible to the admin with the cause

## Test plan

- `desktop/core/tests/knowledge_source_sync.rs` with a scripted connector
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
