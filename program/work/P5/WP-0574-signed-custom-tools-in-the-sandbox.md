---
id: WP-0574
title: Signed custom tools in the sandbox
phase: 5
epic: E5.8 Developer surface
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0416]
requirements: [REQ-DEV-005]
estimate: L
editions: [desktop, workspace]
delivered: []
editions_reason: runs user code; the demo never executes code
docs: [developer-sdk, ai-security]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0574 — Signed custom tools in the sandbox

## Goal

Let customers add their own tools without lowering the security bar.

## Scope

- Tool package and manifest; publisher signing and trust list; sandbox execution with declared network only; firewall and audit

## Acceptance criteria

- [ ] AC-1: A signed tool from a trusted publisher runs and is audited
- [ ] AC-2: An unsigned, tampered or untrusted tool is refused
- [ ] AC-3: A tool cannot reach a host its manifest does not declare, the database, or any credential

## Test plan

- `desktop/core/tests/custom_tools.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0017; sandbox from WP-0416.
- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
