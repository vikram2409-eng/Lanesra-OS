---
id: WP-0550
title: Hybrid retrieval with reranking
phase: 5
epic: E5.6 Ontology-grounded knowledge
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0320]
requirements: [REQ-KNOW-001]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [knowledge-and-memory]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0550 — Hybrid retrieval with reranking

## Goal

Raise retrieval quality without adding a bundled vector database.

## Scope

- Fusion of full-text and vector results; reranking through the gateway; score parts in the context receipt; fallbacks

## Acceptance criteria

- [ ] AC-1: A query returns fused, reranked chunks with score parts
- [ ] AC-2: With no embedding or reranking model the search still works on full text
- [ ] AC-3: The reranking call respects data boundaries
- [ ] AC-4: Latency stays within the performance budget

## Test plan

- `desktop/core/tests/knowledge_hybrid_search.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
