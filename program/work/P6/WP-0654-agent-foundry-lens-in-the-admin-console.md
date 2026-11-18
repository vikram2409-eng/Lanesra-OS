---
id: WP-0654
title: Agent Foundry lens in the admin console (desktop and Team Workspace)
phase: 6
epic: E6.1 Admin console
status: proposed
owner: unassigned
planned_for: codex
depends_on: [WP-0601, WP-0653]
requirements: [REQ-ADM-001, REQ-ADM-009]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [admin-console]
branch:
pr:
updated: 2026-11-17
---

# WP-0654 — Agent Foundry lens in the admin console (desktop and Team Workspace)

## Goal

Bring the Agent Foundry's Create / Observe / Improve presentation, proven in
the online demo (WP-0653), into the real admin console so a model-platform
buyer meets the same mental model in the desktop edition and Team Workspace,
not only on the marketing demo. The agent tools (Agents, Models/Gateway,
Tools/Skills, Knowledge, Memory, Guardrails, Monitor, Traces, Evaluations,
Optimization) are grouped and labelled as an agent lifecycle, driven from the
same menu-registry-as-data every edition reads, so there is one source of
truth and no drift between desktop, Team Workspace and the demo.

## Scope

- Express the Agent Foundry's Create / Observe / Improve grouping in the
  admin menu registry (WP-0601), so desktop, Team Workspace and the demo all
  render it from one definition.
- Apply it to the AI / agent half of the admin menu only: Agents, Models
  (LLM & MCP / Gateway), Tools (Skills + Tool-Call Firewall), Knowledge,
  Memory, Guardrails, Monitor, Traces, Evaluations and Optimization.
- Keep the business-platform half (Data model, Automation, Apps, Access and
  org, Deployment, Integrations) in its own world, unchanged by this lens.
- A "where did it move" map (REQ-ADM-008) from every previous location of an
  agent tool to its new Create / Observe / Improve home.
- Server-enforced role visibility and feature-flag gating carried over
  unchanged (REQ-ADM-003).

## Out of scope

- The online-demo build of the lens (WP-0653, already in review).
- Rearranging the business-platform menu into Foundry terms - it stays in the
  Build/Run/Govern world; this WP only shapes the AI areas.
- Any new agent capability. This is presentation and navigation, not features.

## Open design question (resolve first, record as an ADR)

REQ-ADM-001 already specifies one admin menu in three bands - **Build / Run /
Govern** - defined once as data for every edition. This WP's Create / Observe
/ Improve is the Agent Foundry's own lens. Decide and record in an ADR which
of these it is, because the menu registry must hold exactly one answer:

1. A sub-view: AI Studio (Build) and AI operations (Run) keep their Build/Run/
   Govern homes, and Create/Observe/Improve is how the agent tools are grouped
   and labelled inside them. Lowest risk; preserves the platform story.
2. A parallel top level: "Agent Foundry" becomes a first-class world beside
   "Business Platform", each with its own bands. Strongest for an agent-first
   buyer; most divergence from REQ-ADM-001.

Recommendation on file (see WP-0653 notes and the 2026-11-17 analysis): option
1 - keep two worlds, Business Platform (Build/Run/Govern) and Agent Foundry
(Create/Observe/Improve), with the agent tools grouped by the Foundry lens
inside the AI areas.

## Acceptance criteria

- [ ] AC-1: The admin menu registry carries the Agent Foundry's Create /
  Observe / Improve grouping for the AI tools, defined once and read by
  desktop, Team Workspace and the online demo (REQ-ADM-001); the three
  editions render the same structure from the same data.
- [ ] AC-2: Every agent tool has exactly one home in the new grouping, and
  the business-platform tools are unchanged; a role that may not use a tool
  never sees it, enforced by the server, not only hidden (REQ-ADM-003).
- [ ] AC-3: A "where did it move" map resolves every previous location of an
  agent tool to its new home, and the command palette and deep links still
  open each tool by address (REQ-ADM-002).
- [ ] AC-4: The decision above is recorded in an ADR the menu registry cites;
  the registry holds exactly one grouping, with no competing definition.
- [ ] AC-5: Website help for every slug in `docs:` explains the Agent Foundry
  grouping, how to find each tool and where it moved from; `sitemap.xml`,
  `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page updated
  (REQ-ADM-009).
- [ ] AC-6: Desktop edition, Team Workspace and the online demo behave the
  same from the same menu data; tests run in both desktop and server modes.

## Test plan

- `desktop/core/tests/<admin_menu>.rs`: the menu registry yields the Agent
  Foundry grouping; role visibility denies a scoped user the tools they may
  not use (deny path, not only the happy path). Tag `Covers: REQ-ADM-001`.
- Frontend tests that the admin shell renders the Create/Observe/Improve
  grouping for the AI areas and leaves the business-platform areas intact.
- Full regression suite green; test count not lower than before
  (`testing/REGRESSION.md`).

## Notes

- Depends on the admin menu registry (WP-0601) existing, and on the demo
  (WP-0653) having settled the shape.
- Feature-flag the rearrange so it can ship dark and be turned on per the
  owner's call (REQ-ADM-003).

## Progress log

<!-- Append dated entries, newest last. -->
### 2026-11-17 claude
Created as the backlog item to expand the Agent Foundry demo lens (WP-0653)
into the desktop edition and Team Workspace. Proposed, unassigned. The open
design question (reconcile with Build/Run/Govern, REQ-ADM-001) must be settled
in an ADR before build; recommendation on file is option 1 (two worlds).
