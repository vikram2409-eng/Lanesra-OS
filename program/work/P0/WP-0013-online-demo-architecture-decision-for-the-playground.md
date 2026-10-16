---
id: WP-0013
title: Online demo architecture decision for the Playground
phase: 0
epic: E0.3 Architecture decisions
status: ready
owner: unassigned
depends_on: []
requirements: [REQ-PRE-010]
estimate: M
editions: [demo]
delivered: []
editions_reason: architecture decision for the online demo itself
docs: [multi-target-architecture]
branch: 
pr: 
updated: 2026-10-16
---

# WP-0013 — Online demo architecture decision for the Playground

## Goal

The spec ships the Playground in the online demo with simulated runs using the same Team format. The demo is a 1.7 MB single-file `app.js` with no build step. The owner has decided (ADR-0004) that the demo may change as much as needed to host the Playground, including adding a build step. Decide how the Team format, simulator and Playground UI are shared without forking the logic.

## Scope

- Options to compare: (a) hand-port into app.js, (b) a small shared TypeScript package built to a single ES module the demo loads, (c) the desktop frontend compiled for the web with a mock backend
- Consider: drift risk, demo load size, no-build-step constraint, testability, Netlify deploy
- Output: ADR with the decision and a JSON Schema for the Team format shared by both
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - The demo already simulates agents, pipelines, an Agent Teams canvas with approvals, evals and per-agent chat (`app.js:11526-13642`), but stores camelCase data while desktop uses snake_case; decide a shared, versioned format (gap audit §11)

## Acceptance criteria

- [ ] AC-1: ADR accepted by the owner
- [ ] AC-2: Team format JSON Schema committed under `program/spec/schemas/`
- [ ] AC-3: WP-0223 updated to match the decision

## Test plan

- Not a code change
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Dependency on the gap audit (WP-0008) removed 2026-10-16 so it can start in parallel with the critical path; the demo is self-contained in `app.js`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
