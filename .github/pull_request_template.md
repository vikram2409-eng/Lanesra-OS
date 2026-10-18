## Summary

<!-- What does this change, and why? -->

## Where

<!-- Which part of the repo does this touch? -->
- [ ] Website / online demo (repo root)
- [ ] Desktop app / Team Workspace server (`/desktop`)
- [ ] Build program tracking (`/program`)
- [ ] Docs only

## Build program (Agent Playground work only)

<!-- Delete this section for changes outside the build program. Title the PR "[WP-XXXX] Title". -->
- Work package: WP-XXXX
- Requirements: REQ-…
- Acceptance criteria proven in this PR: AC-…
- [ ] Work package Progress log updated; `programctl status` re-run; session log added
- [ ] New tests tagged `Covers: REQ-…`; test totals in `program/testing/REGRESSION.md` not lower
- [ ] No test skipped, ignored, deleted or loosened
- [ ] Migrations (if any) numbered above `main`'s maximum and listed in `migrate.rs`
- [ ] New user-visible capability is behind its feature flag

**Clean room** (orchestration and agent work):
- [ ] No prompt, persona or format text copied from CrewAI, Agent Framework, LangGraph or AutoGen
- [ ] No function transliterated from those projects; their source was not opened
- [ ] Design traced to the spec / an ADR (link):

**Security** (tools, approvals, auth, identities, data access):
- [ ] Threats from `program/security/THREAT-MODEL.md` this change mitigates or affects:
- [ ] Deny and limit paths have negative tests

## Verification

<!-- How did you confirm this works? Test output, commands run, screenshots for UI changes. -->
- [ ] `cargo test --no-fail-fast -p lanesra-core -p lanesra-server -p lanesra-cli --features lanesra-core/test-support` passes (desktop changes)
- [ ] `npm run build` and `npm test` pass (desktop frontend changes)
- [ ] `node --check app.js` passes (website changes)
- [ ] `python3 program/tools/programctl.py validate` passes (program changes)

## Related issues

<!-- Closes #123, or "N/A" -->
