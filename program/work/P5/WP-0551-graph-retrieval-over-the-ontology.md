---
id: WP-0551
title: Graph retrieval over the Ontology
phase: 5
epic: E5.6 Ontology-grounded knowledge
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0550]
requirements: [REQ-KNOW-002]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [knowledge-and-memory, ontology]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0551 — Graph retrieval over the Ontology

## Goal

Let agents reason over how records relate, which document search cannot give them.

## Scope

- `related:` context binding expanding along Link Types with hop and row limits; citations to records; access checks per record

## Acceptance criteria

- [ ] AC-1: A binding returns linked records within its limits with citations
- [ ] AC-2: A record the caller cannot read never appears
- [ ] AC-3: The limits cannot be exceeded by a model-supplied argument

## Test plan

- `desktop/core/tests/knowledge_graph_retrieval.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
