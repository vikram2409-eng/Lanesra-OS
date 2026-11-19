# Session log — 2026-11-18 — muse — WP-0710 finish (AC-5 demo + AC-6 docs)

## What this session did

Finished WP-0710 on branch `wp/WP-0710-demo-and-docs` (claimed earlier the same day from main `68bda93`). The engine (AC-1..3) shipped in #789; this session built the demo layer (AC-5), the docs layer (AC-6), and the missing AC-4 proof (a server HTTP test for the per-item submit).

## AC-5 — demo (`app.js`, `styles.css`, Count lane)

- Count lane (`lane-wh-count.json`) now ships with `repeat_over.assign` (`child_field`, `counted_by`) on `count-lines`, `skip_when` (`$item.variance == 0`) on `post-count-adjustments`, a `skip_condition` (`$record.status == COUNTING`) on the Post adjustments stage, and the check-variances PASS branch retargeted to `post-adjustments`. Seed record CC-2026-011 (no variance) added; CC-2026-001 lines gained `counted_by: user-05`.
- Demo literal `LANE_WH_COUNT` embedded + registered in `ensureWarehouseLaneData()`; dashboard "Cycle count in progress" card starts/opens the Count lane.
- Runner: stage gate in `laneEnterStep` (skip/entry conditions evaluated once per stage on first entry, audit wording matches the dry run and the engine's `lane_stage_skipped`); per-item form completion (`laneSubmitFormStepItem`, `__items` state mirroring the engine, step completes only when every non-skipped item is done); `skip_when` honoured in record steps; `cycle_count.set_status` Action Type + `warehouse_count_variance` approval branch; COUNT_ADJUSTMENT stock moves post negative variances; persistent start-refusal card on the record page.
- Self-test extended (Count definition == example file, validates, worked-example shape) — 6/6.
- E2E: new `desktop/e2e/tests/lane-wh-count.spec.ts` (port 4178) — 4/4: per-item completion with assignees, approve-posts-only-nonzero, no-variance stage skip, start refusal. Plus a Count dry-run test in `lane-publish-dryrun.spec.ts`.

## AC-6 — docs

- `/help/process-lanes`: WP-0710 added to `wps`; two new sections ("Try it: the Count lane", "Per-item work, stage conditions and start conditions"); stale "once WP-0710 lands" sentence fixed in "Builder additions".
- Releases v0.99.11 at the head; Roadmap Process Lanes entry updated (description + shipped list + remaining list no longer says per-item writes wait for WP-0710).
- `llms.txt` (lines 23, 26) + `llms-full.txt` (line 23): "the runner honors `assign` and `skip_when` since WP-0710; runtime support for `resolve` is a follow-up".
- Sitemap already lists `/help/process-lanes` — no change.
- Builder surface notes updated (4 spots); the assign note now reads "since WP-0710" to keep the provenance the builder tests assert.

## AC-4 — server HTTP test

- New test `per_item_submit_works_over_http_the_same_as_on_desktop` in `desktop/server/tests/process_lane_http.rs` (Covers: REQ-LANE-017, WP-0710 AC-4): seeds a batch + two tally lines over the same service calls the core test uses, imports + publishes a repeating form lane over HTTP, starts a run over the admin route, submits item 1 (run stays `waiting_person`), submits item 2 (run `completed`), verifies each value landed on its own child record.

## Regression fallout (fixed, not loosened)

- Two `lane-builder-demo.spec.ts` tests failed first: they asserted the pre-WP-0710 Count definition (no skip filter). Updated to the shipped definition — test 238 now asserts the filter is present; test 396 exercises remove (warning returns) → re-add (filter restored) and asserts the export. The builder's "since WP-0710" note was restored after the AC-6 rewrite removed the only `WP-0710` mention the test asserts.

## Local proof (all green, 2026-11-18)

- Core `process_lane_repeat_conditions`: 6/6. Server `process_lane_http`: 2/2. Demo self-test: 6/6. E2E: 45/45 (4 new Count + 41 regression). `programctl validate`: 0 errors, 0 warnings. `node --check app.js`: OK. `cargo fmt`: clean.
- WP-0715 / WP-0719 are already `done` in their files and in STATUS.md — no fold-in needed.
- Playwright note: `npx playwright install chromium` failed twice (CDN download dropping at 70–90%); worked around by downloading the headless-shell zip with `curl --retry` and extracting it to `~/.cache/ms-playwright/chromium_headless_shell-1254/` + `INSTALLATION_COMPLETE` marker. Tests then ran.

## Next

Push, open the draft PR `[WP-0710] …`, CI is the final proof, merge under blanket approval. Then WP-0653 finish.

## CI failure on #796 and the fix (2026-11-18, head d52935f)

- CI: 19 successful, 6 neutral (Netlify deploy canceled), 2 failing — `rust (tests 4/4)` and the aggregate `rust (tests)`. One test failed: `dry_run_count_flags_the_two_zero_variance_lines` (`desktop/core/tests/process_lane_publish.rs`): expected 1 zero-quantity flag, got `[]`.
- Cause: the Count lane's new Post adjustments `skip_condition` (`$record.status == COUNTING`) fired in the CORE dry run on the approved path too. The core dry run never applied simulated `set_status` writes to the simulated record — the approval's `cycle_count.set_status` → APPROVED was recorded as a write, but the anchor status stayed COUNTING, so the stage was skipped and `post-count-adjustments` was never simulated. The online demo's dry run already applies that rule (`app.js` `laneDryRun`: a `.set_status` write moves `simRecord.status`).
- Fix (simulator parity, not a test change): `simulate_record_step` in `desktop/core/src/services/process_lane_publish.rs` now applies a non-repeating step's `.set_status` write to `state.record_fields["status"]` (anchor only; a repeating step's writes act on its children). The WP file Progress log carries the same record.
- Local proof after the fix: the failing test passes; full `process_lane_publish` target 24/24; `cargo fmt` clean; clippy on the core lib clean (re-run at push time if the tree moved).
- Next: push the fix to the r2 branch; CI re-run is the final proof, then mark #796 ready and merge under blanket approval.
