---
id: WP-0018
title: Tracker upkeep and test feedback for AIs without a shell
phase: 0
epic: E0.1 Program and engineering readiness
status: ready
owner: unassigned
depends_on: [WP-0003]
requirements: [REQ-PRE-014]
estimate: S
editions: []
delivered: []
branch: 
pr: 
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

- [ ] AC-1: A PR that changes a work package file without regenerating STATUS.md gets a bot commit that fixes it, and the tracker check then passes
- [ ] AC-2: The PR comment lists validation, trace and per-job CI results, and updates in place on each push
- [ ] AC-3: The workflow never runs for forks and never pushes to `main`
- [ ] AC-4: AGENT-PROFILES.md and BRIEFING.md describe Muse's GitHub-only flow end to end

## Test plan

- Throwaway PR proving AC-1 to AC-3 (run links in the Progress log)
- Unit test for the comment renderer in `program/tools/`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
