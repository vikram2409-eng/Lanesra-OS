---
id: WP-0708
title: Agent steps, assistance and autonomy level per step
phase: 7
epic: E7.4 Agents in lanes
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0707]
requirements: [REQ-LANE-016, REQ-LANE-011]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-14
---

# WP-0708 — Agent steps, assistance and autonomy level per step

## Goal

Put agents inside a person's job, with a dial for how much they do.

## Scope

- Agent step kind running a published agent or Team; assistance on any step (an agent proposes values, a person confirms); four autonomy levels; acceptance and override recorded per step; identity rules

## Acceptance criteria

- [ ] AC-1: An agent proposal is shown as a proposal and written only when confirmed at the "proposes" level
- [ ] AC-2: At "does it with approval" the write waits in the approvals inbox; at "does it" it goes through the tool firewall like any agent call
- [ ] AC-3: In a run a person started, the agent cannot read or write beyond that person's access
- [ ] AC-4: Override rate per step is reported
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-7: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- `desktop/core/tests/process_lane_agent_steps.rs` with the scripted provider
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
