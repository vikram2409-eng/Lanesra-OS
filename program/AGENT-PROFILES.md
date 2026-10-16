# Agent profiles

The build uses two AI agents, **Claude Code** and **Muse AI** (ADR-0004). **Neither owns any part of
the work** (ADR-0005): the owner uses whichever one has weekly usage left, and switches to the other
when that runs out — at any point, including in the middle of a work package. Both follow
[`PROTOCOL.md`](PROTOCOL.md) and [`../AGENTS.md`](../AGENTS.md).

What differs between them is only **how** they work, which changes the mechanics of claiming,
testing and delivering — never which work they may take.

> If anything below about an agent's capabilities is wrong, correct this file. Open question for
> the owner: RAID Q-12.

## How each agent works

| | Claude Code | Muse AI |
|---|---|---|
| Works through | Its own shell and git in a cloud container, plus GitHub tools | GitHub only, as far as the repo shows: its changes arrive as scripts or patches applied by a GitHub Action |
| Can run tests itself | Yes: `cargo`, `npm`, Playwright, `programctl` | No: CI tests its changes |
| Reads instructions from | `CLAUDE.md` → `AGENTS.md` automatically | `BRIEFING.md` + the task card the owner pastes in |
| Claims a work package by | Pushing `wp/WP-XXXX-*` (or a claim marker) | Owner pushes the claim, or the Agent apply Action does (WP-0018) |
| Delivers by | Commits on the branch, draft PR | One patch with code, tests and the WP file edits → `program/inbox/WP-XXXX/` → Agent apply Action (WP-0018). Until that exists, the owner commits it on the branch through GitHub |
| Watch out for | The container is temporary: push before the session ends. Debug builds use ~30 GB: `cargo clean` on "No space left on device" | Cannot see test results unless it reads the CI run; marks criteria "untested locally - see CI" |

Because Muse cannot run tests, **CI (WP-0003) and the Agent apply Action (WP-0018) are what make
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
