# Schema-extension proposal — RAID Q-33..Q-36

**Status: Proposed, for the owner to accept.** Written 2026-11-14 on the
ADR-0025 prep track (Claude's instruction, ferried by the owner).

This document proposes the schema shape for each of the four gaps raised
as RAID questions in WP-0719 (Count and Replenish lanes as definitions).
It changes nothing: `program/spec/schemas/process-lane.schema.json` is
untouched, no lane definition is edited, no core code changes. The schema
changes only after the owner accepts each item — and each item is accepted
separately. The owner may accept Q-33 and Q-36 but not Q-35, for example;
acceptance of one does not imply acceptance of the others.

## Background

The four warehouse lanes (Receive Shipment, Pick & Pack, Count,
Replenish) exist as definitions in `program/spec/schemas/examples/`, and
they validate against the current schema. Writing them by hand exposed
four things the schema cannot express. Each is recorded in
`program/research/lane-authoring-log.md` and as a `Q-` item in
`program/RAID.md`, with a default the lanes follow today. This proposal
replaces the defaults with a schema shape, so the engine (WP-0710 for
Q-33/Q-34, WP-0705 for Q-35) can build them cleanly instead of working
around them.

The current defaults, for reference:

- Q-33: the Count lane assigns the whole repeating step to
  `role_wh_picker` (single-role); per-item assignment waits for WP-0710.
- Q-34: the Count post-adjustments step posts per line with quantity =
  variance, and notes the runner must skip zero-variance lines.
- Q-35: the manager confirms `supplier_id` as run data at step 1.1.
- Q-36: reuse `role_wh_picker` until a Counter role is added.

## Q-33 — Per-item assignment on a repeating step

**Question (RAID Q-33).** How does a repeating step declare a different
assignee per item (WP-0710 per-item assignment)? The Count lane's count
step assigns the whole step to one role; several counters working at
once needs a per-item assignee, and the schema has no field for it.

**Proposed shape.** Add an optional `assign` object to `repeat_over`:

```json
"repeat_over": {
  "field": "lines",
  "item_id_field": "id",
  "assign": {
    "by": "child_field",
    "field": "counted_by"
  }
}
```

- `assign.by`: `"child_field"` or `"role"` (default `"role"` — the
  step's role, which is today's behaviour).
- `assign.field`: required when `by` is `"child_field"`. The child
  field holding the assignee for that item. The value is a role id (any
  holder of the role can take the item) or a user id (that person).
- The step's `role` becomes the fallback: a child with no assignee in
  its field is assigned to the step's role.

**Semantics (runtime, WP-0710).** Each child item is assigned to the
role or user its field names; the items are worked at the same time;
the step completes when all items are done. The runner records, per
item, who did it — that is run state, not a definition field. A
per-item assignment does not make steps parallel: no step runs in
parallel with another (ADR-0025 decision 6).

**Example.** The Count lane's count step:

```json
{
  "id": "count-lines",
  "kind": "form",
  "role": "role_wh_picker",
  "repeat_over": {
    "field": "lines",
    "assign": { "by": "child_field", "field": "counted_by" }
  },
  "fields": [
    { "key": "counted_qty", "label": "Counted qty", "type": "integer",
      "binds_to": "record.lines[].counted_qty" }
  ]
}
```

Each count line's `counted_by` names its counter; the five lines of
CC-2026-001 go to different counters and are counted at the same time.
Note the consequence: the cycle-count line object needs a `counted_by`
field. That is an object-model / seed change, not a schema change — the
seed's CC-2026-001 lines would carry it, and the demo's Counting section
would set it when the manager assigns the count.

**Alternative considered: a per-item role list on the step**
(`[{item, role}, …]`). Rejected. It duplicates the child data into the
definition, and it goes stale when the lines change (a line added or
removed after the lane was written). The child field is the single
source of truth; the definition names the field, not the assignments.

**On acceptance.** WP-0710 implements `repeat_over.assign` (schema +
runtime + the per-item completion rule) and the Count lane is updated
to use it. The builder (WP-0718) gains the per-item assignment editor:
it shows the child field picker and, for the seed record, which item
goes to whom.

## Q-34 — Skip children with nothing to post

**Question (RAID Q-34).** When a repeating record step posts for every
child, how does it skip children with nothing to post (Count:
zero-variance lines must not post a zero-quantity COUNT_ADJUSTMENT)?
Is the skip a condition on the step, or does the Action Type skip them?

**Proposed shape.** Add an optional `skip_when` condition to a step,
alongside `repeat_over`. It is a `condition_or_group` — the same grammar
as check conditions:

```json
{
  "id": "post-count-adjustments",
  "kind": "record",
  "role": "role_wh_warehouse_manager",
  "repeat_over": { "field": "lines" },
  "skip_when": {
    "condition": { "field": "$item.variance", "operator": "equals", "value": 0 }
  },
  "writes": [
    { "action_type": "warehouse.post_stock_move",
      "inputs": { "move_type": "COUNT_ADJUSTMENT", "quantity": "$item.variance" } }
  ]
}
```

**Semantics (runtime, WP-0710).** `skip_when` is evaluated per child
before the step posts for that child. A child that matches is skipped:
no write is posted for it, and the skip is recorded in the run and the
audit (so the count's history shows which lines were skipped and why).
The step completes when all non-skipped items are done. If every child
is skipped, the step completes with zero writes and the run continues —
this is the no-variance path, where the close step still sets the count
APPROVED. The dry run (WP-0714) asserts the skip with
`write.not_posted`.

**Rules.** `skip_when` is only meaningful on a repeating step
(`repeat_over` present); on a non-repeating step it is a schema error
(`validate_definition`). It may reference `$item` fields and `$record`
fields, and `$run_data` fields set by earlier steps — never a field set
later in the run.

**Example.** The Count lane's post-adjustments step, above: the three
variance lines (HG-1001 −2, HG-1003 +1, HG-1005 −2) post a
COUNT_ADJUSTMENT each; the two zero-variance lines post nothing.

**Alternative considered: the Action Type skips zero-quantity writes
itself.** Rejected. It hides the decision inside the catalog, so the
definition cannot state it, the dry run cannot assert it, and the
builder cannot show it. A zero-quantity adjustment is not a real
adjustment — the definition should say which lines are skipped, in the
open, where the author and the test can see it. The Action Type still
refuses a zero-quantity write if one reaches it (a safety net, not the
rule).

**On acceptance.** WP-0710 implements `skip_when` (schema + runtime +
the skip recording) and the Count lane is updated to use it. The
builder (WP-0718) gains the filter editor on a repeating record step
and warns when a repeating record step has no filter — it will post
for every child, including the ones with nothing to post.

## Q-35 — A product → supplier reference / lookup

**Question (RAID Q-35).** The Replenish lane's draft PO needs a
supplier, but the reorder_rule names the product, not the supplier. Who
resolves the supplier — the `purchase_order.create` Action Type (from
the product's most recent PO), or a schema reference / lookup?

**Proposed shape.** Add an optional `resolve` object to a form field
(and, for completeness, to a write input). It names the record to look
up, how to find it, and which field to take:

```json
{
  "id": "supplier_id",
  "label": "Supplier",
  "type": "string",
  "binds_to": "run_data.supplier_id",
  "resolve": {
    "kind": "latest_record",
    "object": "purchase_order",
    "where": { "product_id": "$record.product_id" },
    "order_by": "order_date",
    "direction": "desc",
    "take": 1,
    "field": "supplier_id"
  }
}
```

- `resolve.kind`: `"latest_record"` (v1 — the only kind proposed).
  Other kinds (`first_match`, `count`) are future work, not this
  proposal.
- `resolve.object`: the object to look in.
- `resolve.where`: field → value, where a value may be a `$record.*`
  or `$run_data.*` reference or a literal.
- `resolve.order_by` / `direction` / `take`: which matching record
  wins (latest by `order_date`, first one).
- `resolve.field`: the field to take from the winning record.

**Semantics.** The field's suggested value is resolved at step time
from the target workspace's data; the person confirms or overrides it.
The builder shows it as a pick (the supplier's name, not a typed id),
not as a free-text field. The resolved value is available as
`$run_data.supplier_id`. If no record matches, the field is empty and
the person types it, with the reason shown ("no purchase order found
for this product"). The dry run (WP-0714) shows the resolved value for
the seed record, so the author sees which supplier the lane would
suggest.

**Example.** The Replenish lane's step 1.1: the supplier field resolves
to the supplier on the product's most recent purchase order; the
manager confirms it; the draft PO is created with that supplier. The
"rule stays triggered until the goods arrive" lifecycle is unchanged —
this proposal does not add a "being replenished" state; that stays a
separate question.

**Alternative considered: the `purchase_order.create` Action Type
resolves the supplier itself.** Rejected as the primary mechanism. It
hides a business rule ("the supplier is the one on the product's most
recent PO") inside the catalog, so the definition cannot state it, the
builder cannot show the pick, and the dry run cannot show the resolved
supplier. The Action Type still validates the supplier (it must be a
real supplier) — validation stays in the catalog; resolution is
declarative in the definition, where the author can see and test it.

**Deliberate narrowness.** This is the only one of the four proposals
that adds a query-like mechanism to the schema, so it is kept narrow
on purpose: one kind (`latest_record`), one field, no joins, no
aggregation, no expressions in `where`. If the owner wants more, that
is a separate proposal — this one does not grow into a query language.

**On acceptance.** A small follow-up after WP-0705 adopts the `resolve`
shape (schema + the resolution at step time + the Action Type
validation), and the Replenish lane is updated to use it. WP-0705 itself
merged without it (PR #775) — so this is a follow-up, not inside WP-0705.
The builder (WP-0718) gains the related-record picker: pick a supplier
from the product, instead of typing an id.

## Q-36 — Add a Counter role, or keep reusing Picker?

**Question (RAID Q-36).** There is no Counter role (`role_wh_counter`)
in the warehouse roles; the Count lane reuses `role_wh_picker` (the
seed's counted-by is a picker). Add a Counter role, or keep reusing
Picker?

**Proposal: add `role_wh_counter`.**

This one is not a schema change. Roles are values — the role model and
the demo seed — not schema fields. The proposal is to add the role to
`ensureWarehouseRoles` (demo) and to the role model where the warehouse
roles land on desktop / Team Workspace, and to change the Count lane's
count step role from `role_wh_picker` to `role_wh_counter`.

**Why add it.** A counter is a distinct job: blind counts, variance
review, recounts. Reusing Picker conflates two jobs in the role model,
and the seed carries the mismatch in the open — the lane's description
notes that the counted-by (user-05) is a picker. The variance-review
step is already a different role (the warehouse manager), so the lane
already distinguishes jobs; the count step should too.

**Consequences.** The seed needs a counter user: either user-05's role
changes from picker to counter, or a counter user is added and the
Count lane's `counted_by` points at them. The demo's Counting section
role checks must accept `role_wh_counter`. The lane definition change
is one line (the step's `role`) plus removing the description note
about the mismatch. The builder shows the Counter role in the role
picker once it exists.

**Alternative considered: keep reusing Picker.** Rejected. It saves
one role and keeps the conflation. The cost of adding the role is a
seed change and a role-check change; the cost of not adding it is a
role model that cannot say who counts, which is the Count lane's whole
job.

**On acceptance.** The role is added (demo seed + role model), the
Count lane's count step changes role, and the description note is
removed. No WP-0710 work: this is a role-model change, not a schema
change.

## What acceptance means, per item

| Item | Accepted by | Implemented by | Lane updated |
|---|---|---|---|
| Q-33 `repeat_over.assign` | Owner | WP-0710 (schema + runtime) | Count — count step assigns per item |
| Q-34 `skip_when` | Owner | WP-0710 (schema + runtime) | Count — post-adjustments skips zero-variance lines |
| Q-35 `resolve` (lookup) | Owner | Follow-up to WP-0705 (schema + resolution) | Replenish — supplier resolves from the latest PO |
| Q-36 `role_wh_counter` | Owner | Role-model / seed change (no WP) | Count — count step role becomes Counter |

Each RAID item closes when its row is accepted **and** implemented
(the implementation is tracked in the named WP, or, for Q-36, in the
seed change). Acceptance alone does not close the item — it authorises
the implementation. Until then, the four lanes keep their current
defaults, and the definitions stay as they are.
