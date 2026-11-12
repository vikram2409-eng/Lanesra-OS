---
id: WP-0160
title: Agent editor in eight settings groups
phase: 1
epic: E1.7 Agent setup experience
status: ready
owner: claude
depends_on: [WP-0120, WP-0014]
requirements: [REQ-SETUP-001, REQ-SETUP-002, REQ-SETUP-003, REQ-SETUP-004, REQ-SETUP-005]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [build-your-first-agent]
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/746
updated: 2026-11-11
---

# WP-0160 — Agent editor in eight settings groups

## Goal

Rebuild the agent editor around the eight plain-language groups with essentials, folded More options, effect notes, live risk and cost badges, and 'Using defaults' states.

## Scope

- Group components and layout per mockup 2 (without the guide panel, which is Phase 2)
- Three-field minimum agent
- One home per setting with read-only links elsewhere

## Gate-0 decisions (2026-11-11, owner review)

- **Persona → Job (Q1):** the Job group is the single home for the system prompt — instructions, tone, and the role/goal/background/reasoning_mode from WP-0120. The standalone Persona modal is removed; the editor's Job save calls `update_ai_agent` + `set_ai_agent_persona`, reusing the existing endpoints (no `AiAgentInput` change).
- **Run & triggers (Q2):** in addition to the eight setting groups, the editor gains a ninth "Run & triggers" area for invocation (the existing Triggers panel), since schedules/webhooks are how an agent is invoked, not setup. The eight groups still own all settings (REQ-SETUP-001).
- **Online demo (Q3):** mirrors the eight-group layout and the three-field minimum; the in-editor live test is documented as unavailable (no server), like other demo limitations.
- **Delivery (Q4):** staged in one PR. Stage 1 = consolidate today's per-row modals (Memory, Guardrails, Persona, Routing, Tool limits, Acts as, Versions, scope, skills) into the grouped editor sections, reusing existing APIs, incl. the Persona→Job merge. Stage 2 = live risk/cost badges, one-sentence effect notes, and "Using defaults" states (REQ-SETUP-004/005) + the `RiskBadge`/`LifecyclePill` components.
- Mapping of today's ten row buttons to their one home is in the design proposal; Identity · Job · Knowledge & memory · Tools & actions · Data access · Safety · Model & cost · Test & publish, plus Run & triggers.

## Component tasks (from WP-0014 inventory)

- [ ] Build `RiskBadge` (low/medium/high/critical) in `desktop/src/components/ui`
- [ ] Build `LifecyclePill` (draft/active/paused/archived) in `desktop/src/components/ui`

## Acceptance criteria

- [ ] AC-1: A new agent with name, job and one tool runs
- [ ] AC-2: Effect note appears for a threshold-crossing change
- [ ] AC-3: Axe passes in light and dark

## Test plan

- vitest + RTL for each group; E2E create-and-run agent
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.
- ADR-0022: the agent editor is listed under AI Studio in the Admin console menu registry (WP-0601).

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-11 claude: Gate-0 refine with the owner. Confirmed deps WP-0120 and WP-0014 are done and REQ-SETUP-001..005 exist. Produced a design proposal (eight-group mapping of today's ten per-row buttons, three-field minimum, screen uplift) and settled four decisions (above): Persona folds into Job, a ninth Run & triggers area, the demo mirrors the layout, and delivery is staged in one PR. Status set to ready. Next: build Stage 1 — rebuild the agent editor (`desktop/src/features/settings/AiAgentsAdmin.tsx`) from the Core/Advanced tabs + per-row modals into the grouped sections, reusing existing APIs, with the Persona→Job merge; then Stage 2 for badges/effect notes/'Using defaults'.
- 2026-11-11 claude: Stage 1a shipped on PR #746. Replaced the ten per-agent row buttons (Edit/Memory/Guardrails/Persona/Routing/Tool limits/Acts as/Versions/Triggers/Deactivate) with one "Set up" button that opens a grouped `AgentSetupPanel` (exported) with eight group tabs — Agent, Persona, Knowledge & memory, Safety, Model & cost, Identity, Test & publish, Run & triggers — reusing the existing editors inline; the panel owns its own save mutations (invalidate without closing). Persona now lives under the agent's job (the Persona→Job merge). Create/template drops into the panel. Row keeps read-only Routing/Acts-as badges. Tests: `desktop/src/__tests__/agentSetup.test.tsx` (2); `test-setup.ts` gains a jsdom scrollTo stub. tsc + 68 vitest + build green. Not done: Stage 1b (split the Agent tab into Identity/Job/Tools/Data groups + unify the Job save), Stage 2 (risk/cost badges, effect notes, "Using defaults", RiskBadge/LifecyclePill), the online-demo mirror, and the create-and-run E2E. Status stays ready until the WP is fully delivered across editions.
