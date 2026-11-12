---
id: WP-0592
title: Compliance report
phase: 5
epic: E5.10 AI compliance
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0591]
requirements: [REQ-COMP-004]
estimate: M
editions: [desktop, workspace, demo]
delivered: []
docs: [ai-compliance]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0592 — Compliance report

## Goal

One document to hand an auditor.

## Scope

- Signed bundle (JSON and PDF) per agent, Team or workspace

## Acceptance criteria

- [ ] AC-1: The bundle verifies against its signature
- [ ] AC-2: Gaps are listed, not hidden

## Test plan

- `desktop/core/tests/compliance_report.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
