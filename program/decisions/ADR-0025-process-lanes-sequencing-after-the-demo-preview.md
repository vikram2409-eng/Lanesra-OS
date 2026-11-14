# ADR-0025: Process Lanes — sequencing after the demo preview

- Status: Accepted (program owner, 2026-11-14: "plan next steps based on recommendations"; "Codex is not in the picture for now")
- Date: 2026-11-14
- Amends: ADR-0023 decisions 1 and 6; ADR-0021 (planned lanes) for Phase 7
- Requirements: no new ones; REQ-LANE-014, REQ-LANE-017 and REQ-LANE-019 are read as below

## Context

On 2026-11-14 the state of Phase 7 was:

- Done or merged: WP-0701 (model and schema, ADR-0024), WP-0703 (lane definitions in the core),
  WP-0702 (Receive Shipment in the online demo), WP-0712 (blueprint view). WP-0715 (drafting a
  lane with the Admin Assistant) is in progress on a Muse branch.
- Not started: everything between the core definitions and a desktop runner: WP-0704, WP-0141,
  WP-0705, WP-0706, WP-0707. The online demo has a runner; desktop and Team Workspace do not.
- Pull request #762 (draft, Muse) adds a second lane, Pick and Pack, to the online demo, with a
  provisional Action Type `run_data.set`. ADR-0023 allowed a demo preview for WP-0702 only.
- Codex is not building. Eight Phase 7 packages and WP-0141 were planned for it.

## Decision

1. **The runner is split.** WP-0707 becomes the first cut: form, record and check steps on the
   straight path, on desktop and Team Workspace, needing only WP-0704 and WP-0705. A new WP-0720
   is the second cut: approval, branch, pause, handoff and cancel screens, after WP-0706. Packages
   that only need a runner (WP-0708, WP-0711, WP-0713, WP-0717) wait for the first cut.
2. **Engine work is one lane of work, done by one agent in sequence.** WP-0704, WP-0141, WP-0705,
   WP-0706, WP-0707 and WP-0720 all change the graph runtime and are planned for Claude Code.
   WP-0704 and WP-0141 are refined and `ready`.
3. **Muse AI works beside it on packages that do not touch the runtime:** finishing #762, WP-0715,
   closing out WP-0702 and WP-0712 in the tracker, a new WP-0719 (Count and Replenish lanes as
   definitions only), and the two Phase 1 design packages WP-0147 and WP-0114.
4. **Demo freeze.** Pull request #762 is accepted as a second and last demo preview. After it, no
   new lane behaviour goes into the online demo until the first cut of the runner (WP-0707) has
   shipped on desktop and Team Workspace. Lanes may still be written as definitions (WP-0719).
   This narrows RAID R-33.
5. **`run_data.set` is not an Action Type in the core.** Action Types are governed writes to
   records. Setting a run-data value from a step is a property of the step. The demo may keep its
   provisional form until WP-0705 fixes the rule (RAID Q-32); #762 must mark it provisional in the
   definition's description so nobody builds on it.
6. **Parallel work without parallel steps.** WP-0710 lets the items of a repeating step be
   assigned to different people and worked at the same time. That covers the known case (a cycle
   count with several counters). Parallel steps inside a stage stay out of scope until a real
   journey cannot be modelled this way.
7. **Order of the larger bets**, none of which blocks the next lane:
   1. Autonomy level per step (WP-0708), once the first cut exists
   2. Drafting with the assistant (WP-0715), then the visual builder (WP-0718) when the bar below is met
   3. An ageing view, "what is late", from the stage times already recorded (added to WP-0713)
   4. Parallel steps, only if decision 6 proves insufficient
   5. Lanes on more than one record, after child lanes (WP-0716)
8. **Bar for starting the visual builder (WP-0718):** four warehouse lanes exist as definitions,
   at least one was drafted through the assistant, the publish checks exist (WP-0714), and the
   authoring log from WP-0719 lists where writing definitions by hand was hard.
9. **Codex's other planned packages** (Phases 1, 5 and 6) keep `planned_for: codex` as a record of
   the plan but are unstaffed. Any agent may take one when it becomes ready (ADR-0021 decision 2).
   In Phase 7 they are re-planned: WP-0708, WP-0709 and WP-0710 to Claude Code; WP-0714, WP-0716
   and WP-0718 to Muse AI.

## Consequences

- A desktop runner needs four packages (WP-0704, WP-0141, WP-0705, WP-0707) instead of five.
- Two packages added: WP-0719 (S) and WP-0720 (M). Phase 7 has 21 work packages.
- The gate (WP-0799) also waits for WP-0720.
- What actually gates the next lane in the product is WP-0710 (repeat over order lines), not any
  of the bets in decision 7.
