# ADR-0024: Process Lane model and definition schema

- Status: Proposed
- Date: 2026-11-12
- Work package: WP-0701
- Requirements: REQ-LANE-001..025, REQ-NFR-008
- Amends: ADR-0007 (run and step state machine — two new waiting statuses)
- Source: the owner's Process Lane spec, [`../spec/process-lane-spec.md`](../spec/process-lane-spec.md), as changed by ADR-0023 decision 4 (which wins where they differ)
- Design: [`../design/SOLUTION-DESIGN.md`](../design/SOLUTION-DESIGN.md) section 17
- Schema: [`../spec/schemas/process-lane.schema.json`](../spec/schemas/process-lane.schema.json)
- Example: [`../spec/schemas/examples/lane-wh-receiving.json`](../spec/schemas/examples/lane-wh-receiving.json)

## Context

A Process Lane is a guided, single-screen, step-by-step execution of one business process for one
record. The owner specified it in a draft spec (2026-11-12) modelled on the pattern of
case-management screens, in Lanesra's own words (clean room — nothing copied). ADR-0023 accepted
the feature as Phase 7 and changed the spec in ten places; this ADR fixes the model those changes
imply, so the online demo (WP-0702) and the core (WP-0703 onward) build from one definition.

What the platform has today:

- The graph runtime (`ai_execution_graphs`, `ai_runs`, `ai_run_nodes`, migration 0062) has
  condition, action, agent, approval and loop nodes, durable runs, checkpoints, trace, approvals
  and audit. It has **no node that waits for a person to do work** other than an approval.
  Node types are declared in `desktop/core/src/models/execution_graph.rs:15-40` (`NODE_TYPES`);
  source kinds in `GRAPH_SOURCE_KINDS` (`desktop/core/src/models/execution_graph.rs:64`) are
  `workflow` and `pipeline` only.
- Run statuses are the ADR-0007 §1 set, declared in
  `desktop/core/src/models/graph_run.rs:11-24` (`RUN_STATUSES`); step statuses are ADR-0007 §2
  (`RUN_NODE_STATUSES`). Every status change goes through one guard function,
  `run_state::transition`, and nothing leaves a terminal status. ADR-0007 §6: adding a status
  later needs an ADR that amends it — this ADR is that amendment.
- `ai_runs` today carries `graph_id`, `status`, `trigger_input`, `context_json`,
  `current_node_id`, `pending_approval_id`, `resume_at`, `source_entity_type` /
  `source_entity_id`, `steps_executed`, `started_at`, `finished_at` (migration 0062). It has no
  anchor-record column, no run-data column, and no lane columns. `ai_run_nodes` has no stage
  column and no entered/completed times. SOLUTION-DESIGN §18 already names these additions
  (`ai_runs` gains anchor record and run data; `ai_run_nodes` gains stage and step times);
  WP-0703/WP-0704 implement them.
- Action Types (`action_type_service::invoke`) are governed writes that obey Business Rules.
  Idempotency keys for repeat-safe writes are WP-0141, not yet built; the first usable lane
  version waits for it (ADR-0023).
- Business Rules conditions (`desktop/core/src/models/business_rule.rs:101-130`,
  `BusinessRuleCondition`) compare one field against a value or another field, with operators
  from `desktop/core/src/domain/conditions.rs:20-35` (`CONDITION_OPERATORS`: equals, not_equals,
  contains, not_contains, starts_with, ends_with, in_list, not_in_list, is_empty, is_not_empty,
  greater_than, less_than, on_or_after, on_or_before) and match types `all` / `any`.
- The online demo shares a definition format with the core by schema, per ADR-0015: one JSON
  Schema is the single contract, schema-first, snake_case, `format_version` semver, no dual
  format. The precedent is `program/spec/schemas/team-format.schema.json` (draft 2020-12,
  `$id` under `https://lanesraos.com/spec/schemas/`).
- Vocabulary is locked (REQ-CORE-008, REQ-LANE-003): lane, stage, step, run, anchor record.
  "Case" and "flow" are not used. "Run" and "Step" are the runtime's words.

## Decision

### 1. The model

| Term | Meaning | Stored as |
|---|---|---|
| **Process Lane** | The definition: a named business process on one Custom Object, made of ordered stages and steps, with declared run data. Versioned; Draft → Published; a run keeps the version it started with. | `process_lanes` + `process_lane_versions` (WP-0703); in the demo, a JSON literal in the same schema |
| **Stage** | A named phase of the lane, one chevron in the stage bar. **Grouping only; it does not execute.** Order is the order of the `stages` array — there is no separate `order` field. | Stage id on each compiled node (see §3) |
| **Step** | One unit of work inside a stage. Exactly one step is current at a time (v1: steps are sequential). | One compiled graph node (see §3); one `ai_run_nodes` row per attempt |
| **Anchor record** | The one business record the run is about (a row of a Custom Object). A lane runs on one object; a step that needs a second object writes it — the run stays anchored to the one record. | `ai_runs.anchor_object_key` + `ai_runs.anchor_record_id` (WP-0703) |
| **Run** | One live execution of a lane for one anchor record. It **is** a Run in the existing run tables — not a new kind of object, not a `LaneRun` with its own statuses. | `ai_runs` row whose graph has source kind `lane` |
| **Run data** | Values the lane collects that are not record fields, with a schema declared in the lane definition. Every value a person enters is stored in run data or mapped to a named record field — nothing entered is lost. | `ai_runs.run_data_json`, validated against the lane's declared `run_data` schema (WP-0704) |
| **Branch** | A declared route from a step's outcome to another step, or to the end of the run with a named outcome. | Edges on the compiled graph (§3); `branches` on the step in the definition |
| **Outcome** | The named result a run ends with. `done` for a run that reaches the end; a branch may end the run with another named outcome (for example `held`). Recorded in run data and shown in the label. | `run_data.outcome` |

A lane definition is data in one JSON Schema, `program/spec/schemas/process-lane.schema.json`,
shared by the online demo, desktop and Team Workspace (REQ-LANE-001). The schema is the single
contract: schema-first, types derived from it, never the reverse (ADR-0015). Breaking changes
bump the `format_version` major version and are a new work package, never a silent edit.

A published lane is never edited in place. A change publishes a new version; runs already
started finish on the version they started with (WP-0703 AC-2). A lane with no published version
cannot start a run. There is no `disabled` status in v1: retiring a lane means publishing a
successor or deleting the draft; a lane whose latest version is a draft simply has nothing to
run.

### 2. The definition (what the schema carries)

A lane definition has: `id` (`lane-` + kebab-case), `name`, `description`, `object_key` (the
Custom Object the lane runs on), `version` (integer, ≥ 1), `status` (`draft` | `published`),
`stages` (≥ 1, ordered), `run_data` (declared fields), optional `start_condition`, optional
`excludes` (lane ids that may not run concurrently on the same record).

A stage has: `id`, `name`, `steps` (≥ 1, ordered), optional `entry_condition`, optional
`skip_condition`.

A step has: `id`, `name`, `kind` (form | record | check | approval | agent), `role` (the role
that may do it; a check step has none — it runs automatically), `required` (default true),
optional `description`, optional `assistance` (agent help + autonomy level, §7), optional
`fields` (form steps), optional `writes` (record and approval steps), optional `validation`
(any step — a rule that must hold before the step completes), optional `branches` (check and
approval steps only), optional `repeat_over` (a step that repeats over child records), optional
`ends_run` (completing this step ends the run with a named outcome), optional `notify` (automatic
notification on completion), optional `replaces` (workflow actions this step replaces, §8),
optional `child_lane` (a step that starts another lane and waits, §9).

Step order inside a stage, and stage order inside a lane, is array order. There is no `order`
field anywhere: two builders reading the same definition must produce the same order, and a
second ordering field is a second source of truth.

### 3. How a lane compiles to an execution graph

A lane compiles to an execution graph with `source_kind = "lane"` and
`source_id = "{lane_id}@v{version}"` (WP-0703; the compiled graph must pass
`validate_for_publish`). The graph has one node per step, plus one `condition` node per stage
entry/skip condition, and the lane's `start_condition` is evaluated before the run starts —
if it fails, the run is refused with the reason, and no run row is created. One step compiles
to exactly one node. The spec labels three steps "form + record" (1.2, 3.1, 3.2 in Part B):
those split into a form step and a record step in the definition — a step that collects a value
and a step that posts a write are two units of work, and a hybrid would need two nodes for one
step. The example lane in this package shows the split.

Every compiled node records its lane position in its config: `lane_stage_id` and
`lane_step_id`. The runner renders the stage bar from the nodes' stage ids; the run reports
stage and step times from them. A stage contributes no node of its own — it is grouping only.

| Step kind | Compiles to | Node config (in addition to `lane_stage_id` / `lane_step_id`) |
|---|---|---|
| form | **`human_step`** (new node type, §4) | `fields` (each with `binds_to`: `run_data.<field>` or `record.<field>`, or `record.<collection>[].<field>` inside a repeating step), `validation`, `role`, `assistance`, `repeat_over` |
| record | `action` — calls a governed Action Type | `action_type`, `inputs` (a mapping from input name to a literal or a `$` reference: `$record.<field>`, `$run_data.<field>`, `$item.<field>` inside a repeating step), `replaces`. The idempotency key is runtime-generated, one per step attempt (WP-0141) — it is not declared in the definition |
| check | `condition` | the check rule (§5); its `pass` / `fail` outcomes are the node's two branches |
| approval | `approval` — through the approvals inbox (`ai_approvals`, WP-0150) | `role`, the guarded `writes` (posted only on approve), `branches` (`approved` / `rejected`) |
| agent | `agent` | `pipeline` / `skill`, `autonomy` level, whether the step needs a person's confirmation before its writes post |

The `human_step` node is the only new node type. The other four kinds reuse nodes the engine
already has; their lane behaviour is config, not new code.

A step that repeats over child records (`repeat_over: { "field": "<field on the anchor record
holding the child array>", "item_id_field": "<field holding the child's id, default \"id\">" }`)
compiles to a `loop` node whose body is the step's node. Each iteration is one attempt row of
the body node (`ai_run_nodes.attempt`), so a repeated step's iterations are individually
inspectable, exactly as a bounded loop's iterations are today (ADR-0007 §2). Inside the body,
`$item.<field>` binds to the current child record, and a `$run_data.<field>` reference to a
`per_child` run-data field resolves to the current child record's value.

A stage `entry_condition` compiles to a `condition` node placed before the stage's first step:
if it fails, the stage is skipped. A stage `skip_condition` compiles the same way: if it holds,
the stage is skipped. Skipping a stage records each of its steps as `skipped` with reason
`stage_skipped`.

### 4. The human step node

New node type: **`human_step`**. It is the engine's one mechanism for waiting on a person to do
work. A human seat's work item (WP-0560) is a lane human step (ADR-0023 decision 7); Guided
Intake (WP-0631) is the form-only case of a lane.

Config:

```json
{
  "lane_stage_id": "verify",
  "lane_step_id": "confirm-arrival",
  "role": "role_wh_receiver",
  "fields": [
    { "key": "carrier", "label": "Carrier", "type": "string", "required": true,
      "binds_to": "run_data.carrier" }
  ],
  "validation": { "condition": { "field": "$run_data.received_date", "operator": "equals",
    "compare_field": "$record.expected_date" },
    "message": "Received date must be the expected date" },
  "assistance": { "autonomy": "propose", "agent": "wh-agt-receiving" },
  "repeat_over": null
}
```

Runtime behaviour (WP-0704): when the worker reaches a `human_step` node, the run moves to
`waiting_person` (§6) and the step row sits in `waiting_person`. No worker holds a lease while
a person is doing the work. When the person submits the step, the runtime validates the entered
values against the lane's run-data schema and the step's `validation`, applies the field
mappings (run-data fields into `run_data_json`, record fields through the same governed write
path a direct edit uses), and completes the node. A `human_step` node never writes a record
directly — a form step's record mappings go through the governed write path, and a step that
posts records is a `record` step, not a form step.

### 5. Conditions

Stage entry/skip conditions, the lane start condition, step `validation`, and check-step rules
all use the same condition language: Business Rules operators
(`desktop/core/src/domain/conditions.rs:20-35` — equals, not_equals, contains, not_contains,
starts_with, ends_with, in_list, not_in_list, is_empty, is_not_empty, greater_than, less_than,
on_or_after, on_or_before), with match `all` / `any` for groups. A condition compares one field
against a literal `value` or against another field (`compare_field`), mirroring
`BusinessRuleCondition`.

Fields are referenced by scope:

- `$record.<field>` — a field of the anchor record (`$record.status`,
  `$record.expected_date`).
- `$run_data.<field>` — a declared run-data field (`$run_data.carrier`).
- `$item.<field>` — a field of the current child record, inside a repeating step only
  (`$item.qty_received`). Outside a repeating step it is an error (publish check, WP-0714).

One check in the example cannot be expressed as a single operator comparison: step 2.1's
`qty_received + qty_rejected ≤ qty_ordered` per line is arithmetic. For that, and only that, a
validation or check may carry an `expression` string in this narrow grammar:

```text
expression := sum comparator operand
sum        := operand ("+" operand)*
operand    := "$record." field | "$run_data." field | "$item." field | number
comparator := "<=" | "<" | ">=" | ">" | "==" | "!="
```

Example: `"$item.qty_received + $item.qty_rejected <= $item.qty_ordered"`. The schema
constrains the shape with a pattern; evaluation is WP-0710's. No other expression language is
admitted in v1 — a lane that needs more than a sum comparison is a question for the owner, not
a silent extension.

A check step evaluates its rule and produces outcome `pass` or `fail`. Its `scope` is `anchor`
(evaluate once against the anchor record and run data) or `per_item` (evaluate once per child
record, aggregated with `all` or `any` across the children, declared on the check). The
discrepancy check in the example is `per_item` with aggregate `all`: it passes only when every
line is fully received.

### 6. Statuses — amending ADR-0007

This ADR amends ADR-0007 by adding **two** waiting statuses. Everything else in ADR-0007 is
unchanged: one guard function, no status written directly in SQL, nothing leaves a terminal
status, replay and fork create a new run.

| Status | Meaning | Terminal | Holds a lease |
|---|---|---|---|
| `waiting_person` | **New.** The run waits for a person to do a human step (or an approval that is not an `approval` node — there is none in v1). The step row sits in `waiting_person`. | no | no |
| `waiting_child` | **New.** The run waits for a child lane run it started (§9). | no | no |

Both are waiting statuses: `is_waiting_status` returns true for them, `resume_run` accepts them,
and the interrupted-run sweep treats them as durable waits (they survive a restart exactly as
`waiting_approval` does). Step statuses gain the same two values: a step row may sit in
`waiting_person` or `waiting_child` alongside the existing `waiting_approval`.

The spec's own status set (`IN_PROGRESS`, `WAITING_APPROVAL`, `ON_HOLD`, `DONE`, `CANCELLED`) is
replaced. In particular there is **no `ON_HOLD` run status**: a hold is a branch that ends the
run with a named outcome (§10), not a state a run sits in.

`stop_reason` gains no new values in v1: a run that ends with a named outcome records the
outcome in run data (`run_data.outcome`) and finishes `completed`; the existing `end_reached`
covers the stop reason.

How the lane's life-cycle verbs map onto the state machine:

| Verb | Mapping | Notes |
|---|---|---|
| Pause | Any non-terminal status → `paused` (via `run_state::transition`). The current step and its attempt are kept; the run's current stage/step are read from the current node. | Resume: `paused` → `queued`; the worker picks the run up at its current node. |
| Handoff / reassign | The current step's assignee changes to another person or role. The run status does **not** change. The handoff (who, from whom, when) is written to the audit trail. | WP-0704 / WP-0711. A handoff to a role the step does not allow is refused. |
| Skip | Only an optional step (`required: false`) can be skipped. Its step row becomes `skipped` with who, when and why; the run advances to the next node. | Skipping a required step is refused. A skipped step is never retried by the run. |
| Reopen a finished step | Allowed only while the step's stage is still the current stage, and only while the run is `waiting_person` or `paused`. It creates a **new attempt row** for that node (attempt + 1) and sets the current node back to it. It reverses nothing: corrections are new writes, never edits to posted records. | The previous completed attempt stays in the audit trail. Reopening a step in a finished stage is refused. |
| Cancel | Any non-terminal status → `cancelled` (terminal, `stop_reason: cancelled_by_user`). Posted writes stay in place; the cancel lists them — the runner shows what the run posted, from the audit trail and run data. | A cancelled run is final. Starting again means a new run (allowed: the cancelled run is not active). |
| Finished | The run reaches the end node, or a step's `ends_run` fires, or a branch's `end` fires → `completed` (`stop_reason: end_reached`). If any step ended `completed_partial`, the run is `completed_partial`. | A finished run is final (ADR-0007 §3). A correction after finish is a new run or a manual record change, recorded as such. |
| Hold | **Not a status.** The hold branch ends the run with the named outcome `held` (see §10). The work resumes as a new run, which sees the held run's audit trail. | This is the deliberate replacement for the spec's `ON_HOLD`. |

One active run per lane per record (REQ-LANE-010): starting a second run of the same lane on the
same anchor record is refused, with a link to the active run. Different lanes may run on the
same record unless the lanes declare that they exclude each other (`excludes` on the lane).
Enforcement is at run start (WP-0703); the demo enforces the same rule against its store.

Every stage and step records `entered_at` and `completed_at` (SOLUTION-DESIGN §18:
`ai_run_nodes` gains stage and step times; WP-0704). These feed the board, "my steps" and the
lane's time-per-stage reporting (REQ-LANE-019). There are no timers or SLA escalation in v1 —
the times are recorded from the start so a later package can use them.

### 7. Agent assistance and autonomy

Any step may carry `assistance: { "autonomy": "<level>", "agent": "<agent or pipeline id>" }`
(REQ-LANE-016):

| Level | Meaning |
|---|---|
| `person` | No agent. The person does the step. (Default when `assistance` is absent.) |
| `propose` | The agent prefills values as a proposal; the person confirms or overrides. |
| `approve` | The agent does the work; the write waits in the approvals inbox before it posts. |
| `auto` | The agent does the work; the write goes through the Tool-Call Firewall like any agent call. |

Every acceptance and every override of an agent proposal is recorded per step (in the step row
/ audit trail). The override rate is the evidence for moving a step up a level later; it feeds
lessons from corrections (REQ-MEM-010). An `agent` step kind is the same mechanism with the
agent as the primary actor: its autonomy is `auto` unless the step declares otherwise, and a
step that requires a person's confirmation declares `approve` or `propose`.

Access (REQ-LANE-011): a step assigned to a role the viewer lacks shows its name and who it is
waiting on, but is locked, and its content is shown only as far as the viewer's record and
field access allows. This changes the spec (A3.5), which hid nothing. An agent step in a run a
person started acts as that person narrowed by the agent's scope — not as the agent alone
(changes the spec, ADR-0023 decision 4, REQ-GOV-002). In an unattended run it acts as its Agent
Identity (WP-0107–WP-0109).

### 8. Workflows and the "replaces" rule

Workflows fire by default on what a lane writes (REQ-LANE-013). A lane step may declare the
workflow actions it replaces, and only those are not fired for that step's writes:

```json
"replaces": [{ "workflow_id": "wf-wh-01", "action_index": 0 }]
```

`action_index` is the position of the action in the workflow's `actions` array — workflow
actions have no stable ids in the demo or the core today, so the index is the reference. It is
fragile by nature: the publish check (WP-0714) verifies the workflow exists and has an action at
that index, and a workflow edit that reorders its actions must re-check every lane that
replaces it. If workflow actions gain stable ids later, `replaces` gains an `action_id`
alternative; v1 uses the index.

The declaration is visible in three places: in the lane definition (the `replaces` field), in
the workflow (which shows "replaced by lane step X" — WP-0709), and in the Dependency Explorer.
A step that does not declare a replacement replaces nothing — a lane never silently owns a
transition. This replaces the spec's proposed rule (A3.7 / O-2) that the lane owns the journey's
transitions; the owner accepted the change in ADR-0023 decision 4.

A workflow can start a lane run (REQ-LANE-013): a workflow action of type "start lane" names
the lane id; the run starts under the same one-active-run rule (§6) and is refused with a link
if one is already active. The action type itself is WP-0709's.

### 9. Child lanes

A step may start a child lane on a related record and wait for its outcome (REQ-LANE-023,
WP-0716):

```json
"child_lane": { "lane_id": "lane-wh-count", "anchor_record": "$record.supplier_id" }
```

`anchor_record` is a `$` reference naming the related record the child lane runs on. The parent
run's step sits in `waiting_child` (§6) until the child run finishes; the parent shows the
child's stage. The child's outcome is recorded in the parent's run data. Depth is limited
(WP-0716 sets the limit; a lane that would exceed it is refused at start). The example lane has
no child lane; the field exists so the definition format is stable before WP-0716.

### 10. Branches and their exits

Branches are declared in the lane definition (outcome → target step), never improvised at
runtime (REQ-LANE-008). Only check and approval steps declare branches: a check step's outcomes
are `pass` / `fail`; an approval step's are `approved` / `rejected`. Other step kinds complete
and advance to the next step in order.

```json
"branches": [
  { "on": "fail", "target": { "stage": "hold-review", "step": "review-discrepancy" } },
  { "on": "pass", "target": { "stage": "stage-put-away", "step": "stage-goods" } }
]
```

Every branch must have a declared way out: `target` (another step — continue or route
elsewhere) or `end` (finish the run with a named outcome):

```json
{ "on": "rejected", "end": { "outcome": "held" } }
```

A lane with a dead end cannot be published (WP-0714's publish check enforces it; the schema
enforces the shape with `oneOf` — each branch has exactly one of `target` or `end`). A step may
also declare `ends_run: { "outcome": "<name>" }`: completing that step ends the run with the
named outcome instead of advancing. The example's hold step uses `ends_run` with outcome
`held`.

The spec's hold branch ended in `ON_HOLD` with no exit — that is the dead end this rule
removes. In the example, rejecting at manager review routes to a "Record hold" step (the
manager records the hold reason in run data), and completing that step ends the run with
outcome `held`. Nothing is posted for the disputed lines, put-away never happens for them, and
the run's audit trail holds the decision. Resuming the receipt is a new run on the same PO.

### 11. How the live item is labelled

Internally the live instance is a Run (REQ-LANE-003). In the product it is labelled by its
anchor record, lane and stage:

```text
{anchor record display label} · {lane name} · stage {n} of {N}
```

Example: `PO-2026-001 · Receive Shipment · stage 2 of 4`. The anchor record display label is
the record's number where the object has one (the purchase order's `po_number`), else its name.
While the run is active the label shows the current stage position. When the run finishes it
shows the outcome instead of a stage: `PO-2026-001 · Receive Shipment · done`, or
`PO-2026-001 · Receive Shipment · held`. This settles the spec's O-5: the instance is a run,
labelled by record, lane and stage — not the record itself, and not a separate "case".

### 12. Mapping the spec (A3.1–A3.9) to the model — AC-1

| Spec | Maps to | Change from the spec |
|---|---|---|
| A3.1 Lane definition (`id`, `name`, `description`, `objectKey`, `version`, `stages[]`, stage `id/name/order`, step `id/name/kind/role/required/writes/validation/onComplete`) | The JSON Schema (§2). `objectKey` → `object_key` (snake_case, ADR-0015). | `order` dropped — array order is canonical. `onComplete` removed — completion is the graph's end node or a step's `ends_run`. Added: `status` (draft/published), `run_data`, `start_condition`, `excludes`. A lane runs on one object, unchanged. |
| A3.2 Step kinds (form, record, check, approval, agent) | §3 compile table. | Notify is not a kind — it is an automatic effect (`notify` on a step), unchanged from the spec. Added: `assistance` + autonomy level on any step (§7); `validation` on any step; check steps have no `role`. **The spec's "form + record" hybrid steps split** into a form step and a record step, so each step compiles to exactly one node (§3). |
| A3.3 Run (`LaneRun`, statuses `IN_PROGRESS/WAITING_APPROVAL/ON_HOLD/DONE/CANCELLED`, `currentStageId/currentStepId`, `stepStates[]`, `audit[]`) | A Run in `ai_runs` / `ai_run_nodes` (§1, §6). Current stage/step are read from the current node. Audit is the run's trace + audit trail. | The `LaneRun` object and its status set are **replaced** by the ADR-0007 statuses plus `waiting_person` (and `waiting_child`). **One active run per anchor record → one per lane per record**, with `excludes`. Pause/resume/handoff as mapped in §6. Skip only for optional steps, unchanged. Reopen only while the stage is current, as a new attempt row — unchanged in intent, new in mechanism. `DONE` → `completed`; `CANCELLED` → `cancelled`. |
| A3.4 Gating and branching (required step gates the next; branches declared; approval branches; done is terminal) | §5, §10. | Added: **every branch must declare a way out** (continue, another step, or end with a named outcome); a dead end cannot be published. The hold branch's `ON_HOLD` end is replaced by ending with outcome `held`. |
| A3.5 Roles and permissions (locked step visible, waiting-on name shown; agent steps under the agent's identity) | §7. | **Changed:** a locked step shows its name and who it waits on, but its content is shown only as far as the viewer's access allows (the spec hid nothing). **Changed:** an agent step in a run a person started acts as that person narrowed by the agent's scope, not as the agent alone. |
| A3.6 Writes are real (immediate, real records) | §3 record steps → `action` nodes calling Action Types; §6. | Made precise: writes are all-or-nothing, repeat-safe via an idempotency key per step attempt (WP-0141), obey Business Rules and access exactly as a direct edit, and never edit a posted ledger row. |
| A3.7 Relationship to what exists (workflows, pipelines, approvals inbox, Object 360, agents) | §8; agent steps run a pipeline whose trace attaches to the run; approval steps create inbox entries; Object 360 shows the run. | **Changed (O-2, decided in ADR-0023):** the lane does **not** own the transition. Workflows fire by default; a step declares the actions it `replaces`, and only those are skipped. The workflow stays the fallback for the same change outside a lane. |
| A3.8 Runner screen (stage chevron bar; step list; current step's work; record summary; Back / Complete step / Pause; mobile stacks) | REQ-LANE-014; WP-0707 (product screen), WP-0702 (demo preview). | Unchanged in substance. Notes, attachments and reassign are available at any point (REQ-LANE-018, WP-0711). |
| A3.9 Builder (v1 data-defined; visual builder later) | The schema is the v1 builder: lanes are data. Visual builder is WP-0718; describe-a-process drafting is WP-0715. | Unchanged. |

The spec's A3.10 non-goals stand: no visual builder in v1, no parallel steps in a stage, no
timers/SLAs, no multi-record lanes, no lanes across workspaces. A3.11's feature acceptance
criteria are superseded as the acceptance basis by REQ-LANE-001..025 (the requirements are the
contract; the spec's ACs are the worked example's checks, carried into the example lane's
WP-0702 ACs). A3.12's open items are all settled: O-1 data-defined v1 confirmed; O-2 the
`replaces` rule (§8); O-3 notify is never a step; O-4 sequential plus repeat-over-child (§3);
O-5 run naming settled in §11; O-6 the program home is Phase 7 (ADR-0023).

### 13. What the online demo may and may not do before the editions — AC-3

The demo-first exception (ADR-0023 decision 6, exception to REQ-PRE-021) is granted for WP-0702
only, with two limits: the demo uses the lane definition schema fixed here, and it shows no
behaviour the core will not have. WP-0707 aligns the demo with the product screen.

The demo **may**:

- Use this schema for the Receive Shipment lane definition, and validate it in CI (the check
  added in this package).
- Implement the runner screen (REQ-LANE-014) against the demo store: stage bar, step list,
  current step's work, anchor-record summary, Back / Complete step / Pause.
- Persist runs in localStorage alongside the warehouse seed, mirroring the `ai_runs` /
  `ai_run_nodes` shapes (status names, attempt rows, entered/completed times, run data).
- Perform real writes to the demo store through the same governed path a direct record edit
  uses in the demo (StockMove ledger append-only, `available` computed, `qty_received +
  qty_rejected ≤ qty_ordered`, reservation identity preserved).
- Implement the discrepancy branch and the seeded PO-2026-005 run waiting at manager review;
  enforce one active run per lane per record; enforce the `replaces` rule for `wf-wh-01` and
  `wf-wh-07` (skip exactly those actions, no others).
- Show the run on the anchor record's Object 360 with its audit trail.

The demo **may not**:

- Invent a field, step kind, status, branch, or outcome the schema does not define. In
  particular it may not show `ON_HOLD`, `IN_PROGRESS`, `WAITING_APPROVAL`, `DONE` or
  `CANCELLED` as run statuses — it uses the ADR-0007 statuses plus `waiting_person` (and
  `waiting_child` only for a child lane, which the example does not have).
- Simulate writes. Every step completion that declares a write must write the real demo
  records; a step that writes nothing must declare no writes.
- Build a second engine or its own run tables. Runs are rows in the demo store shaped like
  `ai_runs` / `ai_run_nodes`, not a separate `LaneRun` object.
- Skip schema validation: an invalid example fails the build (the CI check in this package).
- Show behaviour the core will not have: no parallel steps in a stage, no timers or SLA
  escalation, no multi-record lanes, no lanes across workspaces, no visual builder.
- Decide anything the schema leaves to a later package (Action Type ids are provisional until
  WP-0705 adopts them — see §14; the expression grammar is fixed here, its evaluation is
  WP-0710's).

If the demo needs something the schema cannot express, that is a question for the owner via a
`Q-` item in `program/RAID.md` and a schema change in a new work package — not a demo-only
extension.

### 14. Open points and what follows

- **Action Type ids in the example are provisional.** The example names its record-step writes
  (`warehouse.post_stock_move`, `purchase_order.set_line_quantities`,
  `purchase_order.set_status`). The Action Type catalog is WP-0705's; it must adopt these ids or
  update the example in the same pull request. The schema validates the shape of a write, not
  the catalog — the catalog is the core's, and the demo's write functions must match it.
- **Run data at rest.** Run data may hold personal or commercially sensitive values (a carrier
  name, a delivery-note number). It is stored with the run, visible to whoever may see the run,
  and never sent to a model or written to a trace except through the same redaction the run
  trace already applies (WP-0111). WP-0704 records the access rule.
- **The `waiting_person` / `waiting_child` sweep.** The interrupted-run sweep (ADR-0007 §5)
  must not touch runs in these statuses — they are durable waits, not interrupted work. WP-0704
  adds the sweep's exclusion; this ADR only names it.
- **What follows:** WP-0702 (demo lane), WP-0703 (definitions in the core: tables, API,
  compile), WP-0704 (human steps and run data), WP-0705 (record and check steps), WP-0706
  (approvals and branches), WP-0707 (runner screen), WP-0709 (lanes and workflows),
  WP-0710 (repeat and conditions), WP-0714 (dry run, publish checks), WP-0716 (child lanes).
  The first usable version additionally waits for WP-0141 (idempotency keys).

## Alternatives considered

- **A separate lane engine with its own tables and statuses (the spec as written).** Rejected
  in ADR-0023: it would repeat durability, approvals, trace and audit, and collide with the
  locked words Run and Step. This ADR implements the rejection.
- **Stages as executable nodes.** Rejected: a stage that executes is a step by another name.
  Keeping stages as grouping-only means the stage bar, the board and the timing reports all
  read the same grouping, and the engine never has to decide what "running a stage" means.
- **A hold as a waiting status (`waiting_hold` / `ON_HOLD`).** Rejected: a run that sits in a
  hold has no defined resume — that is exactly the dead end the spec's branch had. Ending the
  run with a named outcome keeps every run's end explicit and auditable, and a new run is the
  resume.
- **Workflow actions replaced by default when a lane writes.** Rejected: silent replacement
  hides work from the workflow's owner and from the Dependency Explorer. The `replaces`
  declaration is visible in three places (§8) and replaces only what it names.
- **An arithmetic expression language with functions and comparisons.** Rejected for v1: the
  one arithmetic check the example needs is a sum comparison. A fuller language is a new
  work package when a lane needs it, not a silent extension.
- **Naming the new node `human` instead of `human_step`.** Rejected: `human` reads as a seat
  or a person, and collides with the human-seats vocabulary. `human_step` names what it is — a
  step a person does.

## Consequences

- The graph runtime gains one node type (`human_step`) and one source kind (`lane`); the run
  and step status vocabularies each gain two waiting statuses. `validate_for_publish` learns
  both (WP-0703). `run_state::transition`, `is_waiting_status` and the interrupted-run sweep
  must know the new statuses (WP-0704).
- `ai_runs` gains anchor-record and run-data columns; `ai_run_nodes` gains stage and step
  times (WP-0703/WP-0704, already named in SOLUTION-DESIGN §18).
- The demo and the core share one definition format from the start: a lane definition written
  for the demo imports unchanged into the core (WP-0703 AC-3), and the CI check in this package
  is the first anti-drift test ADR-0015 §5 called for.
- The first usable lane version waits for WP-0141 (idempotency keys) — the only package outside
  Phase 7 it depends on.
- A human seat's work item, Guided Intake and the Path component now have their homes: all
  three are lanes or lane parts (ADR-0023 decision 7), so their packages build on this model
  instead of inventing their own waiting mechanism.
- Owner acceptance: this ADR is **Proposed** until the owner accepts it. The pull request says
  so, and nothing in WP-0702 or later depends on it being Accepted before the demo work starts
  — the demo builds against this schema, and if the owner changes the model, the schema and the
  example change in the same pull request as the acceptance.
