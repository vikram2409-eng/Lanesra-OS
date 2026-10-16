---
id: WP-0009
title: Foundational ADRs: run state machine, trace schema, errors, migrations, prompt catalog
phase: 0
epic: E0.3 Architecture decisions
status: in_review
owner: claude
depends_on: [WP-0008]
requirements: [REQ-PRE-007]
estimate: L
editions: []
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/628
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
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - Use the spec corrections in RAID I-23 and `program/research/gap-audit.md` (14 node types, `evaluate_result` judge node, sequential branches, existing output validation and publish gate)

## Acceptance criteria

- [ ] AC-1: Six ADRs merged with status Accepted
- [x] AC-2: Each ADR has at least one example (JSON event, transition table, migration header)
- [x] AC-3: Phase 1 work packages reference the ADRs they must follow

## Test plan

- Not a code change; ADRs reviewed by the owner (and a second AI if available)
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-10-16 claude: The six ADRs are written as Proposed. Each lists the requirements it serves and the work packages it constrains:
  - ADR-0007, run and step states: one vocabulary, a single guard function `transition`, and leases plus a startup sweep, which fixes RAID I-12.
  - ADR-0008, spans and events: span tree, the `lanesra.run-event` v1 envelope with a gapless `seq`, replay from storage, a 16 KiB limit with body references, bodies off by default, one bus on three transports.
  - ADR-0009, error taxonomy: five classes with retry and failover rules, `AppError::Classified`, stable codes; air-gap fails closed (RAID I-17).
  - ADR-0010, migrations: required header, one transaction per migration (today's runner has none), table, JSON, money and index conventions.
  - ADR-0011, prompt catalog: versioned Markdown files with front matter, strict rendering, an evaluation suite per prompt, prompt ids recorded on spans.
  - ADR-0012, feature flags: a code registry with per-edition defaults, per-workspace overrides, `404 feature_disabled`; never on migrations, fixes or policy; removed within two releases of GA.
  - AC-2: each ADR has an example. AC-3: 39 work packages (all of Phase 1, plus WP-0005, WP-0007 and WP-0010) now have a "Must follow" line; BRIEFING has a hard rule.
  - State: in_review on PR #628. AC-1 is met when the owner merges, which accepts the ADRs; the merging agent then sets each to Accepted.
  - Next: WP-0010 (flags), WP-0110 (run schema), and WP-0007 (migration runner transaction and header check).
