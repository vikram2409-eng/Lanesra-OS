---
id: WP-0017
title: Mirror work packages to GitHub Issues
phase: 0
epic: E0.1 Program and engineering readiness
status: in_review
owner: claude
depends_on: [WP-0001]
requirements: [REQ-PRE-013]
estimate: S
editions: []
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/492
updated: 2026-10-16
---

# WP-0017 — Mirror work packages to GitHub Issues

## Goal

The owner asked for a GitHub Issues mirror (ADR-0004). Give people a browsable board of the plan in
GitHub while the work package files stay the single source of truth for every AI.

## Scope

- `program/tools/issues_mirror.py`: one issue per WP, labels `program`, `phase:N`, `status:*`, `est:*`, a milestone per phase, body regenerated from the file, closed when done or dropped
- `.github/workflows/program-issues.yml`: runs on push to main touching `program/work/`, and on demand
- Optional later: a GitHub Projects board fed from the same labels (needs the owner to create the project; `GITHUB_TOKEN` cannot write user-owned projects)

## Acceptance criteria

- [x] AC-1: Dry run lists one create per work package with labels, milestone and state
- [x] AC-2: Unchanged issues produce no update; changed ones (state, body, labels) do; dependency links resolve to issue numbers
- [ ] AC-3: First run on main creates all issues and milestones (verify after merge; link the Actions run here)
- [ ] AC-4: A second run with no changes performs zero updates (idempotent)

## Test plan

- `program/tools/test_issues_mirror.py` (no network): AC-1, AC-2 and closed-state mapping
- AC-3 and AC-4 are verified on the first two runs on main
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

### 2026-10-16 claude
PR opened: https://github.com/vikram2409-eng/Lanesra-OS/pull/492 (shared planning PR, owner-requested). Status in_review. After merge: status done, `delivered:` filled for its editions.
