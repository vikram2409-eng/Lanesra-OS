# Agent Playground build program

The control plane for building the Agent Playground and Orchestration layer
([spec](spec/agent-playground-design-spec-v1.html)). Everything an AI agent or person needs to know
about what to build, in what order, what state it is in, how it is tested and how to hand over
lives in this folder and is version-controlled with the code.

**Start here:** [`PROTOCOL.md`](PROTOCOL.md) (the rules) → [`STATUS.md`](STATUS.md) (where we are)
→ [`ROADMAP.md`](ROADMAP.md) (sequence and why).

```
program/
├── PROTOCOL.md          How work is claimed, built, tested, reviewed and handed over (binding)
├── STATUS.md            Generated dashboard: phase progress, active, ready, every WP
├── ROADMAP.md           Phases, gates, critical path, parallel lanes, sequencing rationale
├── RAID.md              Risks, assumptions, issues, dependencies, questions for the owner
├── spec/                The design spec (immutable copy) and REQUIREMENTS.md (stable IDs)
├── work/P0..P4, Pv2/    One file per work package (YAML front matter + body + progress log)
├── gates/               G0..G4 checklists with evidence and sign-off
├── decisions/           Architecture decision records (ADR-XXXX)
├── testing/             Test strategy, regression baseline, performance baseline
├── research/            Clean-room analyst notes and the spec-vs-code gap audit
├── security/            Threat model (from WP-0011)
├── log/YYYY/MM/         One file per working session, any agent
├── templates/           Work package, session log, ADR, gate templates
└── tools/programctl.py  validate · status · next · show · trace · new-log
```

## The tool

```bash
python3 program/tools/programctl.py validate     # every file well-formed; every requirement covered
python3 program/tools/programctl.py status       # regenerate STATUS.md
python3 program/tools/programctl.py next         # ready, dependencies done, unclaimed
python3 program/tools/programctl.py show WP-0121 # one WP with its deps, requirements, dependants
python3 program/tools/programctl.py trace        # requirement → WP → tagged tests
python3 program/tools/programctl.py new-log --agent codex --wp WP-0121
```

CI (`.github/workflows/program.yml`) runs `validate`, `status --check` and the tool's own tests on
every pull request that touches `program/`, `AGENTS.md` or the PR template.

This folder is excluded from the website build (`netlify.toml`), so tracker updates never trigger
a site deploy.
