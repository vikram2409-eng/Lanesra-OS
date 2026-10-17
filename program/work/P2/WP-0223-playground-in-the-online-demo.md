---
id: WP-0223
title: Playground in the online demo
phase: 2
epic: E2.5 Templates and demo
status: proposed
owner: unassigned
depends_on: [WP-0013, WP-0203]
requirements: [REQ-PG-016]
estimate: XL
editions: [demo]
delivered: []
editions_reason: the online demo port of the Playground
docs: [agent-playground]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0223 — Playground in the online demo

## Goal

Simulated runs on sample data with write tools always simulated, per the demo ADR.

## Scope

- Shared Team format: `program/spec/schemas/team-format.schema.json` (ADR-0015) — snake_case, versioned, the single contract
- Shared simulator + demo Playground UI from one versioned TypeScript package built to a single ES module (ADR-0015 §2); the demo gains a build step (allowed by ADR-0004 #5)
- The demo's existing camelCase team simulation (`app.js:11526-13642`) is migrated to the schema at this WP — no dual-format support, no compatibility shim
- Anti-drift conformance tests (ADR-0015 §5): (1) sample Teams validate against the schema, (2) desktop Team serialisation validates against the schema, (3) the built demo module's Team payload validates against the schema
- Owner decisions (ADR-0013, ADR-0014, 2026-10-16):
  - The Playground appears in the online demo in the same release as desktop and Team Workspace, never earlier (REQ-PRE-021). The shared package is built alongside each feature but the demo bundle on `main` between releases keeps the last release's Playground state (ADR-0015 §4)

## Acceptance criteria

- [ ] AC-1: Demo E2E covers a template run
- [ ] AC-2: Shared TS package builds to a single ES module the demo loads (`Covers: REQ-PRE-010`)
- [ ] AC-3: Three conformance tests green: sample data, desktop serialisation and demo bundle all validate against `team-format.schema.json` (`Covers: REQ-PRE-010`)
- [ ] AC-4: Demo's camelCase team data migrated to the schema; no dual-format code paths
- [ ] AC-5: Demo Playground hidden until the release that ships it on desktop/Team Workspace (`Covers: REQ-PRE-021`)

## Test plan

- To be defined at refinement; must include integration tests with the scripted provider and, for UI, vitest + E2E + axe
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from the spec; refine into detailed acceptance criteria (and split if larger than L) at the review of the previous phase's gate.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
