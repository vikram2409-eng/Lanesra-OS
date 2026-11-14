# Session 2026-11-14 · claude · planning (no work package)

## Goal of this session

Owner: check pull request #762 (Muse), plan next steps from the recommendations of 2026-11-14,
with Codex not building; plan the next packages for Claude Code and independent ones for Muse AI.

## What was done
- Reviewed #762 (draft; demo only: lane fixes and a Pick and Pack lane). One failing check:
  `program`, test `test_demo_reads_the_shared_example` (WP-0712), because the demo now embeds the
  lane definition instead of fetching the shared example.
- ADR-0025: runner split (WP-0707 first cut, new WP-0720 second cut); engine work planned for
  Claude Code in sequence; demo freeze after #762; `run_data.set` not an Action Type in the core
  (RAID Q-32); per-item assignment in WP-0710; order of the larger bets and the bar for the builder.
- WP-0704 and WP-0141 refined and `ready` (Claude). WP-0715 `ready` (already in progress on a Muse
  branch). New WP-0719 (Count and Replenish as definitions, authoring log; Muse) and WP-0720.
- Re-planned Codex's Phase 7 packages and WP-0141.

## State at end
- `programctl validate`, `status --check`, `docs --check` and the tool tests pass. No product code changed.

## Next step for whoever continues
- Claude Code: WP-0704 and WP-0141, then WP-0705, WP-0707.
- Muse AI: fix and merge #762; finish WP-0715; close out WP-0702 and WP-0712; WP-0719; then WP-0147, WP-0114.
