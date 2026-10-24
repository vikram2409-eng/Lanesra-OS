---
id: WP-0127
title: Do-not-persist state fields
phase: 1
epic: E1.3 Agent loop
status: in_review
owner: claude
depends_on: [WP-0111]
requirements: [REQ-LOOP-021]
estimate: M
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [execution-graph-agent-teams]
branch: claude/enterprise-build-planning-jxy2kq
pr: 723
updated: 2026-10-24
---

# WP-0127 — Do-not-persist state fields

## Goal

Fields marked memory-only are never written to checkpoints, traces or audit, and are re-fetched on resume.

## Scope

- Field marking in state and bindings
- Redaction at every persistence hook
- Re-fetch on resume

## Acceptance criteria

- [x] AC-1: A planted secret value never appears in any table after a run (scan test)
- [x] AC-2: Resume after restart re-fetches the field

## Test plan

- `desktop/core/tests/do_not_persist.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-24 claude: Refined proposed → ready and claimed. Dependency met (WP-0111 done). Marking is per node (`"do_not_persist": true` on an agent or transform node), since a node's output is the state field; the help lives in `execution-graph-agent-teams` (there is no `task-contracts` article).
- 2026-10-24 claude: Built. New `core/src/do_not_persist.rs` registers a marked node's output strings (raw and JSON-escaped) for the run's thread; the graph run repository, the audit repository and the trace persist hook scrub them to `[not persisted]`, and spans opened inside a marked node keep no bodies. The runtime keeps marked outputs in memory only, holds them out of every checkpoint, and on resume runs marked completed nodes again (step span `lanesra.node.refetched`), failing the run with a message if one cannot be fetched again. Publishing a graph that marks any other node type is refused. Tests: `core/tests/do_not_persist.rs` (4), including a scan of every table for a planted secret (AC-1, checked to fail with the mark removed) and a resume re-fetch (AC-2). Desktop and demo Agent Teams panels gain the checkbox.
- 2026-10-24 claude: In review in #723.
