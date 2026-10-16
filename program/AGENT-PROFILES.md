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

## Handoff between the two

1. The agent finishing a session writes the WP Progress log and a session log (PROTOCOL §6) and pushes.
2. The owner tells the next agent which WP to take (or the agent runs `programctl next`).
3. The next agent reads `BRIEFING.md`, the WP file and its Progress log, then the PR and CI state.
4. A PR authored by one agent is reviewed by the other before the owner merges.

Running both at once is fine if they work different lanes (ROADMAP "Parallel lanes"), for example
Claude Code on the Rust run model while Muse works on design tokens and screens.
