# Process Lane — Spec

> **Owner's draft, committed 2026-11-12 as the source for Phase 7.** Where this draft and
> [`../decisions/ADR-0023-process-lanes.md`](../decisions/ADR-0023-process-lanes.md) differ, the ADR
> wins (see its decision 4 for the list of changes). The requirements are `REQ-LANE-001..025` in
> [`REQUIREMENTS.md`](REQUIREMENTS.md); the design is section 17 of
> [`../design/SOLUTION-DESIGN.md`](../design/SOLUTION-DESIGN.md). This file is kept as written.

**Status:** Draft spec for discussion (2026-11-12). Not built. Name adopted from the user's pick: **Process Lane**.
**Inspiration:** Pega Blueprint case-type screens (Risk Event Intake / Risk Event Completion — stage chevrons, steps per stage, case fields). Modelled after the pattern, not copied: LanesraOS vocabulary, LanesraOS objects, no Pega code or text lifted.
**Two parts, separate by design:** Part A specs the feature (product-level). Part B specs one real example on the warehouse demo, using its actual entities, IDs and numbers.

---

# Part A — Process Lane functionality

## A1. What it is

A Process Lane is a **guided, single-screen, step-by-step execution of one business process, for one business item**. The user is told step 1, does it, and the lane moves them to step 2 — all on one screen, no hunting through the app. The business item travels along the lane: it has a current stage and a current step, and the lane shows where it is.

The problem it solves: today, completing one business job means visiting several places to record its details (open the order, run a pipeline, check the ledger, check stock, check approvals). A lane collapses that journey into one screen and one source of truth for "where is this job".

## A2. Vocabulary (locked)

| Term | Meaning |
|---|---|
| **Process Lane** | The definition: a named business process made of ordered stages and steps. Example: "Receive Shipment". |
| **Run** | One live execution of a lane for one business item. Example: the run of "Receive Shipment" for PO-2026-001. A run is the instance; the lane is the definition. |
| **Stage** | A named phase of the lane, shown as one chevron in the stage bar. Example: Verify, Record receipt, Stage & put away, Confirm. |
| **Step** | One unit of work inside a stage. Exactly one step is current at a time (v1: steps are sequential). |
| **Anchor record** | The business record the run is about (a row of a Custom Object). The run is anchored to it; the run and its writes are visible on the record's Object 360. |

Names deliberately **not** used: *case type / case* (Pega's words), *flow* (collides with workflows, pipelines and execution graphs), *playbook* (already means the agent execution graph, REQ-CORE-007), *procedure / SOP* (accurate but bureaucratic).

## A3. The model

### A3.1 Lane definition

```text
ProcessLane
  id                e.g. lane-wh-receiving
  name              e.g. "Receive Shipment"
  description       one paragraph: what job this lane does, for whom
  objectKey         the Custom Object key the lane runs on, e.g. purchase_order
  version           integer; a run keeps the version it started with
  stages[]          ordered
    Stage
      id, name, order
      steps[]       ordered
        Step
          id, name, kind, role, required (bool), writes, validation, onComplete
```

A lane runs on **one** object (v1). A lane that needs a second object (e.g. a shipment created during receiving) does it through a step that writes that object — the run stays anchored to the one anchor record.

### A3.2 Step kinds (v1)

| Kind | What the user does | Example |
|---|---|---|
| **form** | Fills fields on the anchor record or a related record, on this screen. | Enter received qty per PO line. |
| **record** | Performs a record action the lane defines: create/update a related record. | Post a RECEIPT StockMove; update Stock on hand. |
| **check** | An automatic validation that must pass to advance. No user input; the lane shows pass/fail and why. | `qty_received + qty_rejected ≤ qty_ordered` per line. |
| **approval** | Routes a decision to a person/role. The run waits until decided. | Manager reviews a receipt discrepancy. |
| **agent** | Runs a pipeline/skill. The agent does its part; the user sees progress and the result, and confirms it where the step requires. | Receiving Agent suggests put-away bins from `product.preferred_bin_id`. |

**Notify is not a step kind.** In the Pega screenshots, "Notify" appears as a step. In LanesraOS, notification is an automatic side-effect of completing a step (a workflow action), not work a user performs. A lane may notify on step completion, but the user never "does" a notify step. Similarly, Pega's "Assign" steps map to the lane's step assignment (role + user), not to a step of their own.

### A3.3 Run (live state)

```text
LaneRun
  id, laneId, laneVersion, anchorRecordId (e.g. purchase_order:po-2026-001)
  status            IN_PROGRESS | WAITING_APPROVAL | ON_HOLD | DONE | CANCELLED
  currentStageId, currentStepId
  startedBy, startedAt, doneAt
  stepStates[]      per step: status (DONE | SKIPPED | BLOCKED), doneBy, doneAt, note
  audit[]           every advance + every write, with who/when (append-only)
```

- **Persisted.** A run survives leaving the screen: reopening the lane returns to the current step. (Demo: localStorage alongside the warehouse seed. Desktop/product: SQLite.)
- **One active run per anchor record.** Starting a second run on the same record is blocked, with a link to the active run.
- **Pause / resume / handoff.** A run can be paused (status stays, current step kept) and its current step reassigned to another user/role. Handoff is recorded in the audit trail.
- **Skip.** Only an optional step can be skipped, and the skip (who, when, note) is recorded. A required step cannot be skipped.
- **Go back.** A completed step can be reopened only while its stage is still current, and reopening reverses nothing: corrections are new writes, never edits to posted records. The ledger stays append-only.

### A3.4 Gating and branching

1. **A required step must be complete and valid before the next step unlocks.** Validation runs before advancing; a failed check blocks with the reason shown, not a silent refusal.
2. **Branching.** A step's outcome can route the run to a different stage or step. The defining example: a discrepancy found during receipt routes to a hold/review branch instead of continuing to put-away. Branches are declared in the lane definition (outcome → target step), not improvised at runtime.
3. **Approval branches.** Approving continues along the approved branch; rejecting routes to the rejection branch (hold, recount, return) and never posts the guarded write.
4. **Done is terminal for the run.** A DONE run cannot be reopened; a correction after DONE is a new run or a manual record change, recorded as such.

### A3.5 Roles and permissions

- Who can do a step comes from the lane definition's `role` per step, resolved against the existing role model (in the warehouse demo: the six seeded roles — Manager, Receiver, Picker, Packer, Admin, Viewer — and `ensureWarehouseRoles()` scoping).
- A step assigned to a role the current user lacks is **visible but locked**, showing who it is waiting on ("Waiting on Warehouse Manager"). Nothing about the step's content is hidden; only the action is gated.
- Agent steps run under the acting agent's identity and the Tool-Call Firewall scoping for that agent (same as pipelines today): an agent step can only touch the object types its agent is scoped to.

### A3.6 Writes are real (core principle)

Completing a step performs its record writes **immediately, in the real records** — a StockMove posted, Stock on hand updated, a PO line's `qty_received` set, a PO status advanced. Never canned/simulated text, never a "recorded note" standing in for a write. This is what separates a lane from a tour overlay and from the current receiving pipeline, whose steps are simulated text with no writes.

Consequence: a lane step's writes must obey the same rules as a direct record edit — StockMove ledger append-only, `available = on_hand − reserved` computed never stored, `qty_received + qty_rejected ≤ qty_ordered`, reservation identity (`Σ qty_reserved` over SO lines = `stock.reserved`) preserved.

### A3.7 Relationship to what already exists

| Existing piece | How a lane relates to it |
|---|---|
| **Workflows** (event automation: when X, do Y) | A lane step completion can fire a workflow; a workflow can start a lane run. Where a lane covers a journey, its steps **own** that journey's transitions: a status change made by a lane step does **not** re-fire the workflow action the step already performed (no duplicate task). A workflow remains the fallback for the same status change happening **outside** a lane. (Proposed rule — see open item O-2.) |
| **Pipelines** (agent runs) | A lane's **agent** step runs a pipeline. The pipeline's steps show as sub-steps inside the lane step, its trace attaches to the run's audit trail, and its effects are real writes through the same step-write mechanism. The standalone pipeline-run screen remains for inspecting agent runs; the lane is how a human does the job. |
| **Approvals inbox** | A lane's **approval** step creates an Approvals inbox entry. The run waits (`WAITING_APPROVAL`) until it is decided; the decision routes the run (A3.4 rule 3). Approving a count line still posts its COUNT_ADJUSTMENT exactly as today. |
| **Object 360** | An anchor record's Object 360 shows its lane runs (definition, current stage/step, status) alongside its related records. The runner screen embeds the anchor record's key fields. |
| **Agents** | A lane does not replace agents; it is the human-facing surface agents plug into. An agent step is how agent work appears inside a human's job. |

### A3.8 Runner screen (layout)

One screen, same pattern as the reference screenshots:

- **Top — stage chevron bar.** Current stage highlighted; completed stages checkmarked; the final stage turns green when the run is DONE. The bar always shows all stages, so the user sees the whole job.
- **Left (or above on narrow screens) — step list for the current stage.** Done steps ✓, current step ▶, upcoming steps locked. Steps of other stages are visible but collapsed to their stage.
- **Centre — the current step's work.** The form fields, the record summary, the approve/reject controls, or the agent step's progress + result. Only what this step needs is shown.
- **Right (or below) — record summary.** The anchor record's key fields plus what this run has written so far (e.g. "Received 40 × HG-1001, 30 × HG-1002; 2 StockMoves posted").
- **Footer — Back / Complete step / Pause.** "Complete step" runs validation first; on failure it stays and says why.

Mobile: the same order stacked vertically (stage bar → current step → record summary).

### A3.9 Builder (how lanes get made)

- **v1: data-defined.** A lane definition is structured data, shipped like the warehouse seed (a JSON literal in `app.js` for the demo; Custom Object rows in the product). No visual builder in v1.
- **Later: visual builder in the Foundry.** Stages and steps composed on a canvas, reusing the workflow/pipeline editors. Explicitly out of v1 scope.

### A3.10 Non-goals (v1)

- Visual lane builder (A3.9).
- Parallel steps inside a stage (v1 steps are sequential; the Pega screenshots list steps under a stage without saying whether they run in parallel — LanesraOS v1 picks sequential and revisits).
- Timers / SLAs / due-date escalation on steps.
- Multi-record lanes (one lane = one anchor record).
- Lanes that span warehouses or organisations.

### A3.11 Acceptance criteria (feature)

- [ ] AC-1: A user completes a lane end-to-end on one screen without opening any other record screen; every step's writes land in the real records (verified via Object 360 / record views).
- [ ] AC-2: A required step cannot be skipped and a failed validation blocks advance with the reason shown.
- [ ] AC-3: A discrepancy branches the run to the hold/review path; approving continues with adjusted quantities, rejecting posts nothing and holds put-away.
- [ ] AC-4: A run survives leaving and reopening (current step restored); a second run on the same anchor record is blocked with a link to the active one.
- [ ] AC-5: An approval step waits in the Approvals inbox; deciding it routes the run and records who decided, when.
- [ ] AC-6: A step assigned to another role shows as locked with the waiting-on name; the user cannot complete it.
- [ ] AC-7: The run's audit trail lists every advance and every write, with actor and timestamp; posted ledger rows are never edited by a lane.
- [ ] AC-8: No duplicate work from the workflow interplay (A3.7): a lane-owned transition fires no duplicate task; the same transition outside a lane still fires its workflow.

### A3.12 Open items

- **O-1 (builder timing):** data-defined v1, visual builder later — confirmed as the plan, or should the builder come first?
- **O-2 (workflow interplay):** the proposed "lane owns the transition, workflow is the fallback" rule (A3.7). Alternative: workflows always fire and the lane suppresses duplicates by checking for an existing task. Needs a call.
- **O-3 (step kinds):** v1 is form + record + check + approval + agent. Is notify-as-a-step ever needed, or is automatic notification always enough?
- **O-4 (parallel steps):** sequential in v1. Is there a v1 journey that genuinely needs two steps done in parallel by different people?
- **O-5 (naming of the run):** the live instance is called a "run". Alternative: the instance is the anchor record itself ("PO-2026-001 is in the Receiving Lane") and "run" is reserved for agent runs. Needs a call — it decides what the UI labels the live item.
- **O-6 (program home):** this feature has no REQ and no WP (no lane/stage concept exists in `program/spec/REQUIREMENTS.md`). It needs a REQ + a Work Package before desktop work starts. Proposed: a P1 WP after the online demo proves it; phase is an owner call.

---

# Part B — Warehouse demo example: the Receiving Lane

## B1. The problem today (concrete)

Accepting a shipment in the warehouse demo means visiting four places, with no single source of truth for "where is this receipt":

1. Dashboard → "Receiving — due today" → PO-2026-001.
2. PurchaseOrder record → read the PO and its lines.
3. Receiving pipeline ("Receiving — verify, stage, put away") → run it. Its three steps are canned simulated text: they write nothing.
4. StockMove list → confirm a receipt was posted; Stock record → confirm on hand changed; Approvals → if anything was short or damaged.

A Process Lane collapses that into one screen: the **Receive Shipment** lane, one run per PO.

## B2. The lane

```text
Process Lane: lane-wh-receiving — "Receive Shipment"
Runs on: purchase_order
Version: 1
Entry points: dashboard "Receiving — due today" card (the PO link starts the lane);
  the PurchaseOrder record View ("Start receiving lane" action);
  the Receiving Agent page.
The run is visible on the PO's Object 360 (definition, current stage/step, status).
```

**Roles:** steps for a Receiver (`user-02` Marcus Webb, `user-03` Aisha Khan); the discrepancy review for the Warehouse Manager (`user-01` Sarah Chen). Same six seeded roles and scoping as the rest of the demo.

## B3. Stages and steps — the full run for PO-2026-001

Anchor record: `po-2026-001` — **PO-2026-001**, supplier Kiln & Clay Co. (`sup-01`, Priya Nair), status OPEN, ordered 2026-11-06, expected **2026-11-13** (due today), total **$2,105.00**.
Lines: `pol-001-1` — HG-1001 Stoneware Dinner Set, 16-pc, 40 × $41.00; `pol-001-2` — HG-1002 Ceramic Pasta Bowls, Set of 4, 30 × $15.50.
Starting stock: `stock-hg-1001` 48 on hand (6 reserved, 42 available, lot-007 LOT-DIN-2026-027, bin loc-a-01-03 A-01-03); `stock-hg-1002` 22 on hand (4 reserved, 18 available, no lot, bin loc-a-01-03).

### Stage 1 — Verify (Receiver)

| Step | Kind | What happens | Fields / checks | Writes |
|---|---|---|---|---|
| 1.1 Confirm shipment arrived | form | Receiver confirms the truck/delivery: carrier, delivery-note number, received date 2026-11-13. | Check: PO status is OPEN; `expected_date` is 2026-11-13. | None (confirmation only). |
| 1.2 Match lines to the delivery note | form + check | Per-line delivery-note quantities: HG-1001 ___, HG-1002 ___. | Check: a quantity is entered for every line. | None yet. |

### Stage 2 — Record receipt (Receiver, then Manager on branch)

| Step | Kind | What happens | Fields / checks | Writes |
|---|---|---|---|---|
| 2.1 Enter received quantities | form | Per line: `qty_received`, `qty_rejected` + rejection reason. Happy path: 40 and 30, 0 rejected. | Check (blocks advance): `qty_received + qty_rejected ≤ qty_ordered` per line. | `pol-001-1.qty_received`, `pol-001-2.qty_received`, `qty_rejected` on the PO lines. |
| 2.2 Discrepancy check | check | Automatic. If every line is fully received → Stage 3. If any line is short or rejected → **branch 2B** (run does not continue to put-away). | Rule: any `qty_received < qty_ordered` or `qty_rejected > 0`. | None. |

**Branch 2B — Hold & review (Warehouse Manager, Sarah Chen)**

| Step | Kind | What happens | Writes |
|---|---|---|---|
| 2B.1 Review discrepancy | approval | Manager sees per-line ordered vs received vs rejected, with the receiver's note. Approve → continue to Stage 3 with the adjusted quantities; PO becomes PARTIALLY_RECEIVED. Reject / hold → run goes ON_HOLD, put-away is blocked, no receipt is posted for the disputed lines. | On approve: PO status → PARTIALLY_RECEIVED. On hold: nothing posted. |

This branch is the story of **PO-2026-005** in the seed: 30 × HG-1008 ordered, 18 received, 2 rejected damaged; `mv-015` RECEIPT CONFIRMED with put-away still pending (18 pans sitting in RCV-02). The lane gives that situation a guided path instead of a task and a hold note. (Proposed demo seeding, B5: PO-2026-005 carries a run waiting at step 2B.1.)

### Stage 3 — Stage & put away (Receiver)

| Step | Kind | What happens | Fields / checks | Writes |
|---|---|---|---|---|
| 3.1 Stage the goods | form + record | Assign a receiving bin: RCV-01 (`loc-rec-01`) or RCV-02 (`loc-rec-02`). Confirm the staged quantities per line. | Check: staged qty = received qty per line. | StockMove **RECEIPT** per line: `MV-2026-016` RECEIPT HG-1001 40, `null → loc-rec-01`, `reference_id po-2026-001`, `performed_by` the receiver, status DONE; `MV-2026-017` RECEIPT HG-1002 30, same. (Move numbers are the next in sequence after `mv-015`.) |
| 3.2 Confirm put-away | form + record | Bin per line, suggested from `product.preferred_bin_id`: HG-1001 → `loc-a-01-03` (A-01-03), HG-1002 → `loc-a-01-03`. Receiver confirms or overrides. | Check: put-away qty = staged qty; the bin exists and belongs to wh1. | StockMove **PUTAWAY** per line: `MV-2026-018` PUTAWAY HG-1001 40, `loc-rec-01 → loc-a-01-03`; `MV-2026-019` PUTAWAY HG-1002 30, same path. Stock updates: `stock-hg-1001.on_hand` 48 → **88**; `stock-hg-1002.on_hand` 22 → **52**. Lot: HG-1001 is lot-tracked → moves carry `lot-007`; HG-1002 is not lot-tracked → `lot_id null`. `available` is recomputed, never written (88 − 6 = 82; 52 − 4 = 48). |

These are the exact writes the `wf-wh-01` put-away task describes ("Put away received stock to its suggested bin (product preferred bin)") — the lane performs them instead of creating a task. See B6 for the workflow interplay.

### Stage 4 — Confirm & close (Receiver; Manager notified automatically)

| Step | Kind | What happens | Writes |
|---|---|---|---|
| 4.1 Review summary | form (read-only) | The run's own record: what was received, what was staged and put away, new on-hand 88 / 52, PO total $2,105.00. | None. |
| 4.2 Close the PO | record | Receiver confirms. | `purchase_order.status` OPEN → **RECEIVED**. Automatic notification to Sarah Chen (a workflow action, not a user step). Run status → DONE. |

**End state for the happy path:** PO-2026-001 RECEIVED; `pol-001-1.qty_received` 40, `pol-001-2.qty_received` 30; Stock 88 / 52; four new StockMove rows (`MV-2026-016`…`MV-2026-019`: 2 RECEIPT + 2 PUTAWAY); run DONE with a full audit trail (who did each step, when). Dashboard "Receiving — due today" drops to 0; receiving backlog drops from 6 open POs to 5.

## B4. The discrepancy path, worked with numbers

Suppose 38 of the 40 dinner sets arrive and 2 arrive damaged. Step 2.1 records `pol-001-1`: `qty_received` 38, `qty_rejected` 2 (reason: damaged); `pol-001-2`: 30 / 0. Step 2.2 branches to 2B. Sarah Chen approves at 2B.1. Stage 3 then stages and puts away **38** dinner sets only: RECEIPT 38 → RCV-01, PUTAWAY 38 → A-01-03; `stock-hg-1001.on_hand` 48 → **86**. The PO closes as **PARTIALLY_RECEIVED** (2 short), and `wf-wh-07`'s task ("Receipt discrepancy — review short/rejected quantities; hold put-away") is the fallback record of the decision — the lane itself already holds the decision and the audit trail. The 2 rejected units get no StockMove, exactly as the seed treats PO-2026-005's 2 damaged pans.

## B5. Demo seeding (proposed)

- **Lane definition** `lane-wh-receiving` is seeded as data, like the Day 2 agents and workflows (a JSON literal in `app.js`, healed for pre-existing stored workspaces the same way `ensureWarehouseDay2()` heals Day 2 data).
- **PO-2026-001 has no run.** The visitor starts it — that is the demo.
- **PO-2026-005 carries a run waiting at step 2B.1** (discrepancy review, waiting on Sarah Chen). It shows the branch, the handoff and the approval wait without the visitor having to manufacture a shortfall. Its seeded state mirrors the seed exactly: 18 received / 2 rejected, `mv-015` CONFIRMED, put-away pending.
- **Reset** ("Reset warehouse demo") clears lane runs and restores the seed, but keeps lane definitions. A reset run is a fresh lane, not a deleted definition.

## B6. Relationship to the existing Day 2 pieces

| Existing piece | What changes (and what does not) |
|---|---|
| Receiving pipeline ("Receiving — verify, stage, put away", 3 steps) | Its verify / stage / put-away become the lane's agent assistance: inside steps 1.1, 3.1 and 3.2 the Receiving Agent proposes (bin suggestions from `preferred_bin_id`, line matching), the user confirms. The standalone pipeline-run screen stays for inspecting agent runs. The pipeline stops being the way a human receives — the lane is. Its canned step text is retired for receiving: lane steps write real records. |
| `wf-wh-01` (PO received → put-away task) | When the lane closes the PO as RECEIVED at step 4.2, the workflow must **not** create a duplicate put-away task — put-away already happened in Stage 3. This is the A3.7 "lane owns the transition" rule applied. The workflow stays as the fallback for a PO that becomes RECEIVED outside a lane. |
| `wf-wh-07` (receipt discrepancy → Manager task, hold put-away) | The lane's branch 2B does this job with the decision recorded in the run. The workflow stays as the fallback for a PO that becomes PARTIALLY_RECEIVED outside a lane. |
| Approvals inbox (14 pre-seeded pending) | Step 2B.1 creates its approval entry there. The existing discrepancy entry for PO-2026-005 becomes the seeded run's waiting approval (B5), not a separate standalone item. |
| Receiving Agent (`wh-agt-receiving`), 9 skills, 6 pipelines | Unchanged in scope. The agent's skills are what the lane's agent assistance calls; the pipeline's steps become sub-steps shown inside lane steps. |
| Dashboard "Receiving — due today" | The PO link starts the lane run (B2 entry point). The card's counts are unchanged. |

## B7. Acceptance criteria (warehouse example)

- [ ] AC-1: A visitor completes PO-2026-001 end-to-end on the lane screen alone; the end state (B3) is verifiable in the PO record, Stock records (88 / 52) and StockMove list (MV-2026-016…019).
- [ ] AC-2: Entering `qty_received` above `qty_ordered` on a line blocks step 2.1 with the reason shown; nothing is written.
- [ ] AC-3: The PO-2026-005 seeded run waits at step 2B.1 for Sarah Chen; approving it puts away 18 pans (Stock HG-1008 0 → 18, bin B-01-02) and leaves the PO PARTIALLY_RECEIVED; rejecting/holding posts nothing.
- [ ] AC-4: Leaving mid-lane and reopening returns to the current step; Reset restores the seed (runs cleared, definitions kept).
- [ ] AC-5: Closing PO-2026-001 creates no duplicate put-away task from `wf-wh-01`; receiving the PO outside a lane still fires `wf-wh-01`.
- [ ] AC-6: The run's audit trail is visible on the PO's Object 360 (each step, actor, timestamp, writes).

## B8. Out of scope for this example (future lanes)

Other warehouse journeys are named here so they are not accidentally built into this one: **Pick & Pack Lane** (`sales_order`: allocate → pick task + wave → pack → ship + tracking), **Count Lane** (`cycle_count`: count lines → variance review → Manager approval → COUNT_ADJUSTMENT), **Replenish Lane** (`reorder_rule`: triggered rule → draft PO → Manager approval). Each is a separate lane definition, separate spec section, same Part A machinery. The visual lane builder and the desktop port follow the same pattern as WP-0114 (online demo proves it first; desktop is a port, not a second design).

## B9. Reference — Pega mapping (lineage, not a copy)

| Pega screenshot element | Process Lane equivalent |
|---|---|
| Case type (e.g. Risk Event Intake) | Process Lane definition (e.g. Receive Shipment) |
| Stage chevron (Capture Information → Assess Impact → Review Event → Mitigation Plan → Resolution) | Stage bar (Verify → Record receipt → Stage & put away → Confirm); final stage green when DONE |
| Steps listed under a stage (Extract Customer Data, Assign First Line Risk Team, …) | Steps inside a stage; one current at a time in v1 |
| Assign / Validate / Approve steps | Step assignment by role; check steps; approval steps |
| Notify steps | Automatic notification on step completion (workflow action), not a user step |
| Field table (Case ID, Case Label, Urgency, …) | Anchor record fields shown in the runner's record summary; fields the step needs shown in the step |
| Application Context (organisation, industry, purpose) | Not part of a lane; that is app-level metadata, out of scope |
