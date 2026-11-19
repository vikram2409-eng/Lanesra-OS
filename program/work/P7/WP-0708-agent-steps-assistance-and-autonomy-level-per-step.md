---
id: WP-0708
title: Agent steps, assistance and autonomy level per step
phase: 7
epic: E7.4 Agents in lanes
status: in_progress
owner: claude
planned_for: claude
depends_on: [WP-0707]
requirements: [REQ-LANE-016, REQ-LANE-011]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: wp/WP-0708-agent-steps-assistance-autonomy
pr: 
updated: 2026-11-19
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
- 2026-11-19 claude: AC-1 and AC-4 built — the assistance "propose" level. The schema (`Assistance { autonomy, agent }`, `LANE_AUTONOMY_LEVELS = person/propose/approve/auto`) and its publish validation already shipped; this adds the runtime. When a run parks on a form step with `assistance.autonomy == "propose"`, the lane advance hook (`drive_child_lanes`) generates an agent proposal once: `generate_propose_values` builds a prompt from the step's fields + the anchor record context, runs the named agent (resolved by name among active agents) through `chat_service::run_agent_once_with_text` **under the run's actor** (so the agent is bounded by that person's access — AC-3 foundation), and parses the reply as a JSON object of field values (`parse_json_object`, brace-matched so a fenced/prose-wrapped reply still works); stored under run data `__proposal.<step_id>`, never written. `get_run_view` surfaces it as `current.proposal` + `current.autonomy`. On `submit_person_step`, `record_assist_outcome` records which proposed fields the person kept vs overrode under `__assist.<step_id>`; the person's submit is the only write (AC-1). `assist_stats` reports proposed/overridden/override_rate per step (AC-4). Tests: new `core/tests/process_lane_agent_steps.rs` (2) with the scripted provider. Local: agent_steps 2 passed; no regressions (human_step 5, runner 5, child 5, writes 6, repeat_conditions 6, workflow_interplay 3); clippy + fmt clean. Next (mine): AC-2 (autonomy "approve" → write waits in the approvals inbox; "auto" → through the Tool-Call Firewall), a dedicated AC-3 identity test, and the agent-step kind (a step that runs a published agent/Team). AC-6 (demo) + AC-7 (docs) are muse's.
