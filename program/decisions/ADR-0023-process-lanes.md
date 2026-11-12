# ADR-0023: Process Lanes — Phase 7, a parallel track, previewed in the online demo first

- Status: Accepted (program owner, 2026-11-12: "add Process Lane to the build program with your recommendations")
- Date: 2026-11-12
- Requirements: REQ-LANE-001..025, REQ-NFR-008
- Source: the owner's "Process Lane — Spec" (draft, 2026-11-12), now at [`../spec/process-lane-spec.md`](../spec/process-lane-spec.md), and the review of it the same day
- Design: [`../design/SOLUTION-DESIGN.md`](../design/SOLUTION-DESIGN.md) section 17
- Detailed ADR to follow: WP-0701 (model on the run engine, definition schema)

## Context

The owner specified a Process Lane: a guided, single-screen, step-by-step execution of one business
process for one record, with stages shown as a chevron bar and steps inside each stage. The idea
is close to case types in case-management products; the spec is explicit that it models the
pattern in Lanesra's own words and copies nothing (clean room).

The review compared the spec with what the platform has and plans:

- The graph runtime (`ai_execution_graphs`, `ai_runs`, `ai_run_nodes`) is durable and already has
  condition, action, agent, approval and loop nodes. It has **no node that waits for a person to
  do work** other than an approval.
- Action Types (`action_type_service::invoke`) are governed writes that obey Business Rules.
- "Run" and "Step" are locked words for the runtime (REQ-CORE-008).
- Three planned items cover nearby ground: human seats (WP-0560), the Guided Intake template
  (WP-0631) and the Path component (WP-0621).

## Decision

1. **Process Lanes enter the plan as Phase 7**, `WP-07xx` in `work/P7/`, closed by gate G7. It is
   a parallel track: a package may start when its own `depends_on` are done.
2. **A lane is not a second engine.** A lane compiles to an execution graph (source kind `lane`)
   and a lane run is a Run. Stages are grouping on top of nodes. The runtime gains one node: a
   human step. Statuses are those of ADR-0007 plus waiting for a person. This replaces the spec's
   own status set.
3. **Naming.** Internally the live instance is a Run. In the product it is labelled by its record,
   lane and stage ("PO-2026-001 · Receive Shipment · stage 2 of 4"). This settles the spec's O-5.
4. **Changes to the owner's spec, all accepted by the owner:**

   | Spec | Decision |
   |---|---|
   | Steps 1.1 and 1.2 collect values that are written nowhere | Run data with a declared schema; every entered value is stored or mapped (REQ-LANE-005) |
   | Record steps write records | They call Action Types; a step's writes are all-or-nothing and repeat-safe (REQ-LANE-006) |
   | A locked step hides nothing | It shows its name and who it waits on; content only as far as the viewer's access allows (REQ-LANE-011) |
   | An agent step runs under the agent's identity | In a run a person started it acts as that person narrowed by the agent's scope (REQ-GOV-002) |
   | One active run per anchor record | One per lane per record; lanes may declare exclusions (REQ-LANE-010) |
   | The hold branch ends in ON_HOLD | Every branch declares a way out; a dead end cannot be published; cancel is defined (REQ-LANE-008, 009) |
   | O-2: the lane owns the transition | Workflows fire by default; a step declares the actions it replaces, visibly (REQ-LANE-013) |
   | O-4: sequential only | Sequential, plus a step that repeats over child records (REQ-LANE-017) |
   | Agent help appears only in the example | Assistance and an autonomy level on any step, with overrides recorded (REQ-LANE-016) |
   | No SLAs in v1 | Still out of scope, but stage and step times are recorded from the start (REQ-LANE-019) |

5. **Added beyond the spec:** blueprint view and export, describe-a-process drafting, anytime
   actions, board by stage and "my steps", dry run and publish checks, child lanes.
6. **Demo-first exception.** REQ-PRE-021 says a feature reaches the online demo only after both
   editions release it. For Process Lanes the owner wants the demo to prove the feature first
   (spec O-6, B8). The exception is granted for WP-0702 only, with two limits: the demo uses the
   lane definition schema fixed in WP-0701, and it shows no behaviour the core will not have.
   WP-0707 aligns the demo with the product screen.
7. **Process Lane is the umbrella for waiting on a person.** A human seat's work item (WP-0560) is
   a lane human step, so WP-0560 now depends on WP-0704. Guided Intake (WP-0631) is the form-only
   case of a lane, and the Path component (WP-0621) is the stage bar.
8. **Still out of scope:** timers and SLA escalation, parallel steps inside a stage, lanes
   anchored to more than one record, lanes across workspaces.

## What can start now

| Package | Needs | State of what it needs |
|---|---|---|
| WP-0701 ADR and schema | Nothing | Can start now |
| WP-0702 Demo: Receive Shipment | WP-0701 | Warehouse demo is on `main` |
| WP-0703 Definitions in the core | WP-0701 | Graph runtime exists |
| WP-0704 Human steps and run data | WP-0703 | Run and step model (WP-0110), trace (WP-0111) done |
| WP-0705 Record and check steps | WP-0703, **WP-0141** | WP-0141 (idempotency keys) is not built; its own dependency is done |
| WP-0706 Approvals and branches | WP-0704 | Approvals on one inbox (WP-0150) done |
| WP-0707 Runner screen | WP-0704, WP-0705, WP-0706 | — |

The only package outside this track that the first usable version waits for is **WP-0141**.

## Alternatives considered

- **A separate lane engine with its own tables and statuses** (the spec as written). Rejected:
  it would repeat durability, approvals, trace and audit, and collide with the locked words.
- **Fold lanes into Teams as Graph mode with human seats.** Rejected: that arrives in Phase 3 and
  Phase 5; a lane needs none of the mode compiler and can ship long before.
- **Editions first, demo after.** Rejected by the owner for this feature.

## Consequences

- The plan grows by 19 work packages and 58 points (253 packages in total).
- Planned split (ADR-0021): Claude Code 5 (demo, definitions, human steps, runner, gate),
  Codex 8 (writes, branches, agent steps, workflow interplay, repeat and conditions, publish
  checks, child lanes, builder), Muse AI 6 (the model ADR and schema, WP-0701, by owner decision 2026-11-12; anytime actions, blueprint view, board and metrics,
  describe a process, more lanes).
- The graph runtime gets a new node type and a new source kind; `validate_for_publish` learns both.
- The spec refers to "WP-0114" as its pattern; in this program WP-0114 is the external agent ADR.
  The pattern meant is "demo first, then a port", which decision 6 records.
- The owner's spec was committed to `program/spec/process-lane-spec.md` on 2026-11-12 (RAID Q-31).
  It is kept as written; this ADR wins where they differ.
