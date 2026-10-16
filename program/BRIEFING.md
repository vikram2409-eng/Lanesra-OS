# Briefing for AI agents — Lanesra OS Agent Playground build

Give this file (and the work package you want done) to any AI at the start of a session. It is
self-contained; the files it links to have the detail.

Repository: `github.com/vikram2409-eng/Lanesra-OS` · Owner: Vikram Grover · Builders: Claude Code and Muse AI, used in turn as usage limits allow — neither owns any part of the work

---

## 1. What we are building

**Lanesra OS** is an open-source, self-hosted platform for business apps and governed AI agents
(Rust core + SQLite, Tauri desktop app, Team Workspace server, React UI, plus a single-file online
demo at the repo root). It already has named agents, a checkpointed execution graph, pipelines
(sequential, consensus, peer review), a tool-call firewall with six risk tiers, and approvals.

We are adding the **Agent Playground and Orchestration layer** from the design spec
(`program/spec/agent-playground-design-spec-v1.html`):

- **Playground**: build, chat with, trace, replay, fork and compare agents and teams, side by side.
- **Teams and modes**: Sequential, Consensus, Peer review, Manager-led, Round Table, Handoff,
  Debate and Graph, all compiled onto the one durable graph runtime.
- **Task contracts**: every task declares expected output, schema, guardrails, retry budget, context.
- **Enterprise governance by default**: SSO, Agent Identities, policies, budgets, approvals, audit,
  evaluation gates on publish.

It is enterprise grade: **no shortcuts**. Every acceptance criterion is proven by a test.

It ships in **three editions**: the desktop app, the Team Workspace server, and the online demo
(simulated on sample data). Each work package's `editions:` says where it must ship (most features: all
three), and it is not done until `delivered:` lists all of them.

**Every build updates the website help:** each work package's `docs:` names the
`https://lanesraos.com/help/<slug>` articles it must create or update, and the tracker refuses `done`
until they exist, name the work package and are in the sitemap (`program/FINISH-CHECKLIST.md`).

**Identity first:** humans, Agent Identities and API clients are one principal model, built at the
start of Phase 1 (WP-0100, ADR-0006). Agents get their own identity and Lanesra-issued credentials,
never a human's login or an Administrator's rights.

## 2. Where everything is

| You need | Read |
|---|---|
| The rules you must follow | `AGENTS.md`, then `program/PROTOCOL.md` |
| How you work, and how to take over from the other AI | `program/AGENT-PROFILES.md` |
| Everything on one page | Program dashboard, always current: https://vikram2409-eng.github.io/Lanesra-OS/ (claude.ai copy: https://claude.ai/artifact/TANp9Vzmur4A37bsgHEtKg) |
| What is done, in flight, ready | `program/STATUS.md` (generated) or GitHub Issues labelled `program` (mirror) |
| What to update when you finish | `program/FINISH-CHECKLIST.md` |
| Order of work and why | `program/ROADMAP.md` |
| Your task | `program/work/P<phase>/WP-XXXX-*.md` — Goal, Scope, Acceptance criteria, Test plan, **Progress log** |
| What the spec requires | `program/spec/REQUIREMENTS.md` (stable IDs, e.g. `REQ-LOOP-008`) |
| Decisions already made | `program/decisions/ADR-*.md` (ADR-0004 = the owner's decisions) |
| Risks, open questions | `program/RAID.md` |
| How to test | `program/testing/TEST-STRATEGY.md`, baseline in `program/testing/REGRESSION.md` |

## 3. The plan in one screen

| Phase | Gate to pass | Size | Contents |
|---|---|---|---|
| 0 · Precursors | Engineering ready | 18 WPs · 34 pts | CI on every PR, green `main`, lockfiles, scripted model provider for tests, E2E and migration test harnesses, foundational ADRs, feature flags, threat model, clean-room controls, Issues mirror, Muse relay path |
| 1 · Foundations | Every run produces a full trace | 35 WPs · 114 pts | Identity foundation (principals, Agent Identities, agent credentials), SSO (OIDC + SAML, desktop and server), bring-your-own-model provider coverage and model catalog, run/step model, trace spans and SSE events, step limits and partial results, task contracts, guardrails, prompt catalog, approvals unified, minimal prompt-injection defence, agent settings groups |
| 2 · Playground | Templates pass their evaluation suites | 24 WPs · 90 pts | Three-pane Playground, live trace, step-through, simulate writes, fork/replay, compare, save as test case, publish gate, Guided Setup, three industry templates, online demo Playground |
| 3 · Teams and modes | Security review signed off | 31 WPs · 120 pts | Teams, seats, compiler, run controller, Manager-led / Round Table / Handoff / Debate / Graph modes, team-level roles, budgets, memory recall, team editor and canvas |
| 4 · Enterprise scale | General availability | 19 WPs · 75 pts | Worker pool, hash-chained audit, retention, kill switch, four-eyes, content labels, Missions, shadow mode, A/B, team endpoints |

Points: S=1, M=2, L=4, XL=8. Pace depends on available AI tokens, not calendar weeks.

## 4. How to start a session

```bash
git fetch origin main
python3 program/tools/programctl.py validate
python3 program/tools/programctl.py next            # ready, dependencies done, unclaimed
python3 program/tools/programctl.py show WP-XXXX    # your task, its dependencies and requirements
```

1. Read the WP's **Progress log** — the previous agent's handoff note is the last entry. The previous
   agent may be a different AI; continue on its branch and PR, and set `owner:` to yourself.
2. Claim it: branch `wp/WP-XXXX-short-name` (or a claim marker), status `in_progress`, draft PR `[WP-XXXX] Title`.
3. Write tests from the acceptance criteria first, then implement. Tag tests `// Covers: REQ-… (WP-XXXX AC-n)`.
4. Before you stop, **always**: append a Progress log entry (done / state / next / gotchas /
   commands), tick proven ACs, add a session log (`programctl new-log`), run `programctl status`,
   commit and push. The next agent has no memory of your chat.

## 5. Hard rules

- Never skip, delete, ignore or weaken a test to get green. A red `main` stops the line.
- Clean room: never open CrewAI, Microsoft Agent Framework, LangGraph or AutoGen source; write all
  prompts fresh; use Lanesra words (Team, Playbook, Run, Step).
- One WP per branch and PR; never push to `main`; the other agent reviews; the owner merges.
- Migrations take the next free number above `main`'s maximum at rebase time (ADR-0003).
- Runs, events, errors, migrations, prompts and feature flags follow ADR-0007 to ADR-0012 in `program/decisions/`: use their statuses, event envelope, error classes, table conventions, prompt catalog and flag registry; never invent your own.
- The product is **bring-your-own-model**: organisations connect OpenAI, Gemini, Claude, Azure OpenAI,
  Google Vertex AI, AWS Bedrock, Ollama or on-prem model servers (vLLM) with their own keys, and every feature must work with all of them. The *build team*
  has no API keys, so tests use the scripted provider, recorded runs and contract fixtures - never a live hosted API.
- Spec unclear? Add a `Q-` item to `program/RAID.md` and ask — do not guess.
- Do not hand-edit `program/STATUS.md` or the mirrored GitHub Issues.

## 6. Where we are now (2026-10-16)

- `main` had 7 failing tests; fixed in WP-0016 (root cause: lookup fields' cached value was not
  kept in step with relationship links, so saving a record silently deleted links made by
  workflows). Lockfiles repaired and toolchains pinned in WP-0002.
- Next up (Phase 0, critical path first): **WP-0003** PR CI and branch protection, **WP-0008** gap
  audit → **WP-0009** foundational ADRs, **WP-0005** scripted model provider, **WP-0004** lint
  baseline, **WP-0012** clean-room controls, **WP-0013** online demo architecture,
  **WP-0018** Muse relay path (after CI).
- Live state: `program/STATUS.md`.

## 7. Paste-in starter prompts

**For Claude Code** (it reads `CLAUDE.md` → `AGENTS.md` automatically):

> Work on the Lanesra OS Agent Playground build. Follow AGENTS.md and program/PROTOCOL.md exactly.
> Take work package WP-XXXX (or, if none is named, continue the work package in progress, else the
> first item from `programctl next`). If another AI worked on it before, continue from its last
> Progress log entry on the same branch. Read its Progress log first. Build it to its acceptance criteria with tests, keep
> the full regression suite green, update the Progress log, session log and STATUS.md, push, and
> open a draft PR titled "[WP-XXXX] …". Stop and ask me if the spec is unclear.

**For Muse AI** (it reads the repo itself; name the work package, or let it take the next ready one):

> You are working on the Lanesra OS Agent Playground build in the GitHub repository; Claude Code may
> have worked on this task before you, and may continue after you. Read AGENTS.md, this briefing and
> program/work/…/WP-XXXX-*.md (or the next item under "Ready to start" in program/STATUS.md); follow its acceptance criteria and the hard rules in the
> briefing. You cannot run tests, so make your change as small and reviewable as possible, include
> the tests the work package asks for, work on its `wp/` branch with a draft PR titled "[WP-XXXX] …",
> read the CI results and bot comment on the PR, and write a Progress log entry that says what you changed,
> what is untested locally, and exactly what the next agent (possibly a different AI) should do next. Follow program/FINISH-CHECKLIST.md before
> asking for review; leave STATUS.md to CI.
