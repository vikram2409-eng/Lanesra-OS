---
id: WP-0591
title: Risk classification and control mappings
phase: 5
epic: E5.10 AI compliance
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0590]
requirements: [REQ-COMP-002, REQ-COMP-003]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [ai-compliance]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0591 — Risk classification and control mappings

## Goal

Connect the controls the platform enforces to the frameworks auditors use.

## Scope

- Risk questionnaire and classes; required controls per class at Publish; versioned mapping data for the EU AI Act, ISO/IEC 42001 and NIST AI RMF; evidence links

## Out of scope

- Legal advice; the mappings are aids and say so

## Acceptance criteria

- [ ] AC-1: A high-risk Team cannot publish without its required controls
- [ ] AC-2: Each mapping row links to evidence that exists
- [ ] AC-3: Mapping data is versioned and its changes are audited

## Test plan

- `desktop/core/tests/ai_risk_controls.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Mapping content is written from the public texts of the frameworks.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
