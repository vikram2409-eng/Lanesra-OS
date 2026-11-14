# Session log — ADR-0025 prep track (WP-0714 design, WP-0718 UX, Q-33..Q-36 proposal)

2026-11-14 ~10:54–11:30 EST. Agent: Muse. Instruction: Claude's message,
ferried verbatim by the owner ("Message for Muse — paste as-is"). Claude's
batch (#768, #767, #772, #770, #771) had all merged; he is on WP-0705 now
(PR #775, open draft). The remaining Phase 7 authoring WPs are
dependency-blocked on his engine sequence and the ADR-0025 demo freeze,
so this session was pure design/data prep — three pieces, one branch/PR
each, no core, no app.js, no migration, no demo change, no schema change.

## Piece 1 — WP-0714 design spec (PR #773, draft)

Branch `wp/WP-0714-design-spec` (worktree `lanesra-os-wp-0714`). A Design
section on WP-0714; the WP stays `proposed` and unassigned, ACs untouched.

- Publish checks P-01..P-17, each with a named refusal reason. P-01..P-08
  already enforced by `validate_definition` / `validate_compiled` /
  `publish_lane` (this WP surfaces them as named publish failures);
  P-09..P-15 new: branch exits, check-last-in-stage as a WARN, no
  undeclared `$run_data` / `$record` / `$item` references, no provisional
  Action Types in a publishable lane, replaced workflows exist, run-data
  types match their uses, start-condition references. Consequence stated
  in the spec: the four warehouse lanes cannot publish until WP-0705
  adopts their Action Types.
- Dry run: walks the lane against a seed record with author-supplied
  fixtures, simulates writes without posting anything, asserts the walk
  reaches an end, every `$` reference resolves, every check evaluates
  (never `unknown`), no zero-quantity write, and the end state matches
  the author's expected outcome.
- Test framework: lane cases in the Unified Test Run, with the assertion
  format (`run.ends_with`, `step.<id>.completed/skipped`,
  `check.<id>.passed/failed`, `write.posted` / `write.not_posted`,
  `record.equals`, `run_data.equals`). Cases live with the lane and
  travel in its Solution export.
- Solution export shape: all versions export (not just published), cases
  export with the version they test, dependencies declared; import
  creates a new draft version and runs the publish checks in report mode.
- Grounding: each of the five hardest items from
  `program/research/lane-authoring-log.md` mapped to the check, dry-run
  assertion or builder surface that covers it.
- Folded in (small stuff, owner's minimize-PRs rule): the tracker
  closeout for the two merged WPs — WP-0715 done (PR #772, ad631ef,
  delivered [desktop, workspace, demo]) and WP-0719 done (PR #767,
  2b3c5ab, delivered [] — definitions only).
- Verify: `programctl validate` 0 errors. PR #773 open draft,
  mergeable `blocked` (normal for a draft), 4 files.

## Piece 2 — WP-0718 visual-builder UX spec (PR #774, draft)

Branch `wp/WP-0718-ux-spec` (worktree `lanesra-os-wp-0718`). A Design
section on WP-0718; the WP stays `proposed` and unassigned, ACs untouched.

- Canvas (stages as chevrons via the Path component, steps as cards,
  branches as drawn arrows, reusing the WP-0712 graph canvas; same
  definition format, round-trip test for AC-1; live dry-run preview
  against the seed record).
- Nested any/all condition builder (tree editor; each `$run_data`
  reference links to the step that sets it).
- Branch-target arrow editor (drag an arrow; empty outcome slots are
  visible P-09 gaps; check-last warning inline with a one-click
  "move to end of stage" fix, P-10).
- Per-step reads/writes map + lane-level plumbing view (P-11 shown
  live; undeclared references cannot be entered).
- Approval-writes view (writes + approve vs reject; missing write is a
  visible P-05 gap; provisional writes struck through, P-12).
- Action-Type picker (the adopted WP-0705 catalog; provisional ids
  marked provisional until adopted).
- Live WP-0714 publish checks while editing; publish disabled until the
  REFUSE checks pass.
- Per-lane surfaces for all four warehouse lanes, from the log's "what a
  builder must show". Keyboard + axe (AC-3); same behaviour on desktop
  and Team Workspace (AC-4).
- Verify: `programctl validate` 0 errors. PR #774 open draft,
  mergeable `blocked`, 1 file.

## Piece 3 — Schema-extension proposal, RAID Q-33..Q-36 (PR #776, draft)

Branch `wp/schema-extension-proposal-q33-q36` (worktree
`lanesra-os-q33-q36`). New doc
`program/spec/schema-extension-proposal-q33-q36.md` + a Notes line and
Progress entry on WP-0710 (which stays `proposed` and unassigned —
planned_for claude). No schema change: the schema file is untouched,
no lane definition edited. Each item is accepted separately by the owner.

- Q-33: `repeat_over.assign` — per-item assignment by child field
  (`by: "child_field"`, `field`), step's role as fallback. WP-0710
  implements on acceptance.
- Q-34: `skip_when` on a repeating step — same condition grammar;
  matching children post nothing, recorded as skipped. WP-0710
  implements on acceptance. (The Action-Type-skips-it alternative was
  rejected: the decision belongs in the definition.)
- Q-35: `resolve` on a form field — `latest_record` lookup (the
  supplier on the product's most recent PO). Deliberately narrow: one
  kind, one field, no joins. WP-0705 implements on acceptance.
- Q-36: add `role_wh_counter` — a role-model / seed change, not a
  schema change.
- Each RAID item closes when accepted **and** implemented; acceptance
  alone does not close it. Until then the four lanes keep their current
  defaults.
- Verify: `programctl validate` 0 errors. PR #776 open draft,
  mergeable `blocked`, 2 files.

## Notes and gotchas

- The repo AGENTS.md says one WP per branch/PR. The owner outranks it:
  his minimize-PRs rule is why the WP-0715/WP-0719 closeout rides in
  #773 instead of its own PR, and why the three pieces are three PRs
  (not six). The deviation is stated in the #773 PR body.
- Claude's WP-0705 PR (#775) opened while this session ran; the
  numbering confirms it (#773, #774, #775, #776 in order).
- CI on all three PRs is the final proof (Rust suite, fmt/clippy,
  Tauri compile, e2e — none runnable locally). All three are drafts;
  the owner merges.
- The WP-0714 branch was cut from origin/main ad631ef; the three
  branches are independent (no branch builds on another), so they
  merge in any order.
