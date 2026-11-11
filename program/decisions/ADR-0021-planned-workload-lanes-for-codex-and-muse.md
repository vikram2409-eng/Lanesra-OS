# ADR-0021: Planned workload lanes for Codex and Muse AI

- Status: Accepted (program owner instruction, 2026-11-11: "plan certain workload for Codex and certain for Muse")
- Date: 2026-11-11
- Amends: ADR-0005 decisions 1, 2 and 6; supersedes the answer to RAID Q-26

## Context

ADR-0005 removed all pre-assignment because the owner alternated two AIs as usage limits ran out.
On 2026-11-01 Muse AI stopped building and Claude Code took everything (RAID Q-26). On 2026-11-11
the owner added Codex as a builder, brought Muse back, and asked for a planned split of work.
ADR-0020 adds lanes that touch different code areas from the critical path, so agents can work in
parallel with few conflicts.

## Decision

1. **Three builders:** Claude Code, Codex and Muse AI. All follow `PROTOCOL.md`.
2. **A work package may carry `planned_for: claude | codex | muse`.** It names the default agent.
   It is a plan, not a lock: if the planned agent has no usage left, any agent may take the package
   and continues under the normal switching rules. `owner:` still records who actually holds it.
   The old `agent:` field stays rejected.
3. **Agents may work in parallel** on different work packages (this replaces ADR-0005 decision 6).
   The pushed `wp/WP-XXXX-*` branch is the claim again (PROTOCOL §3), and is required.
4. **How the split was made:**
   - **Codex** has a shell and runs the test suite. It takes backend-heavy lanes that are separate
     from the critical path: enterprise sign-in (WP-0102 to WP-0105), provider coverage (WP-0146),
     toolchain upgrades (WP-0194 to WP-0197), the performance baseline (WP-0015), and in Phase 5
     the agent control plane, delegated credentials and MCP gateway, knowledge, enterprise
     operations, the Python SDK, custom tools, simulated users, the red-team suite, the optimizer
     and human seats.
   - **Muse AI** works through GitHub with CI as its test run. It takes design and
     content-heavy work, and code with a narrow blast radius that CI covers fully: the two design
     ADRs (WP-0114, WP-0147), the SaaS architecture plan (WP-0422), and in Phase 5 the compliance
     pack, Teams as code and promotion, the TypeScript SDK, the email channel and web widget,
     synthetic cases, annotation queues, model right-sizing, human-seat SLAs and the Helm chart.
   - **Claude Code** keeps the critical path (Phases 1 to 4 not listed above), the channel
     framework with Slack and Microsoft Teams, and the Phase 5 gate.
5. **Review:** a pull request is reviewed by a different agent from its author when one has usage
   left, otherwise by the owner. Muse's pull requests that change code always get a review from an
   agent that can run the tests.

## Planned split at the time of this decision

| Agent | Open work packages | Points |
|---|---|---|
| Codex | 34 (10 in Phases 0–1, 24 in Phase 5) | 120 |
| Muse AI | 16 (2 in Phase 1, 1 in Phase 4, 13 in Phase 5) | 40 |
| Claude Code | 5 in Phase 5, plus every package with no `planned_for` | 16 + the rest |

Live numbers are in `STATUS.md` under "Planned workload".

## Consequences

- `programctl validate` accepts `planned_for`; `status` and the dashboard show the split.
- Codex is new to the repo: its first package should be a small one (WP-0196 or WP-0197) to prove
  its setup before it takes sign-in work that later packages depend on.
- Enterprise sign-in (WP-0102 to WP-0104) feeds the Playground through WP-0201. If Codex stalls
  there, Claude Code takes it over; the owner decides.
- Open question for the owner: RAID Q-27 (how Codex works: shell, network, who merges).
