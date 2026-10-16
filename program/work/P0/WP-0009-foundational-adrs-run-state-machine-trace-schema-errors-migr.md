---
id: WP-0009
title: Foundational ADRs: run state machine, trace schema, errors, migrations, prompt catalog
phase: 0
epic: E0.3 Architecture decisions
status: proposed
owner: unassigned
agent: either
depends_on: [WP-0008]
requirements: [REQ-PRE-007]
estimate: L
branch: 
pr: 
updated: 2026-10-16
---

# WP-0009 — Foundational ADRs: run state machine, trace schema, errors, migrations, prompt catalog

## Goal

Phase 1 has ~25 work packages that all touch runs, steps, events and prompts. If each AI invents its own statuses, event shapes or error handling, the result will not fit together. Fix these designs first.

## Scope

- ADR: Run and Step state machine - every status (`queued`, `running`, `paused`, `waiting_approval`, `waiting_budget`, `waiting_model`, `completed`, `completed_partial`, `failed`, `cancelled`) and legal transitions
- ADR: Trace and event schema - span tree, span ids, attributes, SSE envelope with schema version, ordering and replay guarantees, payload size limits, redaction hooks
- ADR: Error taxonomy - transient vs policy vs validation vs fatal; which retry, which never retry
- ADR: Migration conventions - next-free-number at rebase, never lower than main's max, one migration per PR, naming, `workspace_id` and audit columns, JSON columns
- ADR: Prompt catalog format - storage, versioning, variables, evaluation on change
- ADR: Feature flag model (consumed by WP-0010)
- Each ADR lists the requirement IDs it serves and the work packages it constrains

## Acceptance criteria

- [ ] AC-1: Six ADRs merged with status Accepted
- [ ] AC-2: Each ADR has at least one example (JSON event, transition table, migration header)
- [ ] AC-3: Phase 1 work packages reference the ADRs they must follow

## Test plan

- Not a code change; ADRs reviewed by the owner (and a second AI if available)
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
