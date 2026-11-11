# Agent profiles

The build uses three AI agents: **Claude Code**, **Codex** and **Muse AI** (ADR-0004, ADR-0021).
All follow [`PROTOCOL.md`](PROTOCOL.md) and [`../AGENTS.md`](../AGENTS.md).

A work package may name the agent it is planned for in `planned_for:` (ADR-0021). That is the
default, not a lock: when the planned agent has no usage left, another agent takes the package over
under "Switching" below. Agents may work in parallel on different work packages; the pushed
`wp/WP-XXXX-*` branch is the claim.

| Agent | Planned work (live list: `STATUS.md`, "Planned workload") |
|---|---|
| Claude Code | The critical path in Phases 1 to 4; channel framework, Slack and Microsoft Teams; the Phase 5 gate; in Phase 6 the two ADRs, the console shell, the page store, forms carry-over and the gate; every package with no `planned_for` |
| Codex | Enterprise sign-in (WP-0102 to WP-0105), provider coverage (WP-0146), toolchain upgrades (WP-0194 to WP-0197), performance baseline (WP-0015); in Phase 5 the agent control plane, delegated credentials and MCP gateway, knowledge, enterprise operations, Python SDK, custom tools, simulated users, red-team suite, optimizer, human seats; in Phase 6 components, custom screens, blocks, builder experience, dashboards carry-over and app navigation |
| Muse AI | Design ADRs (WP-0114, WP-0147), SaaS architecture plan (WP-0422); in Phase 5 the compliance pack, Teams as code and promotion, TypeScript SDK, email channel, web widget, synthetic cases, annotation queues, model right-sizing, human-seat SLAs, Helm chart; in Phase 6 Admin Home, side panels, findability, the website help refresh, form and content components, starter blocks, layouts, templates and themes |

> Muse AI was paused from 2026-11-01 (RAID Q-26) and is building again from 2026-11-11 (ADR-0021).
> Codex joined on 2026-11-11. How Codex works is assumed below until the owner confirms (RAID Q-27).

> If anything below about an agent's capabilities is wrong, correct this file.

## How each agent works

| | Claude Code | Codex | Muse AI |
|---|---|---|---|
| Works through | Its own shell and git in a cloud container, plus GitHub tools | Its own shell and git in a cloud sandbox, plus GitHub (assumed, Q-27) | GitHub only: reads the repo, creates branches, commits and opens PRs through the GitHub web UI and API — no shell, no git CLI |
| Can run tests itself | Yes: `cargo`, `npm`, Playwright, `programctl` | Yes, if its sandbox has network for `cargo` and `npm` (confirm on its first package) | No: CI is its test run — it reads the CI results and the program-bot PR comment (WP-0018) |
| Reads instructions from | `CLAUDE.md` → `AGENTS.md` automatically | `AGENTS.md` automatically | `BRIEFING.md` + the work package file |
| Claims a work package by | Pushing `wp/WP-XXXX-*` (or a claim marker) | Pushing `wp/WP-XXXX-*` | Creating the `wp/WP-XXXX-*` branch and draft PR itself through GitHub |
| Delivers by | Commits on the branch, draft PR | Commits on the branch, draft PR | Commits on the branch, draft PR — written through GitHub; CI regenerates `STATUS.md` on the branch and posts validation, trace and test results as a PR comment |
| Watch out for | The container is temporary: push before the session ends. Debug builds use ~30 GB: `cargo clean` on "No space left on device" | New to this repo: start with WP-0196 or WP-0197; sandbox disk and network limits are not yet known | Cannot run `programctl` or any test; marks criteria "untested locally - see CI" and cites the bot comment; the patch-relay route (`program/inbox/`) is retired for build program work |

Because Muse cannot run tests, **CI (WP-0003) and the program-bot workflows (WP-0018) are what make
Muse-only weeks possible.** They are early in Phase 0 for that reason.

## Switching from one AI to the other

This is the normal way the build moves, not an exception.

**Before the current AI stops** (its limit is near, or the session ends):
1. It pushes everything, even unfinished work. Unfinished is fine; unpushed is lost.
2. It appends a Progress log entry to the WP: what is done, the exact state (green, or red and why),
   the next concrete step with file and line, gotchas, and the commands to run.
3. It leaves `status: in_progress` and writes a session log, with tokens used if visible.

If the limit cuts a session off before step 2, the next AI starts by reading the branch's last
commits and the CI run, and writes the missing handoff entry itself before continuing.

**When the other AI starts:**
1. Run, or ask for, `programctl brief WP-XXXX`. The task card contains the rules, the requirements,
   the acceptance criteria and the last Progress log entry, so the new AI picks up exactly there.
2. Set `owner:` to the new agent's name in the WP file (first commit or first patch).
3. Continue on the **same** branch and pull request. Do not start over, and do not rewrite the
   previous AI's work without a reason written in the Progress log.

## Reviews

Every pull request is reviewed by an agent other than its author when one is available, or by the
owner. When only one AI has usage left, it may review its own earlier work in a fresh session, and
the owner's review becomes mandatory before merge.

## What any agent must never do

- Take a work package someone else has claimed, unless the owner says to (that is a switch, see above).
- Change files outside the work package's scope, or tick a criterion no test proves.
- Merge to `main`, or end a session without a handoff entry.
