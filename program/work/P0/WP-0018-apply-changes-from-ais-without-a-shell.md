---
id: WP-0018
title: Apply changes from AIs without a shell (Muse AI relay path)
phase: 0
epic: E0.1 Program and engineering readiness
status: ready
owner: unassigned
agent: claude
depends_on: [WP-0003]
requirements: [REQ-PRE-014]
estimate: M
branch: 
pr: 
updated: 2026-10-16
---

# WP-0018 — Apply changes from AIs without a shell (Muse AI relay path)

## Goal

Muse AI has so far delivered work as Python transform scripts that the owner commits under
`scripts/` and the `apply-ontology-pr.yml` Action runs on a branch. Make that a first-class,
validated path for build program work, so a change from Muse gets exactly the same checks as a
change from Claude Code: it lands on a `wp/` branch, programctl validates it, full CI runs, and a
pull request is opened for cross-review.

## Scope

- A new `agent-apply.yml` workflow (workflow_dispatch: work package id, branch, script or patch path, commit message, agent name)
- Accepts either a `.patch` (git apply --3way) or a Python transform script, from `program/inbox/<WP-id>/`
- Refuses to run unless the branch is `wp/WP-XXXX-*` and the WP exists; never targets `main`
- After applying: `programctl validate`, `programctl status`, commit as `<agent> <noreply>`, push; open or update a draft PR titled `[WP-XXXX] ...`; CI (WP-0003) then runs on the PR
- Removes the inbox file after applying so it cannot be replayed
- Leaves `apply-ontology-pr.yml` untouched for existing website work
- Document the relay steps for Muse in `program/AGENT-PROFILES.md`

## Out of scope

- Giving Muse direct repository credentials (owner decision if wanted later)

## Acceptance criteria

- [ ] AC-1: A sample patch in the inbox is applied to a `wp/` branch, validated, committed and a draft PR is opened
- [ ] AC-2: A dispatch targeting `main` or a non-`wp/` branch is refused
- [ ] AC-3: A patch that breaks `programctl validate` fails the run and pushes nothing
- [ ] AC-4: The workflow runs with least privilege (`contents: write`, `pull-requests: write` only) and never executes code from outside the inbox path

## Test plan

- Dry-run job in CI that applies a fixture patch to a throwaway branch in the runner (no push)
- Manual proof of AC-1 to AC-3 on throwaway branches, run links in the Progress log
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it

## Notes

- Security: workflow_dispatch can only be triggered by people with write access; still validate every input and pin actions by SHA.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
