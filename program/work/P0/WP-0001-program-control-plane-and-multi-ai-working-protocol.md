---
id: WP-0001
title: Program control plane and multi-AI working protocol
phase: 0
epic: E0.1 Program and engineering readiness
status: in_progress
owner: claude
depends_on: []
requirements: [REQ-PRE-015]
estimate: M
editions: []
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: 
updated: 2026-10-16
---

# WP-0001 — Program control plane and multi-AI working protocol

## Goal

Give every AI agent and human one place in the repo that says what to build, in what order, what is in flight, how it is tested and how to hand over - so work moves between AI tools without loss.

## Scope

- `AGENTS.md` (universal entry point) plus `CLAUDE.md` and `GEMINI.md` pointers
- `program/`: protocol, roadmap, requirements register, work packages, gates, decisions, RAID log, test strategy, session logs, templates
- `program/tools/programctl.py` (validate, status, next, show, trace, new-log) with self-tests
- `.github/workflows/program.yml` validating the tracker on every pull request
- Pull request template with work package, requirement and clean-room fields
- Netlify build skip for tracker-only changes

## Acceptance criteria

- [x] AC-1: Every spec commitment has a requirement ID and maps to at least one work package (`programctl validate` passes)
- [x] AC-2: `programctl status` generates STATUS.md and `status --check` detects a stale file
- [x] AC-3: `programctl next` lists ready, unclaimed work packages using remote `wp/*` branches as claims
- [x] AC-4: programctl self-tests pass (`python3 -m unittest discover program/tools`)
- [x] AC-5: Protocol covers session start, claim, build, test, handoff, tracker updates, conflicts and escalation
- [ ] AC-6: Program owner has reviewed the plan and answered the open questions in RAID log Q-items

## Test plan

- `program/tools/test_programctl.py`: parsing, validation errors (missing fields, bad deps, cycles, uncovered requirements, done-with-unticked-AC), status rendering, ready-to-start logic
- CI: `program.yml` runs validate, status --check and the self-tests
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

### 2026-10-16 claude
Done: AC-1..AC-5. Program folder, 221 requirements, 121 work packages, protocol, roadmap, RAID,
gates, ADR-0001..0003, programctl (13 self-tests), program.yml CI, PR template, Netlify exclusion.
State: validate green; no product code changed.
Next: AC-6 needs the owner's review and answers to RAID Q-01..Q-11. On merge set status done
(pr link) and mark ADR-0001..0003 Accepted.
Gotchas: `cargo test` rewrites desktop/Cargo.lock (stale version) - do not commit that in
unrelated WPs; WP-0002 fixes it. Use `--no-fail-fast` or failures hide later test binaries.
