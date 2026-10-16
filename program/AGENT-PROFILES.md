# Agent profiles

The build uses two AI agents (ADR-0004): **Claude Code** and **Muse AI**. They have different
capabilities, so they take different roles and hand work over through the repository. Both follow
[`PROTOCOL.md`](PROTOCOL.md) and [`../AGENTS.md`](../AGENTS.md).

> If anything below about an agent's capabilities is wrong, correct this file — it decides who is
> given which work. Open question for the owner: RAID Q-12.

## Claude Code

| | |
|---|---|
| Works through | Its own shell and git in a cloud container; GitHub tools for PRs, issues and reviews |
| Can run | `cargo`, `npm`, Playwright (Chromium pre-installed), `programctl` — the whole test suite |
| Instruction file | `CLAUDE.md` → `AGENTS.md` |
| Best suited for | Rust core and server work, migrations, test infrastructure, CI, debugging failing tests, security-sensitive work, reviewing Muse's PRs, running gates |
| Branches | Uses `wp/WP-XXXX-*` where its tooling allows; otherwise pushes the claim marker `wp/WP-XXXX-claim-claude` (PROTOCOL §3) |
| Limits | A session's container is temporary: everything must be committed and pushed before it ends. Large builds can fill its disk — run `cargo clean` if "No space left on device" appears |

## Muse AI

| | |
|---|---|
| Works through | GitHub only, as far as the repo history shows: its changes arrive as transform scripts run by a GitHub Action, committed as `Muse (Ontology Graph build)`. The owner relays scripts into `scripts/` |
| Can run | Nothing locally; **CI is where its changes are tested** |
| Instruction file | Give Muse [`BRIEFING.md`](BRIEFING.md) and the work package file at the start of each session |
| Best suited for | Frontend screens and the online demo (its past work: UI foundation, graph UX, website), UX copy, documentation, analyst research notes, reviewing Claude Code's PRs |
| How a change lands | Until WP-0018 exists: the current `apply-ontology-pr.yml` route onto a `wp/` branch, then a PR. After WP-0018: put the patch or script in `program/inbox/WP-XXXX/` and run the **Agent apply** Action, which validates, commits and opens the draft PR |
| Limits | Cannot see test output unless it reads the CI run; its Progress log entries must say "untested locally — see CI" |

## Who does what

| Kind of work | Primary | Reviewer |
|---|---|---|
| Rust core, server, CLI, migrations | Claude Code | Muse (design/readability) or owner |
| CI, test harnesses, tooling | Claude Code | Owner |
| Frontend (desktop UI) and online demo | Muse AI | Claude Code (runs the tests, checks a11y) |
| ADRs and research notes | Either | The other agent |
| Gate reviews | Claude Code prepares evidence | Owner signs |

## How the two agents are orchestrated

Roles: **owner** = dispatcher and merger · **Claude Code** = builder for core/server/CI and the
integrator (applies and tests Muse's changes, keeps the tracker consistent, prepares gates) ·
**Muse AI** = builder for UI and the online demo, and reviewer of Claude Code's PRs.

### One round

1. **Pick.** Open `program/STATUS.md` (or the GitHub Issues board, milestone = phase). The sections
   **Next for Claude Code** and **Next for Muse AI** list ready work whose dependencies are done,
   filtered by each WP's `agent` field (`claude`, `muse`, `either`).
2. **Claude Code** — tell it "take the next Claude Code work package". It claims (pushes `wp/` branch,
   status `in_progress`), builds with tests, opens a draft PR, writes the Progress log.
3. **Muse AI** — ask Claude Code: "claim WP-XXXX for Muse and give me its task card". Claude Code
   pushes the claim (`owner: muse`, so nobody else takes it) and prints `programctl brief WP-XXXX`.
   Paste `BRIEFING.md` plus the task card into Muse.
4. **Muse delivers** a patch or transform script that includes its WP file edits (status, ticked
   criteria marked "untested locally - see CI", Progress log entry) and a session log.
5. **Land Muse's change.** Until WP-0018 exists: hand the patch to Claude Code, which applies it to
   the claimed branch, runs the full suite and opens the PR. After WP-0018: drop it in
   `program/inbox/WP-XXXX/` and run the *Agent apply* Action.
6. **Feedback loop.** If tests fail, Claude Code writes the failure and the fix it expects into the
   WP's Progress log and gives you a fresh task card - the card always carries the latest log
   entry, so Muse sees exactly what broke. Repeat 4–6 until green.
7. **Cross-review.** Claude Code reviews Muse's PRs (tests, accessibility, spec fit). Muse reviews
   Claude Code's UI-facing PRs from the diff (UX, copy, consistency). Findings go on the PR.
8. **Merge.** The owner merges. The WP becomes `done`; STATUS.md and the Issues mirror update.

Both agents can run at the same time on different lanes (ROADMAP "Parallel lanes").

### How Muse knows what to pick and what to update

- **What to pick:** the first item under *Next for Muse AI* in `STATUS.md` that is not claimed —
  or simply the WP whose task card you hand it. Muse never needs to search the repo.
- **What to update:** the task card's *What you must deliver* section lists every file and line:
  the code and tests, the WP front matter, ticked criteria, the Progress log entry, a session log.
  Muse never edits `STATUS.md` or GitHub Issues (they regenerate).
- **What happened before:** the card includes the previous agent's last Progress log entry.

### What each agent must never do

- Claude Code: take a WP claimed for Muse, or rewrite Muse's work without a review comment first.
- Muse AI: change files outside its WP's scope, or mark a criterion done that no test proves.
- Either: merge to `main`, or skip the handoff note.
