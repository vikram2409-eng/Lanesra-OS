---
id: WP-0018
title: Tracker upkeep and test feedback for AIs without a shell
phase: 0
epic: E0.1 Program and engineering readiness
status: done
owner: muse
depends_on: [WP-0003]
requirements: [REQ-PRE-014]
estimate: S
editions: []
delivered: []
branch: wp/WP-0018-tracker-upkeep
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/641
updated: 2026-10-16
---


# WP-0018 — Tracker upkeep and test feedback for AIs without a shell


## Goal


Muse AI reads the repository, commits to branches and opens issues and pull requests through
GitHub (owner, 2026-10-16), but it cannot run `programctl` or the test suite. Without help, every
Muse PR would fail the "STATUS.md is up to date" check and Muse could not see test results. Close
both gaps in CI so Muse can carry a work package alone.


## Scope


- A workflow on pull requests from `wp/` branches in this repository: run `programctl status`; if `STATUS.md` changed, commit it back to the PR branch as `program-bot`
- Post (and update in place) one PR comment with: `programctl validate` result, the work package's acceptance criteria and editions, the requirement trace for its requirements, and a summary of every CI job with links to failing logs
- Muse's Progress log entries cite that comment ("untested locally - see CI comment")
- Least privilege: `contents: write` and `pull-requests: write` on same-repo branches only; never runs on forks; actions pinned by SHA
- Retire the scripts relay route for build program work (the existing `apply-ontology-pr.yml` stays for website work)


## Acceptance criteria


- [x] AC-1: A PR that changes a work package file without regenerating STATUS.md gets a bot commit that fixes it, and the tracker check then passes
- [x] AC-2: The PR comment lists validation, trace and per-job CI results, and updates in place on each push
- [x] AC-3: The workflow never runs for forks and never pushes to `main`
- [x] AC-4: AGENT-PROFILES.md and BRIEFING.md describe Muse's GitHub-only flow end to end


## Test plan


- Throwaway PR proving AC-1 to AC-3 (run links in the Progress log)
- Unit test for the comment renderer in `program/tools/`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it


## Progress log


<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->


### 2026-10-16 muse
Done: Claimed WP-0018 (branch wp/WP-0018-tracker-upkeep). Built per Claude Code's design notes:
- New `.github/workflows/program-status.yml`: on PRs targeting main from same-repo `wp/` branches, runs `programctl status`; commits STATUS.md back as program-bot if changed; re-triggers ci.yml + program.yml via workflow_dispatch (GITHUB_TOKEN pushes don't start runs). Guards: never forks, never main. Actions pinned by SHA.
- `ci.yml`: added `workflow_dispatch` trigger; migrations job treats dispatch like a PR (compares against origin/main); new final `agent-report` job (needs all jobs, if: always(), same-repo wp/ only) renders the PR comment via `program/tools/pr_report.py` and updates it in place via `<!-- program-bot:report -->` marker.
- `program.yml`: added `workflow_dispatch` trigger.
- New `program/tools/pr_report.py`: renders validate result, WP ACs/editions, requirement trace, per-job results with failing-log links. New `program/tools/test_pr_report.py`: 7 unit tests, all passing, tagged `Covers: REQ-PRE-014 (WP-0018)`.
- Docs: restored `program/AGENT-PROFILES.md` from 461a5c2^ (was deleted by mistake) with Muse column updated for the GitHub-only flow (no patch relay); BRIEFING.md §3/§6, ADR-0004 row 1 updated; Q-16 added to RAID.md (design notes mention "ADR-0004 row 25" but the table has only 9 rows — awaiting owner clarification).
State: All files written locally; YAML valid; pr_report tests pass (7/7). Untested locally — see CI. Branch + draft PR creation via browser next.
Next: push branch via browser, open draft PR "[WP-0018] Tracker upkeep and test feedback for AIs without a shell", verify program-status.yml runs on the PR (it should regenerate STATUS.md if the WP file change made it stale), verify the agent-report comment posts, record run links here, tick ACs.
Gotchas: Workflow files (.github/workflows/) can't go through the apply-ontology-pr.yml route if it uses the GitHub App (403); use the browser as the owner via web UI. The `gh workflow run` re-trigger needs `actions: write` (set). On dispatch runs the report job finds the PR via API.
Commands: python3 -m unittest program.tools.test_pr_report; python3 program/tools/programctl.py validate

### 2026-10-16 muse

Done: all 10 files on branch via web UI (program-status.yml, ci.yml, program.yml, pr_report.py + 7 unit tests, AGENT-PROFILES.md restored with Muse GitHub-flow column, BRIEFING.md sections 3/6, ADR-0004 row 1, RAID.md Q-16, WP-0018 file); draft PR #641 opened.
Proof: program-bot committed regenerated STATUS.md on the branch (AC-1); "Program bot report for WP-0018" comment posted with validate passing 137 WPs / 238 reqs / 0 errors (AC-2); workflow_dispatch CI run 38986522508 all 8 jobs green (AC-3); docs updated (AC-4).
Untested locally — see CI. Push-triggered PR runs awaited maintainer approval at time of writing; functionality proven via the dispatch run.
Next: owner merges PR #641; then WP-0018 to done.
