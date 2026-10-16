# ADR-0015: Online demo architecture for the Playground

- Status: Accepted (program owner, 2026-10-16, PR #638)
- Date: 2026-10-16
- Work package: WP-0013
- Requirements: REQ-PRE-010, REQ-PRE-021
- Constrains: WP-0223
- Untested locally — see CI

## Context

The spec ships the Playground in the online demo with simulated runs "using the same Team format".
Today the demo is a 1.7 MB single-file `app.js` with no build step. The gap audit (WP-0008,
`program/research/gap-audit.md` §11) found the demo already simulates agents, pipelines, an Agent
Teams canvas with approvals, evals and per-agent chat (`app.js:11526-13642`), but stores its data
in camelCase while the desktop uses snake_case — the "same Team format" the spec assumes does not
exist yet.

Constraints from accepted decisions:

- ADR-0004 #5: the demo may change as much as needed to host the Playground, **including adding a
  build step**.
- ADR-0013 #3 / REQ-PRE-021: a feature appears in the online demo **in the same release as desktop
  and Team Workspace, never earlier**.
- ADR-0012 §3: the demo reads a generated copy of the feature registry, published with each release;
  on `main` between releases the demo keeps the last release's state.

## Options considered

### (a) Hand-port the Team format, simulator and Playground UI into `app.js`

The demo team rewrites the logic in the demo's own style each time the desktop changes.

- For: no build step; smallest upfront change.
- Against: **drift is certain, not just likely** — the camelCase/snake_case split the gap audit
  found is exactly what hand-porting produces, and it would now be institutionalised across the
  simulator and the Playground UI as well. Every desktop change needs a manual mirror change, with
  no mechanical check that the mirror is faithful. Rejected.

### (b) A small shared TypeScript package, built to a single ES module the demo loads

One package holds the versioned Team format (JSON Schema as the contract), the simulator (scripted
runs on sample data, write tools always simulated) and the Playground UI pieces the demo needs.
It builds to one ES module; the demo loads it. The desktop and Team Workspace use the same package
(or generate from the same schema) so there is one implementation.

- For: single source of truth — the drift the gap audit found becomes impossible by construction;
  the simulator is tested once and used everywhere; ADR-0004 explicitly permits the build step;
  Netlify already runs build steps.
- Against: the demo gains a build step; the package needs versioning discipline.
- **Chosen.**

### (c) Compile the desktop frontend for the web with a mock backend

- For: maximum fidelity, no porting at all.
- Against: wildly disproportionate — a full React app plus a mock backend to simulate what is, by
  design, scripted sample data; the 1.7 MB demo would balloon; demo releases couple to frontend
  build changes; the mock backend is itself a second implementation to keep faithful. Rejected.

## Decision

**Option (b).** The Team format, the simulator and the demo's Playground UI come from one shared,
versioned TypeScript package built to a single ES module.

### 1. The Team format is a versioned JSON Schema, and it is the contract

- The schema lives at `program/spec/schemas/team-format.schema.json`. It is the single definition
  of a Team both editions share (WP-0013 AC-2).
- Field naming is **snake_case**, matching the desktop (the desktop is the source of truth; the
  demo adapts, not the other way round). The demo's existing camelCase team data is migrated to
  the schema at the WP-0223 port — no dual-format support, no compatibility shim.
- The schema carries a `format_version` (semver). Breaking changes bump the major version and are a
  new work package, never a silent edit.

### 2. One package, one simulator

- A new package (name and location decided in WP-0223; suggested: `packages/team-playground/`)
  contains:
  1. the JSON Schema (or generates its types from it — schema first, types derived, never the
     reverse);
  2. the simulator: scripted runs on sample data. Write tools are always simulated. It follows
     the scripted-provider pattern (WP-0005): deterministic, seeded, no network, no live model;
  3. the Playground UI pieces the demo needs, written against the schema types.
- The desktop/Team Workspace Playground and the demo import the same Team types and the same
  simulator core. Behaviour differences between editions (real tools vs simulated writes) are
  explicit branches on the edition, not forks of the logic.

### 3. The demo gets a build step

- ADR-0004 #5 allows it. The demo build compiles the shared package to a single ES module and
  bundles it with `app.js` (embed or adjacent module — WP-0223 decides; the 1.7 MB budget is
  watched, not wished away).
- Netlify runs the build. `node --check` on the output remains a CI gate.

### 4. The release rule is mechanical (REQ-PRE-021)

- The shared package is built alongside each Playground feature (ADR-0012 §5.1: the demo version
  is built at the same time but stays hidden until release).
- The demo bundle published to Netlify on `main` between releases keeps the last release's
  Playground state. A Playground feature becomes visible in the demo only in the release that
  turns it on for desktop and Team Workspace. No separate demo flag, no early preview.

### 5. Anti-drift conformance tests (the point of this ADR)

These tests are the reason option (b) beats option (a); WP-0223 implements them:

1. **Schema validity test:** the checked-in example Teams (sample data for the simulator) validate
   against `team-format.schema.json`. `Covers: REQ-PRE-010 (WP-0013 AC-2)`.
2. **Desktop conformance test:** the desktop's Team serialisation (whatever WP-0223's port
   produces) validates against the same schema. A desktop change that breaks the schema breaks CI.
3. **Demo bundle test:** the built demo module exports a Team payload that validates against the
   schema.

If the schema and either edition disagree, CI is red. Drift becomes a build failure, not a
slow divergence nobody notices.

## Consequences

- WP-0223 (Playground in the online demo) is now scoped: port against the schema, build the shared
  package, add the three conformance tests, keep the demo's Playground hidden until the release
  that ships it on desktop/Team Workspace.
- The demo's existing camelCase team simulation (`app.js:11526-13642`) is replaced by the shared
  simulator output at WP-0223 — a deliberate, one-time migration, not parallel implementations.
- New Playground work packages in Phase 2/3 extend the schema (minor versions) rather than
  inventing per-edition formats.

## Alternatives not taken

Hand-porting (drift by design) and compiling the desktop frontend for the web (disproportionate
cost and coupling for scripted sample data) — see above.
