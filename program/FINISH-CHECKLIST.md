# When a work package finishes

Every AI (Claude Code or Muse AI) and every person runs this list before a work package goes to
review, and again after it merges. The program dashboard shows the same list; `programctl brief`
puts it on every task card. Steps are grouped by when they happen.

Format (parsed by `programctl export`, keep it): `## Group` headings, then `- [ ] **Short name** — detail`.

## Prove it

- [ ] **Acceptance criteria proven** — every criterion ticked in the work package file, each backed by a test; nothing ticked that no test proves
- [ ] **Tests tagged** — every new test has a `Covers: REQ-… (WP-XXXX AC-n)` comment so `programctl trace` links it to its requirement
- [ ] **Added to the regression suite** — new test files live where CI runs them (`desktop/core/tests`, `desktop/server/tests`, `desktop/src/**/__tests__`, `desktop/e2e`); none skipped, ignored or weakened
- [ ] **Full regression suite green** — Rust, frontend, demo and program checks pass (CI, or locally for Claude Code); a Muse change says "untested locally - see CI" until CI is green
- [ ] **Regression totals recorded** — new test counts written in `program/testing/REGRESSION.md`; totals never go down without a written reason

## Record it

- [ ] **Work package file updated** — `status: in_review`, `owner`, `pr`, `updated`; a final Progress log entry (done, state, next, gotchas)
- [ ] **Session log written** — `programctl new-log`; include points completed and tokens used if visible
- [ ] **Decisions captured** — an ADR in `program/decisions/` for any design choice others must follow
- [ ] **Risks and questions logged** — new risks, issues or owner questions in `program/RAID.md`; resolved ones closed with a date
- [ ] **Docs updated** — user-facing docs, API reference and README feature list where behaviour changed
- [ ] **Licences and security** — `THIRD_PARTY_NOTICES` for any new dependency; threat model updated if tools, approvals, auth or data access changed
- [ ] **Follow-up work captured** — anything found but out of scope becomes a new work package (status `proposed`), never a widened PR

## Publish it

- [ ] **Tracker regenerated** — `programctl status` and `programctl validate` pass; `STATUS.md` committed
- [ ] **Pushed** — everything committed and pushed to the work package branch
- [ ] **Pull request ready** — PR titled `[WP-XXXX] Title`, template filled (requirements, criteria, clean-room and security checks), marked ready for review
- [ ] **Review requested** — from the other AI if it has usage left, otherwise the owner

## After merge

- [ ] **Marked done** — `status: done` and the PR link in the work package file; claim branch deleted
- [ ] **Unblocked work checked** — the "Unblocks" list on the dashboard: dependants that are now ready are moved from `proposed` to `ready` if they meet the Definition of Ready
- [ ] **Gate checked** — if this was the last work package of a phase, the gate file in `program/gates/` is filled with evidence for the owner to sign
- [ ] **Dashboard refreshed** — GitHub Issues update automatically on merge; the program dashboard page (https://claude.ai/artifact/TANp9Vzmur4A37bsgHEtKg) is republished by Claude Code at its next session (`program/dashboard/README.md`)
