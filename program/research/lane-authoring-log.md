# Lane authoring log — the four warehouse lanes, written by hand

WP-0719. The four warehouse lanes (Receive Shipment, Pick & Pack, Count,
Replenish) exist as definitions in `program/spec/schemas/examples/`. This log
records, for each one, what was hard to express by hand, what the schema
could not express at all, and what a visual builder would need to show. It is
the evidence for ADR-0025 decision 8 (the bar for starting the visual
builder, WP-0718): four lanes as definitions, at least one drafted through
the assistant, the publish checks (WP-0714), and this log.

The lanes are definitions only. Nothing here runs anywhere yet: no demo
runner change, no core code (ADR-0025 decision 4, the demo freeze). The
definitions validate against `process-lane.schema.json` (the CI check globs
the examples directory, so all four validate). Anything the schema cannot
express is recorded here and raised as a `Q-` item in `program/RAID.md`,
not silently worked around.

## Receive Shipment (`lane-wh-receiving`, on `purchase_order`)

The first lane, and the template the other three follow: 5 stages, 12 steps.

**Hard to express.** The discrepancy branch. A check whose fail goes to an
approval, whose approve returns to the middle of the flow (back to the
record stage with adjusted quantities) while reject ends the run held. The
branch targets are the whole design: the check must be the last step of its
stage, or the runner takes the next step in definition order and falls into
the hold branch on the happy path. That exact bug happened on Pick & Pack
(the happy path fell into the hold branch after step 1.5) and was fixed by
moving the check to the end of its stage. Writing the targets by hand means
holding the stage order in your head while you write each step; a typo in a
stage or step id fails silently at authoring time — the schema checks the
shape of a branch target, not that the target exists.

**The schema could not express.** The move numbers. The definition names the
move type and quantity (`warehouse.post_stock_move`, RECEIPT/PUTAWAY) but the
numbers (MV-2026-016..019) are assigned at write time, so the definition
cannot name them and the acceptance proof has to check them
sequence-relative. The partial-receipt semantics (38 of 40 arrive, 2
damaged: put away 38 only, PO PARTIALLY_RECEIVED) live in the approval's
writes and in a record status the schema does not enumerate.

**A builder would need to show.** A visual branch editor: draw the pass/fail
arrows and show where each one lands. A step-order warning: a check that is
not the last step of its stage changes the flow. A live preview of the run
against the seed record (PO-2026-001 with its two lines), so the author sees
the quantities the steps will post.

## Pick & Pack (`lane-wh-pickpack`, on `sales_order`)

5 stages, 13 steps. The hardest lane to write.

**Hard to express.** The `damage_approved` flag. The pick check must pass a
line that is fully picked AND (undamaged OR damage-approved). That needed a
nested any/all condition group AND a boolean run-data field set by the
approval's writes — and the approval's writes use `run_data.set`, which is
provisional, demo only (ADR-0025 decision 5; the definition's description
says so). The two were written together and broke together: the first
version looped forever, because after the replacement pick the damaged unit
still failed the check. The fix was the flag plus the nested condition. It is
the hardest single construct in any of the four lanes, and the one most
likely to be written wrong by hand.

**The schema could not express.** The per-line picked snapshot. The PICK
move posts only the quantity picked in this run, so the runner snapshots the
quantities already picked before the run (`_start_picked`) and posts the
delta. That snapshot is runtime state, not a definition field; the definition
says "only the quantity picked in this run" in prose. The parcel-split
proposal (the agent proposes 1 parcel / 8.0 kg; the packer confirms or
changes it): the definition can name the agent and the autonomy
(`wh-agt-packing`, propose) but not the proposal's content. The "ship is the
only stock-changing step" model (PICK/PACK are location transfers; only SHIP
decrements on hand and releases the reservation): the definition states it in
step descriptions, but the schema does not enforce it.

**A builder would need to show.** A condition builder for nested any/all
groups — the pick check's condition is unreadable as raw JSON. A map of
which run-data fields each step reads and writes, so the flag the check
depends on and the approval that sets it are visibly connected. A "what does
this step post?" preview: the StockMoves per line, with the delta the run
will actually post.

## Count (`lane-wh-count`, on `cycle_count`)

5 stages, 7 steps. The lane the demo's Counting section already does by
hand: CC-2026-001 (Zone A pick face, 5 lines, three variances: HG-1001 -2,
HG-1003 +1, HG-1005 -2).

**Hard to express.** The per-line variance approval. The demo's Approvals
inbox approves each variance line separately — approve HG-1001 and its
COUNT_ADJUSTMENT posts, while the count flips APPROVED only when all lines
are approved. But the schema forbids `repeat_over` on approval steps, so the
lane approves the whole count in one step. The per-line granularity the demo
already has is lost in the definition. The double-set of APPROVED is the
second wart: the approval sets it, and the close step sets it again, because
the no-variance path skips the approval entirely and something still has to
close the count as approved. Setting a status to the value it already has is
harmless, but it is two writes where one belongs.

**The schema could not express.** Per-item assignment (see the AC-2 answer
below). A per-item filter on the repeating record step: the post-adjustments
step repeats over all lines but must skip zero-variance lines — a
zero-quantity COUNT_ADJUSTMENT is not a real adjustment, and the definition
has no way to say "only lines with a variance". A Counter role: none exists
(`role_wh_counter` is not one of the warehouse roles), so the lane reuses
`role_wh_picker` — the seed's counted-by is user-05, a picker — and the
description carries the mismatch.

**A builder would need to show.** A per-line table: expected vs counted vs
variance, with the variance highlighted, because the whole lane is about
those three numbers. An approval step that shows the entire count's
variances at once, not one line at a time. A warning when a repeating record
step has no filter: it will post for every child, including the ones with
nothing to post.

### AC-2: several counters working at once — per-item assignment or parallel steps?

Per-item assignment, not parallel steps. ADR-0025 decision 6 names the
mechanism (WP-0710): the items of a repeating step are assigned to
different people and worked at the same time. A cycle count is the case it
was named for: each count line goes to a different counter, the step
completes when all items are done, and no step runs in parallel with
another. Parallel steps inside a stage stay out of scope.

But the current schema cannot declare per-item assignment. A repeating step
takes one `role`; there is no per-item assignee field. So the Count lane as
written assigns the whole count step to `role_wh_picker` and is single-role
until WP-0710 adds the declaration. That is the gap, raised as Q-33: how
does a repeating step declare a different assignee per item — a field on
`repeat_over`, a per-item role on the step, or something the runner derives
from the lines? Until that is decided, several counters at once is a runtime
capability the definition cannot ask for.

## Replenish (`lane-wh-replenish`, on `reorder_rule`)

4 stages, 7 steps. The shortest lane, and the one with the most guessing in
it.

**Hard to express.** The supplier. The reorder rule names the product, not
the supplier, so the draft PO's supplier cannot be derived from the anchor
record. The manager must confirm it as run data (`supplier_id`, a string the
person types), and the definition says so in the field's description. The
draft PO creation is a new provisional Action Type (`purchase_order.create`)
whose inputs (supplier, product, quantity, status) are a guess at the
catalog's shape — WP-0705 has not adopted it, and the definition will need
revising when the catalog lands. The "rule stays triggered until the goods
arrive" is a lifecycle rule the definition states in prose on the close
step, not in the schema.

**The schema could not express.** A lookup or reference from one record to
another (product → supplier). There is no way to say "the supplier is the
one on the product's most recent purchase order". A conditional write: the
lane cannot clear the rule's `triggered` flag on submission, because there
is no state for "being replenished" — the flag stays true until the stock is
received, and the definition can only note that.

**A builder would need to show.** A way to pick a related record (the
supplier) from the product, instead of typing an id. A preview of the draft
PO before it is created — supplier, product, quantity, total — because the
manager is approving a real order. A clear "what happens on reject": the
draft stays DRAFT, nothing is submitted.

## The five things that were hardest to write by hand

Ranked, hardest first. These are the five things a visual builder must
solve before hand-authoring stops being the bottleneck.

1. **Nested any/all check conditions.** The Pick & Pack damage check is a
   two-level condition group (fully picked AND (undamaged OR
   damage-approved)), and the boolean flag it depends on is set by a
   different step's writes. Written by hand, the condition and the flag
   drift apart; the first version looped forever. A builder needs a
   condition builder that shows the group structure and links each
   `$run_data` reference to the step that sets it.
2. **Branch targets and step order.** Every branch names a stage and a step
   by id, and the schema does not check that the target exists. Worse, a
   check that is not the last step of its stage silently changes the flow —
   the runner takes the next step in definition order. A builder needs to
   draw the arrows and warn when a check is not last.
3. **Run-data plumbing.** Every value a step reads or writes must be
   declared: run-data fields, `binds_to` paths, `$item` / `$record` /
   `$run_data` references. A typo in a path fails silently at authoring
   time — the schema checks the path's shape, not that the field exists.
   A builder needs to list, per step, what it reads and writes, and refuse
   a reference to a field that is not declared.
4. **The approval step's writes.** An approval must have at least one
   write, but the write that belongs to it is not always obvious. The Count
   lane's approval sets a status the close step also sets; the Pick & Pack
   approval sets a flag that is not a record write at all (and uses the
   provisional `run_data.set`). A builder needs to show what an approval
   actually writes, and what happens on each branch, before the author
   publishes.
5. **Provisional Action Types.** Every record step invents an Action Type
   id (`purchase_order.create`, `cycle_count.set_status`,
   `warehouse.post_stock_move`) that the catalog (WP-0705) has not adopted
   yet. The definition is a guess at the catalog's shape, and each guess
   will need revising when the catalog lands. A builder needs the catalog:
   a picker of real Action Types with their inputs, not a free-text id.
