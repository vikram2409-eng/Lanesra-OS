---
id: WP-0718
title: Visual lane builder
phase: 7
epic: E7.5 Authoring
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0712, WP-0714]
requirements: [REQ-LANE-024]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [process-lanes]
branch: wp/WP-0718-ux-spec
pr: 
updated: 2026-11-14
---

# WP-0718 — Visual lane builder

## Goal

Let admins compose lanes without editing data.

## Scope

- Stages and steps on a canvas, a step panel per kind, branches drawn as connections, live publish checks; produces the same definition format; reuses the graph canvas

## Design

Design spec for this WP, written 2026-11-14 on the ADR-0025 prep track
(Claude's instruction, ferried by the owner). Docs only: this WP is
dependency-blocked on WP-0712 (done) and WP-0714 — and WP-0714's build
is itself blocked on WP-0705 + WP-0706, with the ADR-0025 demo freeze
holding until WP-0707 ships on desktop + Team Workspace. The builder
builds on the WP-0714 publish checks (PR #773, design spec), so the
build starts when they land. Status stays `proposed` until then.

The spec is a direct translation of `program/research/lane-authoring-log.md`:
each lane's "what a builder must show" becomes a concrete surface
below, and each of the log's five hardest things to write by hand gets
the editor surface that solves it. ADR-0025 decision 8's bar for
starting this WP — four warehouse lanes as definitions, at least one
drafted through the assistant, publish checks (WP-0714), and the
authoring log — is met on the evidence side (the four lanes exist, the
WP-0715 drafter drafts one, the WP-0714 checks are designed, the log is
written); the build still waits for the engine.

### 1. Canvas

- **Stages as chevrons, steps as cards.** The stage bar reuses the Path
  component (WP-0621); the canvas reuses the graph canvas from the
  blueprint view (WP-0712) and the Ontology Graph. A stage is one
  chevron; a step is one card inside it, showing kind, name, role, and
  its writes in one line. Stages and steps reorder by drag; reordering
  renumbers nothing — order is definition order, and the builder shows
  the order number on each card.
- **Branches as drawn connections.** A check or approval step's branches
  are arrows from the step card to the target step card, labelled pass /
  fail (check) or approve / reject (approval). An `end` branch is an
  arrow to an end marker labelled with its outcome (`held`,
  `not_triggered`, …). The canvas draws the whole flow, so the author
  sees where each arrow lands instead of holding stage order in their
  head.
- **Same definition format.** The builder produces the lane definition
  as data, in the one JSON Schema shared by the online demo and the core
  (WP-0701). There is no builder-only format. AC-1: a lane built in the
  builder equals the same lane written as data. The proof is a
  round-trip test: data → builder → data is byte-identical modulo key
  order, and the WP-0715 deterministic drafter and the builder produce
  the same format from the same description.
- **Live preview.** The WP-0714 dry run runs against the seed record
  from inside the builder: the author picks the anchor record (for
  example PO-2026-001 with its two lines) and sees the walk — stages
  visited, checks passed / failed, writes the run would post — before
  publishing. This is the Receiving lane's ask from the log: a live
  preview of the run against the seed record, so the author sees the
  quantities the steps will post.

### 2. Condition builder — nested any/all groups

The Pick & Pack damage check is the case it exists for: fully picked
AND (undamaged OR damage-approved), a two-level condition group whose
boolean flag is set by a different step's writes. Written by hand as
raw JSON it is unreadable, and the first version looped forever.

- **Tree editor.** A condition group is a row with an AND / OR switch
  and nested rows; a condition is a row with field / operator / value.
  Rows add, remove and nest by button, not by typing JSON. The group
  structure is visible as indentation, not brackets.
- **References link to the step that sets them.** Each `$run_data`
  reference in a condition is a chip; clicking it jumps to the step
  that sets that field (the approval that sets `damage_approved`, for
  example). A reference to a field no step sets is flagged at once —
  the condition and the flag cannot drift apart. This is the WP-0714
  P-11 check, shown live while editing.
- **Operators from the schema, not free text.** The operator list is
  the schema's (`equals`, `not_equals`, `in_list`, `is_empty`,
  `is_not_empty`, `greater_than`, `less_than`, `on_or_after`,
  `on_or_before`), and the value field offers the record's values and
  the declared run-data fields. The narrow `expression` grammar
  (`$field [+ $field]* (<=|<|>=|>|==|!=) ($field|number)`) is offered
  as a single-line alternative for arithmetic checks, with the same
  field picker.

### 3. Branch-target arrow editor — and the check-last warning

- **Drag an arrow.** From a check or approval step, the author drags
  an arrow to the target step (or to an end marker). The editor writes
  the branch target (`stage` + `step`) — the author never types a
  stage or step id. A target that does not exist cannot be drawn: the
  P-06 check (branch targets resolve) is enforced by construction.
- **Every outcome gets an arrow.** A check shows two arrow slots (pass,
  fail); an approval shows two (approve, reject). An empty slot is a
  visible gap on the canvas and a P-09 failure in the checks panel —
  a branch with no exit cannot be published.
- **Check-last warning.** When a check is not the last step of its
  stage, the editor shows an inline warning on the check card: the
  runner takes the next step in definition order on pass, so if the
  steps after the check are the hold branch's steps, the happy path
  runs them. The warning names the step that would run on pass and
  offers a one-click fix: move the check to the end of the stage. This
  is the WP-0714 P-10 WARN, shown where the author can act on it. The
  dry-run preview (§1) shows the same thing by walking the happy path.

### 4. Per-step reads/writes map — run-data plumbing

The Pick & Pack lane's ask from the log: a map of which run-data
fields each step reads and writes, so the flag the check depends on
and the approval that sets it are visibly connected.

- **On each step card:** what it reads (`binds_to` paths, `$record` /
  `$run_data` / `$item` references in its conditions and write inputs)
  and what it writes (Action Types with their inputs, run-data fields
  it sets). A step that reads a field it does not declare, or writes a
  field no later step reads, is flagged.
- **Lane-level plumbing view:** a table with run-data fields as rows
  and steps as columns — each cell says read, write, or both. The
  author sees at a glance that `damage_approved` is written by the
  damage approval and read by the pick check, and that no field is
  written twice or read before it is written.
- **Refuse an undeclared reference.** A `$run_data` reference to a
  field that is not declared in `lane.run_data.fields` (or a
  `per_child` field for a repeating step) cannot be entered — the field
  picker only offers declared fields, and a pasted reference to an
  undeclared one is rejected with the P-11 reason. This is the
  authoring log's hardest item 3, solved in the editor rather than at
  publish time.

### 5. Approval-writes view

The Count lane's approval sets a status the close step also sets; the
Pick & Pack approval sets a flag that is not a record write at all
(and uses the provisional `run_data.set`).

- **An approval step's panel shows its writes** — each write with its
  Action Type and resolved inputs — **and what happens on each
  branch**: approve → these writes post and the run continues here;
  reject → the run goes here (target step or end outcome), and these
  writes do not post. The author sees the whole decision before
  publishing.
- **A missing write is a visible gap.** An approval with no writes is
  a P-05 failure in the checks panel, not a silent publish refusal.
- **A provisional write is flagged.** A write whose Action Type is
  not in the adopted WP-0705 catalog (for example the provisional
  `run_data.set`) is shown struck-through with the P-12 reason: the
  lane cannot publish until the catalog adopts it. The approval panel
  says so in plain language, not in a schema error.

### 6. Action-Type picker — pending the WP-0705 catalog

- **A picker, not a free-text id.** When the author adds a write, the
  picker offers the adopted Action Types from the WP-0705 catalog,
  each with its inputs — the author picks the type, then fills the
  inputs from the field picker (`$record`, `$run_data`, `$item`,
  literals). This is the authoring log's hardest item 5: every record
  step today invents an Action Type id that the catalog has not
  adopted, and each guess needs revising when the catalog lands.
- **Until the catalog lands**, the picker shows the provisional ids
  used by the four warehouse lanes, each marked provisional with the
  note that it cannot publish (P-12). The author can draft with them;
  the builder never lets a provisional id through to publish.
- **Inputs resolve visibly.** Each input shows the value it will carry
  for the seed record (from the dry-run preview), so the author sees
  `quantity: 40` rather than `quantity: $item.qty_ordered`.

### 7. Live publish checks

- **The WP-0714 checks run live while editing.** P-01..P-17 are
  evaluated on every change; failures list in a checks panel with the
  step and the named reason, and the failing step's card is marked.
  AC-2: every publish check is shown while editing.
- **Publish is disabled until the REFUSE checks pass.** WARNs (P-10
  check-last, P-14 type mismatch) are shown but do not block. The
  publish button states why it is disabled, naming the first failing
  check — the author is never left guessing.
- **The same checks, everywhere.** The builder, the dry run, and the
  publish path evaluate the same check list from WP-0714, so a lane
  that passes in the builder passes at publish. There is no
  builder-only rule and no publish-only rule.

### 8. Per-lane surfaces (from the authoring log)

Each lane asked for one surface beyond the shared editor; the builder
provides all four.

- **Receive Shipment:** the live preview against the seed record
  (§1) — PO-2026-001 with its two lines — so the author sees the
  quantities the steps will post, and the branch editor (§3) for the
  discrepancy branch (check → approval → back to the record stage with
  adjusted quantities, reject → end held).
- **Pick & Pack:** the condition builder (§2) for the damage check, the
  reads/writes map (§4) for the `damage_approved` flag, and a "what
  does this step post?" preview — the StockMoves per line, with the
  delta the run will actually post (the PICK move posts only the
  quantity picked in this run).
- **Count:** a per-line table — expected vs counted vs variance, with
  the variance highlighted — because the whole lane is about those
  three numbers; an approval step that shows the entire count's
  variances at once, not one line at a time; and a warning when a
  repeating record step has no filter (it will post for every child,
  including the ones with nothing to post — the Q-34 case).
- **Replenish:** a way to pick a related record (the supplier) from
  the product, instead of typing an id (the Q-35 lookup); a preview of
  the draft PO before it is created — supplier, product, quantity,
  total — because the manager is approving a real order; and a clear
  "what happens on reject": the draft stays DRAFT, nothing is
  submitted.

### 9. Keyboard, accessibility, and the two editions

- **AC-3:** the builder is keyboard operable — every canvas action
  (select a step, move it, draw an arrow, open a step panel) has a
  keyboard equivalent — and passes axe checks. The canvas is not a
  pointer-only surface.
- **AC-4:** desktop and Team Workspace behave the same from the same
  code; tests run in both modes. The builder is a React surface on the
  shared core service (the same service the Tauri command and the
  server route call), not a desktop-only or server-only feature.
- **AC-5 / AC-6** (demo behaviour, help pages) are written when the
  freeze lifts, against the same behaviour on the warehouse sample
  data. No demo change in this design.

### 10. Out of scope for this WP

- **No runner.** The builder produces definitions; it does not execute
  them. Runs are WP-0707.
- **No publish without the checks.** The builder never publishes a
  lane that fails a REFUSE check, and never invents a check of its
  own — the check list is WP-0714's.
- **No schema change.** The builder edits the existing definition
  format. Schema extensions (per-item assignment, the zero-variance
  filter, a supplier lookup, a Counter role) are a separate proposal:
  `program/spec/schema-extension-proposal-q33-q36.md` (RAID Q-33..Q-36),
  for the owner to accept. When the schema gains a field, the builder
  gains the surface for it — the per-item assignment editor and the
  skip filter live in the Count lane's surfaces (§8) once Q-33/Q-34
  are accepted.

- [ ] AC-1: A lane built in the builder equals the same lane written as data
- [ ] AC-2: Every publish check is shown while editing
- [ ] AC-3: The builder is keyboard operable and passes axe checks
- [ ] AC-4: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-5: Online demo (`app.js`) shows the same behaviour on the warehouse sample data; demo E2E covers it
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains what the feature does, how to set it up step by step and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest; Playwright E2E
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-14 muse: UX spec written as a Design section (ADR-0025 prep track, Claude's instruction via the owner). Canvas (stages as chevrons, steps as cards, branches as drawn arrows, reusing the WP-0712 graph canvas), nested any/all condition builder with references linked to the step that sets them, branch-target arrow editor with the check-last warning and one-click fix, per-step reads/writes map and lane-level plumbing view, approval-writes view, Action-Type picker (pending the WP-0705 catalog), live WP-0714 publish checks, per-lane surfaces for all four warehouse lanes, keyboard + axe. Docs only — no core, no app.js, no schema change. WP stays proposed and unassigned: the build is blocked on WP-0714 (whose build is blocked on WP-0705 + WP-0706) and the ADR-0025 demo freeze. ACs untouched.
