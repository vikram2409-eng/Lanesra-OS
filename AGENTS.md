# Instructions for AI agents

This file is read automatically by Codex, Cursor, GitHub Copilot, Jules and other agents. Claude
Code reads `CLAUDE.md` and Gemini reads `GEMINI.md`; both point here. **Every AI agent follows this file.**

## The Agent Playground build program

The repo is building the Agent Playground and Orchestration layer described in
[`program/spec/agent-playground-design-spec-v1.html`](program/spec/agent-playground-design-spec-v1.html).
Several AI agents (and people) work on it in turn. **The repo is the only shared memory between
them.** Nothing you know from a previous chat counts unless it is written in `program/`.

Before you change anything, in this order:

1. Read [`program/PROTOCOL.md`](program/PROTOCOL.md) — how work is claimed, built, tested and handed over. It is binding.
2. Read [`program/STATUS.md`](program/STATUS.md) — what is done, in flight, blocked and ready.
3. Run `python3 program/tools/programctl.py validate` and `python3 program/tools/programctl.py next`.
4. If you were given a work package (WP) id, run `python3 program/tools/programctl.py show WP-XXXX`
   and read its **Progress log** — it is where the previous agent left off.

If you are asked to do something that is not a work package, say so and ask whether to create one
(see PROTOCOL.md §8). Small fixes outside the program (typos, the website) do not need one.

## Non-negotiable rules

- **No shortcuts.** Enterprise grade: every acceptance criterion is met and proven by a test. If you
  cannot finish, leave it unfinished and say so in the Progress log — never fake, stub or weaken a test to pass.
- **Never skip, ignore, delete or loosen an existing test** to get green. If a test is wrong, fix it in
  a separate commit that explains why, and record it in the work package.
- **Clean room.** Analysts and implementers are separate roles. Implementers never open, fetch or
  search the source code of CrewAI, Microsoft Agent Framework, LangGraph or AutoGen while writing
  Lanesra code. Analysts study those projects only as plain-language behaviour notes in
  `program/research/` (no code, prompts or structure copied). Use only the spec and those notes.
  Write all prompts fresh. Use Lanesra vocabulary (Team, Playbook, Run, Step), never Crew, Kickoff,
  Flow, Superstep, Magentic, StateGraph, Send, Command, Swarm or GraphFlow in code.
- **One work package per branch and pull request.** Do not mix work packages.
- **Migrations:** take the next free number at the time you rebase on `main`, never lower than
  `main`'s maximum; the runner skips any version below the highest applied one.
- **Spec questions are the owner's.** If the spec is ambiguous, add a `Q-` item to `program/RAID.md`
  and ask; do not silently pick a meaning.
- **Do not hand-edit `program/STATUS.md`.** Regenerate it with `programctl status`.
- **Do not push to `main`.** Open a pull request; a different reviewer (human or another AI) reviews it.

## Repository layout

- Repo root: the website and the single-file online demo (`app.js`, no build step). Netlify publishes it.
- `desktop/`: the product — Rust core (`core/`), Team Workspace server (`server/`), CLI (`cli/`),
  Tauri shell (`src-tauri/`), React frontend (`src/`). See `desktop/README.md` and `CONTRIBUTING.md`.
- `program/`: the build program — plan, tracker, decisions, tests strategy, logs. Not published.

## Commands

```bash
cd desktop
cargo test --no-fail-fast -p lanesra-core -p lanesra-server -p lanesra-cli --features lanesra-core/test-support   # Rust suite
npm ci && npx tsc --noEmit && npm test && npm run build                       # frontend
cd .. && node --check app.js                                                   # online demo
python3 program/tools/programctl.py validate && python3 program/tools/programctl.py status --check
```
