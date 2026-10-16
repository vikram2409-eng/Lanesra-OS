---
id: WP-0013
title: Online demo architecture decision for the Playground
phase: 0
epic: E0.3 Architecture decisions
status: in_progress
owner: muse
depends_on: []
requirements: [REQ-PRE-010, REQ-PRE-021]
estimate: M
editions: [demo]
delivered: []
editions_reason: architecture decision for the online demo itself
docs: [multi-target-architecture]
branch: wp/WP-0013-demo-architecture
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
- Owner decisions (ADR-0013, ADR-0014, 2026-10-16):
  - The online demo shows a feature in the same release as desktop and Team Workspace, never earlier (REQ-PRE-021, ADR-0012): the demo is built alongside each feature but stays hidden until release

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

### 2026-10-16 muse
Done: Claimed WP-0013 (branch wp/WP-0013-demo-architecture). Wrote ADR-0015 (`program/decisions/ADR-0015-online-demo-architecture-for-the-playground.md`, status Proposed) deciding option (b): a shared versioned TypeScript package built to a single ES module, with the Team format as a versioned JSON Schema as the contract; hand-porting rejected (drift by design, gap audit §11 camelCase/snake_case split), desktop-frontend-for-web rejected (disproportionate). Committed the schema at `program/spec/schemas/team-format.schema.json` (snake_case, format_version semver, 8 modes, seats, task contracts). Updated WP-0223 scope + acceptance criteria (AC-2..AC-5) to match the decision.
State: ADR-0015 is Proposed, not Accepted — AC-1 needs the owner's acceptance. AC-2 done (schema committed, JSON-validated locally). AC-3 done (WP-0223 updated). Nothing is testable beyond CI here: this WP is "not a code change" per its test plan; untested locally — see CI (`programctl validate` must pass on the PR).
Next: open the draft PR "[WP-0013] Online demo architecture decision for the Playground" with the repo PR template; ask the owner to accept ADR-0015 (AC-1). After acceptance, flip ADR-0015 status to Accepted and move WP-0013 toward in_review per FINISH-CHECKLIST.
Gotchas: AGENT-PROFILES.md is empty (nothing to read there). No PR template issue — used `.github/pull_request_template.md`. Do NOT edit program/STATUS.md (CI regenerates it).
Commands: python3 -c "import json; json.load(open('program/spec/schemas/team-format.schema.json'))" (schema validity check)
