# Build protocol

How every AI agent and person works on the Agent Playground build. It exists so that any agent can
stop at any point and a different agent (another vendor, another day, no shared chat history) can
continue without loss. **If it is not in the repo, it did not happen.**

Contents: 1 Sources of truth · 2 Session start · 3 Claiming · 4 Building · 5 Testing · 6 Session end
and handoff · 7 Review and merge · 8 Changing the plan · 9 Gates · 10 What to update, when ·
11 Conflicts · 12 Escalation · 13 Conventions

---

## 1. Sources of truth

| Question | Where the answer lives | Who changes it |
|---|---|---|
| What must the product do? | `spec/agent-playground-design-spec-v1.html` (immutable) and `spec/REQUIREMENTS.md` (IDs) | Owner, via ADR |
| In what order? Why? | `ROADMAP.md`, `depends_on` in each work package | Planner, via PR |
| What exactly is one unit of work? | `work/P<phase>/WP-XXXX-*.md` | Agent doing it (progress); planner (scope) |
| What is done / in flight / ready? | `STATUS.md` — **generated**, never hand-edited | `programctl status` |
| Who is working on what right now? | Remote branches `wp/WP-XXXX-*` and draft PRs titled `[WP-XXXX]` | Agent claiming |
| Why was something designed this way? | `decisions/ADR-XXXX-*.md` | Anyone proposing, owner accepts |
| Risks, assumptions, open issues, questions for the owner | `RAID.md` | Anyone |
| How we test; the regression baseline | `testing/TEST-STRATEGY.md`, `testing/REGRESSION.md` | Anyone, via PR |
| Is a phase finished? | `gates/G<n>-*.md` with evidence | Agent prepares, owner signs |
| What happened in a session | `log/YYYY/MM/*.md` (one file per session) | The agent that ran it |

Precedence when sources disagree: spec (as amended by accepted ADRs) > REQUIREMENTS.md > work
package > anything else. Report a disagreement as a RAID issue; do not resolve it silently.

## 2. Session start (every session, every agent)

```bash
git fetch origin main && git status
python3 program/tools/programctl.py validate
python3 program/tools/programctl.py next          # ready + dependencies done + not claimed
```

1. Read `AGENTS.md`, this file and `STATUS.md`.
2. **Resuming a work package?** Check out its branch, run `programctl show WP-XXXX`, read its
   **Progress log** (newest last), the latest session log for it in `log/`, and every unresolved
   review comment on its PR. Run the test suite before changing anything, so you know the state you inherited.
3. **Starting fresh?** Pick from `programctl next` (or the WP the owner assigned). Read the WP's
   requirements in `spec/REQUIREMENTS.md`, the spec sections they come from, and every ADR the WP cites.
4. If the WP is not `ready` (see Definition of Ready, §4), do not start coding; refine it first (§8).

## 3. Claiming a work package

A claim must be visible to every other agent immediately, without waiting for a merge.

1. Create the branch `wp/WP-XXXX-short-name` from the latest `main` and push it. **The pushed
   branch is the claim** (`programctl next` hides claimed WPs by reading remote `wp/*` branches).
   If your tool forces its own branch name, also push a claim marker:
   `git push origin HEAD:refs/heads/wp/WP-XXXX-claim-<agent>`.
2. First commit on the branch: in the WP file set `status: in_progress`, `owner: <agent>`
   (e.g. `claude`, `codex`, `gemini`, `cursor`, `human-<name>`), `branch:`, `updated:`; run
   `programctl status`; commit `chore(program): claim WP-XXXX`.
3. Open a **draft** pull request titled `[WP-XXXX] <title>` using the PR template.
4. If `git push` of the branch is rejected because it exists, someone else has it. Pick another.

Abandoning a claim: set `status: ready`, `owner: unassigned`, write why in the Progress log, push,
close the draft PR and delete the claim branch.

## 4. Building

**Definition of Ready** (a WP may move to `ready` only when all hold):
goal and scope are clear; every acceptance criterion is testable; test plan names test files;
dependencies listed; requirement IDs listed; relevant ADRs exist and are Accepted; size is L or smaller
(split XL work packages before starting).

**How to work:**

- Write the tests for an acceptance criterion first, see them fail, then implement.
- Follow the existing patterns in the file you are editing (models → repositories → services →
  commands/routes → UI). Do not introduce a new pattern without an ADR.
- New user-visible capability goes behind its feature flag (WP-0010) until its phase gate passes.
- Keep `main` releasable: no half-migrations, no dead UI entry points without a flag.
- Commit small, in logical steps, each passing the fast checks. Push at least at every session end.
- Never commit secrets, real customer data or API keys. Tests use the scripted model provider.

**Definition of Done** (a WP may move to `done` only when all hold):

- [ ] Every acceptance criterion ticked, each proven by a test tagged `Covers: REQ-…`
- [ ] Full regression suite green in CI; test count not lower than before
- [ ] No new `#[ignore]`, `.skip`, `todo!()`, `unimplemented!()` or commented-out tests
- [ ] Lint, format and typecheck clean; no new warnings
- [ ] Migrations (if any) pass the upgrade test; numbering rule respected
- [ ] Security checklist answered for WPs touching tools, approvals, auth, identities or data access
- [ ] Clean-room checklist ticked
- [ ] User-facing docs and API reference updated; ADRs written for any design decision
- [ ] Progress log has a final entry; `STATUS.md` regenerated
- [ ] Reviewed by someone other than the author and merged by the owner

## 5. Testing

The full rules are in `testing/TEST-STRATEGY.md`. In short:

- **The regression suite is everything CI runs.** It must be green before merge. A red `main` is
  a stop-the-line event: the next agent's first job is to fix it (or revert the cause).
- Run before every push: the Rust tests for the crates you touched, `tsc`, `vitest` and
  `programctl validate`. Run the full suite before marking `in_review`.
- Tag tests with the requirement and WP they prove: `// Covers: REQ-LOOP-008 (WP-0121 AC-1)`.
  `programctl trace` builds the requirement → test matrix from these tags.
- Agent and orchestration behaviour is tested with the scripted model provider (WP-0005) — never
  with live models in CI. Live-model evaluation runs separately and never blocks a merge.
- Record the new test totals in `testing/REGRESSION.md` when your WP adds tests.

## 6. Session end and handoff (mandatory, even if the work is unfinished)

Assume the next session is a different AI that has never seen this conversation.

1. **Progress log** in the WP file — append a dated entry:
   ```
   ### 2026-10-20 codex
   Done: AC-1, AC-2 (tests in core/tests/agent_loop_limits.rs). Migration 0081 added.
   State: branch green locally; CI not yet run. / Branch RED: test X fails because Y.
   Next: AC-3 - wall-clock limit. Start in chat_service.rs:1286; the provider call is not cancellable yet.
   Gotchas: the scripted provider needs `with_latency` for timeout tests.
   Commands: cargo test -p lanesra-core --test agent_loop_limits
   ```
2. Tick the acceptance criteria that are actually proven. Update `status` and `updated`.
3. If you found a risk, a spec ambiguity or a needed decision: add it to `RAID.md`.
4. Run `programctl new-log --agent <you> --wp WP-XXXX` and fill in the session log (short).
5. `programctl status`, `programctl validate`, commit, push. Never leave work only on your machine.

## 7. Review and merge

- Move to `in_review`: CI green, Definition of Done met except review, PR marked ready, `pr:` set.
- **Cross-review:** a different agent or a person reviews. Prefer a different AI vendor than the
  author — it catches different mistakes. The reviewer checks the diff against the acceptance
  criteria and the spec, not just that tests pass, and tries to break it.
- Review findings are answered on the PR; fixes go on the same branch.
- The owner merges. After merge, the author (or next agent) sets `status: done` in a follow-up
  commit on the next branch, or the owner does so on merge; delete the `wp/` branch.

## 8. Changing the plan

- **New work found while building:** do not widen your PR. Add a new WP file (status `proposed`,
  next free id in that phase's range) in your PR, or a RAID item, and mention it in the Progress log.
- **A WP is too big:** split it into new WPs; mark the original `dropped` with a link to the parts.
- **Refinement:** at each gate review, the next phase's WPs are refined (acceptance criteria made
  precise, split to L or smaller) and moved from `proposed` to `ready`.
- **Spec change or requirement meaning change:** ADR + owner acceptance, then update REQUIREMENTS.md.
- Work package id ranges: Phase 0 `WP-00xx`, 1 `WP-01xx`, 2 `WP-02xx`, 3 `WP-03xx`, 4 `WP-04xx`, v2 `WP-09xx`.

## 9. Gates

Each phase closes with a gate (`gates/G<n>-*.md`). The next phase's WPs stay `proposed` until the
gate passes. To run a gate: fill in every check with evidence (CI run links, test names, `trace`
output, demo notes), list open RAID items, and ask the owner to sign. Gates come from the spec's
roadmap: G1 every run produces a full trace; G2 templates pass their evaluation suites; G3 security
review signed off; G4 GA. G0 (engineering readiness) is added by this plan.

## 10. What to update, when

| Event | Update |
|---|---|
| Start a WP | Push `wp/` branch; WP status/owner/branch; draft PR; `programctl status` |
| Any working session ends | WP Progress log; ticked ACs; session log; push |
| Discover a risk, issue, question or assumption | `RAID.md` |
| Make a design decision that others must follow | New ADR (`decisions/`), link from WP |
| Add tests | `Covers:` tags; `testing/REGRESSION.md` totals when the WP completes |
| WP ready for review | status `in_review`, `pr:` link, `programctl status` |
| WP merged | status `done`; delete branch; `programctl status` |
| Phase complete | Gate file with evidence; owner sign-off; refine next phase |
| Spec meaning changes | ADR, then `spec/REQUIREMENTS.md` |

## 11. Conflicts

- `STATUS.md` conflicts: take either side, run `programctl status`, commit. Never merge it by hand.
- Your WP file conflicts: keep both Progress log entries in date order.
- Migration number conflicts: renumber yours to the next free number above `main`'s maximum and
  update the `MIGRATIONS` list in `desktop/core/src/db/migrate.rs`.
- Two agents changed the same code for different WPs: the later PR rebases and resolves; if
  behaviour conflicts, stop and raise a RAID issue.

## 12. Escalation — stop and ask the owner when

- The spec is ambiguous or two requirements conflict.
- A shortcut looks necessary to finish (it is not allowed — ask instead).
- A security or data-protection concern appears that no WP covers.
- An existing test seems wrong, or a pre-existing failure blocks you.
- The work would change a public API, the database in a non-additive way, or the online demo's behaviour.

Write the question as a `Q-` item in `RAID.md` and state it in the PR. Keep working on parts that
do not depend on the answer.

## 13. Conventions

- Commits: Conventional Commits with the WP id, e.g. `feat(core): step round limit and partial results [WP-0121]`.
- Branches: `wp/WP-XXXX-short-name`. PR titles: `[WP-XXXX] Title`.
- Dates: ISO `YYYY-MM-DD`, UTC.
- Required CI checks (set on `main` by the owner once WP-0003 lands): `rust`, `tauri`, `web`, `demo`, `program`.
