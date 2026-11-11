---
id: WP-0570
title: Teams as code: declarative export and import
phase: 5
epic: E5.8 Developer surface
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0325]
requirements: [REQ-API-012, REQ-DEV-001]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: a file format and CLI; nothing to show in the demo
docs: [teams-as-code]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0570 — Teams as code: declarative export and import

## Goal

Let engineering teams keep Teams in version control and review changes like code. Pulled forward from v2 (ADR-0020).

## Scope

- Published schema; export and import through CLI and API; validation with line-level errors; diff before apply; secrets never exported

## Acceptance criteria

- [ ] AC-1: Export then import into an empty workspace reproduces the Team and its suite results
- [ ] AC-2: An invalid document is refused with the line and reason
- [ ] AC-3: Import shows a diff and applies nothing until confirmed
- [ ] AC-4: No secret or token appears in an export

## Test plan

- `desktop/core/tests/team_as_code.rs`; `desktop/cli/tests/team_export_import.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
