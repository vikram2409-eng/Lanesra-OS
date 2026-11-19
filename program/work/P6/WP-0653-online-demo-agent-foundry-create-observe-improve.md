---
id: WP-0653
title: "Online demo: Agent Foundry (Create / Observe / Improve)"
phase: 6
epic: E6.6 Navigation and theme
status: in_progress
owner: muse
planned_for: muse
depends_on: []
requirements: [REQ-PG-016]
estimate: M
editions: [demo]
delivered: [demo]
editions_reason: Owner decision - the online demo previews the Agent Foundry information architecture first, on sample data; bringing the same Create/Observe/Improve lens to the desktop edition and Team Workspace admin console is WP-0654.
docs: []
docs_reason: Online-demo presentation only. Each capability it surfaces (Agents, Models, Tools, Knowledge, Memory, Guardrails, Evaluations) already has its help; the cross-edition navigation help is written once in WP-0654 when the lens ships to desktop and Team Workspace.
branch: wp/WP-0653-foundry-demo-finish
pr: 790
updated: 2026-11-18
---

# WP-0653 — Online demo: Agent Foundry (Create / Observe / Improve)

## Goal

Give a model-platform buyer a demo that reads, at a glance, as an AI agent
foundry. A third online-demo variant at `/demo/foundry` re-presents the
Agent Foundry data the demo already seeds (agents, skills, pipelines, eval
suites) through the Create / Observe / Improve information architecture that
Azure AI Foundry, Bedrock and Vertex buyers already carry in their heads -
without touching the sales demo (`/demo`) or the warehouse demo
(`/demo/warehouse`). ~12 of the 13 menu items are features Lanesra already
ships, so this is a re-grouping of real screens, not a mock-up.

## Scope

- A `/demo/foundry` route and a Foundry app shell (brand pill, search
  placeholder) with a left menu in three bands: **Create** (Agents, Models,
  Services, Tools, Knowledge, Memory, Guardrails, Orchestration), **Observe** (Monitor,
  Traces), **Improve** (Data, Evaluations, Optimization), plus an Overview.
- Reuse of the real demo screens for Agents, Models (LLM & MCP / Unified
  Gateway), Tools (Skills), Guardrails and Evaluations - the same functions
  the main demo's Admin runs, rendered under the new menu.
- Foundry-native panels for the items with no standalone admin tab:
  Services (MCP server/client + Integration Hub), Knowledge (Document RAG),
  Memory, Monitor, Traces (OTLP-style span trees), Data (eval datasets)
  and Orchestration (Agent Teams and Pipelines).
- An Optimization / Improve run (the "Run details - Improvement" hero):
  candidate iterations over the eval score, avg vs. baseline, task-weighted.
- Its own store key, isolated from the sales demo; light demo-only seed so
  Knowledge and Memory show real data.
- Discoverability: `/demo/foundry` in `sitemap.xml`, and the "Create agents"
  marketing page CTA opens it.

## Out of scope

- Any change to the shipping desktop or Team Workspace admin menu - that is
  WP-0654, reconciled there with the Build/Run/Govern admin menu (REQ-ADM-001).
- A live model call. This static demo synthesizes the optimization run and
  traces deterministically, labelled as such, exactly as the rest of the
  online demo does (REQ-PG-016: write tools always simulated).
- Renaming "Optimization" to "Training". Lanesra has no model training; the
  Improve band is labelled by what it actually does.

## Acceptance criteria

- [x] AC-1: `/demo/foundry` renders a Foundry shell whose left menu groups
  Create / Observe / Improve with the thirteen items above (twelve at first
  ship in #790; Orchestration added as the thirteenth by AC-6, which the
  criterion itself authorizes); the sales and warehouse demos are
  byte-for-byte unaffected.
- [x] AC-2: Agents, Models, Tools, Guardrails and Evaluations open the real
  demo screens (same code the main demo's Admin runs), under the new menu.
- [x] AC-3: Services, Knowledge, Memory, Monitor, Traces, Data and the
  Optimization hero each render real structured data from the store with no
  page error; the hero shows candidate iterations climbing from the baseline.
- [x] AC-4: a Playwright demo spec
  `desktop/e2e/tests/foundry-demo.spec.ts` loads `/demo/foundry`, asserts the
  three bands and every item, opens a reused screen and a native panel, and
  confirms the sales/warehouse demos still load.
- [x] AC-5: website help for the demo (a `/help` slug), and
  `sitemap.xml` (done), `llms.txt`, `llms-full.txt`, the Releases page and the
  Roadmap page updated to name the Foundry demo.
- [x] AC-6: polish and parity - verify light and dark theme
  and phone width (no horizontal scroll, the mobile menu works), deepen the
  Monitor and Traces panels, and add an Agent Teams / Orchestration view to
  the Create band if it earns its place.

## Test plan

- `desktop/e2e/tests/foundry-demo.spec.ts` (AC-4): the demo is static, so the
  test is an E2E against the served site, mirroring `warehouse-lane*.spec.ts`.
- `node --check app.js` and `node --check site-modern-pages.js` stay green.
- No Rust/test-count change - this is online-demo (website) only; it must not
  lower any total in `testing/REGRESSION.md`.

## Notes

- There is a planned admin information architecture already: REQ-ADM-001
  specifies a single Build / Run / Govern menu, defined once as data that
  desktop, Team Workspace and the online demo all read. The Create / Observe /
  Improve lens in this WP is the Agent Foundry's own narrower view, chosen to
  match what agent-platform buyers expect. Reconciling the two - is Create/
  Observe/Improve a sub-view of AI Studio / AI operations within Build/Run/
  Govern, or an alternative top level - is the open question WP-0654 settles,
  ideally in an ADR. Keep this WP demo-only until that is decided.
- Built in `app.js` (route detection `IS_FOUNDRY`, `FOUNDRY_NAV`,
  `renderFoundrySidebar`, `foundryRenderView`, the `foundry*` panels and
  `ensureFoundryData`) and `site-modern-pages.js` (the marketing CTA).

## Progress log

<!-- Append dated entries, newest last. -->
### 2026-11-17 claude
Done: AC-1..AC-3 built and verified headless (pre-installed Chromium): all 13
menu items render with zero page errors; reused screens (Agents, Models,
Tools, Guardrails, Evaluations) load; the optimization hero shows 0.71 to 0.82
over four candidates. Added to `sitemap.xml`; the "Create agents" marketing
page CTA now opens `/demo/foundry`. PR #790.
State (green): `node --check` clean on both files; no Rust/program totals touched.
Next (Muse): AC-4 Playwright demo spec, AC-5 help + llms/releases/roadmap,
AC-6 light/dark and mobile pass plus Monitor/Traces depth.
Gotchas: the shell's bottom nav is the sales demo's - hidden in foundry mode
in `renderFoundrySidebar`. The only console error loading the demo is the
external PostHog analytics script blocked by the sandbox proxy (not ours).
Commands: `node --check app.js`; serve the repo root and open `/demo/foundry`.
### 2026-11-18 muse
Claimed the finish (AC-4 Playwright spec, AC-5 help + docs, AC-6 polish) on
branch wp/WP-0653-foundry-demo-finish from main 1164060 (the #796 merge).
AC-1..AC-3 shipped in #790; `delivered: [demo]` set. WP-0710's done-status
folds into this PR per the no-small-PRs rule. Plan: AC-4 spec mirroring the
warehouse-lane specs (three bands, every item, a reused screen + a native
panel, sales/warehouse demos still load); AC-5 /help slug + llms.txt /
llms-full.txt / Releases / Roadmap; AC-6 light/dark + phone-width pass,
deeper Monitor/Traces panels, Agent Teams / Orchestration view only if it
earns its place. Next: read the Foundry demo code (app.js IS_FOUNDRY,
FOUNDRY_NAV, foundry* panels, ensureFoundryData) and write the spec.
### 2026-11-18 muse (finish)
Done: AC-4, AC-5, AC-6. AC-4: `desktop/e2e/tests/foundry-demo.spec.ts`
(port 4179, static server) - three bands with all thirteen items, a reused
screen (Agents) and native panels (Traces, Monitor), the Optimization hero
0.71 to 0.82, sales + warehouse demos still load, zero page errors on load.
AC-5: new help article `agent-foundry-demo` (5 sections), sitemap entry for
the slug (`/demo/foundry` was already there), llms.txt + llms-full.txt,
Releases v0.99.12, Roadmap item under "AI Agent Foundry & Orchestration".
AC-6: theme toggle flips `data-theme` light/dark and persists; phone width
390px has no horizontal scroll on home or Traces and the mobile menu works
(drawer opens, navigates, closes). Monitor gained Runs by agent + the
approvals inbox state; Traces gained per-span duration bars + per-trace
tool/guardrail/delegation counts. Orchestration EARNED its place: a new
Create-band view listing the real Pipelines (one per topology, steps named
by agent) and the Sales Ops Team (agent nodes resolved to names); the home
card now links there. AC-1 text updated to thirteen items (twelve at first
ship; Orchestration is the thirteenth, authorized by AC-6 itself).
Gotcha: at phone width the lower rail items sit below the fold, so the spec
navigates through the drawer - the first phone test clicked the rail item
directly and hung on Playwright's actionability wait (trace proved the
click never landed); that is the designed mobile flow, not a product bug.
State (green): spec 7/7 locally; `node --check app.js` clean; validate 0/0.
Next: push, open the PR, CI green, merge under blanket approval.
