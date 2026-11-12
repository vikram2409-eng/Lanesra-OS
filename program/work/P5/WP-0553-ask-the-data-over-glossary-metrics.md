---
id: WP-0553
title: Ask the data over Glossary metrics
phase: 5
epic: E5.6 Ontology-grounded knowledge
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0412]
requirements: [REQ-KNOW-004]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [knowledge-and-memory, business-glossary]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0553 — Ask the data over Glossary metrics

## Goal

Give agents trustworthy numbers from the definitions the business already agreed.

## Scope

- Tool taking metric, grain, filters and time range; executes the Metric definition; cites the metric version; honours Record Scope

## Out of scope

- Free-form SQL (WP-0416 covers sandboxed SQL)

## Acceptance criteria

- [ ] AC-1: A question resolves to a metric call and returns a number with its metric version
- [ ] AC-2: A filter or grain the metric does not allow is refused
- [ ] AC-3: Two users with different Record Scopes get correspondingly different results

## Test plan

- `desktop/core/tests/ask_the_data.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
