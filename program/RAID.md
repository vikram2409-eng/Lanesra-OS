# RAID log — risks, assumptions, issues, dependencies, questions

Anyone adds items; never delete — close them with a date and outcome. IDs are never reused.
Questions (`Q-`) are for the program owner; agents keep working on what does not depend on the answer.

## Questions for the owner

| ID | Question | Why it matters | Default until answered | Status |
|---|---|---|---|---|
| Q-01 | Which AI tools will build (Claude Code, Codex, Gemini, Cursor, Copilot, other)? | Each reads a different instruction file; some cannot push branches or open PRs | `AGENTS.md` + `CLAUDE.md` + `GEMINI.md` cover all common tools | Open |
| Q-02 | Will agents work in parallel, or one at a time with handoffs? | Parallel needs claims and lane discipline; sequential is simpler | Protocol supports both | Open |
| Q-03 | Who reviews and merges PRs? Will you enable branch protection on `main`? | Without required checks, CI cannot stop a bad merge | Owner merges; cross-AI review | Open |
| Q-04 | Mirror work packages to GitHub Issues / a Project board too? | Issues give a web board and notifications; the repo files stay the source of truth | Repo only; a one-way sync can be added later | Open |
| Q-05 | The repo appears to be public. Is a public build plan acceptable? | `program/` will be visible to anyone | Proceed (it is excluded from the website) | Open |
| Q-06 | How far may the online demo change to host the Playground (WP-0013)? | `app.js` is a 1.7 MB no-build file; sharing code may need a build step | Blocks WP-0013 | Open |
| Q-07 | Which identity providers must SSO support and test against (Entra ID, Okta, Google, Keycloak, ADFS)? Desktop single-user too, or server only? | Shapes WP-0101..0104 and test tenants | Server only; test with a local mock IdP plus Keycloak | Open |
| Q-08 | Pull a minimal "untrusted tool output" defence forward from Phase 4 to Phase 1–2? (see R-04) | External MCP and connector tools already run; the Playground widens their use | Spec order (Phase 4) | Open |
| Q-09 | Default monthly AI budget per workspace (spec: "set in planning")? | Needed by WP-0319 | — | Open |
| Q-10 | Budget and API keys for live-model evaluation runs (nightly, non-blocking)? | Template gates (G2) need real-model evidence | Recorded runs only | Open |
| Q-11 | Accept Phase 0 (~3 weeks) before Phase 1, and a re-forecast at G1? | The spec's 28 weeks has no precursor phase | Yes | Open |

## Risks

| ID | Risk | Likelihood | Impact | Mitigation | Owner | Status |
|---|---|---|---|---|---|---|
| R-01 | Scope exceeds schedule: ~120 work packages vs a 28-week proposal; Phase 3 has 31 WPs incl. 6 XL | High | High | Rolling-wave refinement; measure throughput in P0–P1; re-forecast at each gate; split XL | Owner | Open |
| R-02 | Different AI agents produce inconsistent designs or conventions | High | High | ADRs before parallel work (WP-0009); protocol; cross-vendor review; CI | Planner | Open |
| R-03 | Context lost between agents and sessions | High | Medium | Mandatory Progress log and session log; repo is the only memory | All agents | Open |
| R-04 | Prompt-injection defence (content labels) lands in Phase 4 while external MCP/connector tools already run and the Playground widens use | Medium | High | Q-08; threat model in WP-0011 names interim controls (read-only defaults, approvals) | Owner | Open |
| R-05 | Desktop (Tauri commands) and server (REST) paths diverge, doubling work and bugs | Medium | Medium | Gap audit WP-0008 maps both; prefer logic in `core` only | Planner | Open |
| R-06 | Online demo drifts from the desktop Team format | Medium | Medium | Shared JSON Schema (WP-0013); demo E2E | Owner | Open |
| R-07 | SQLite growth from traces and checkpoints on desktop | Medium | Medium | Perf baseline (WP-0015); size budgets; delta checkpoints (WP-0413) | — | Open |
| R-08 | Clean-room contamination: AI implementers know these frameworks from training | Medium | Medium | Vocabulary check, fresh prompts, review checklist (WP-0012); design traced to spec | Reviewers | Open |
| R-09 | Live-model non-determinism makes evaluation gates flaky | High | Medium | Scripted provider and recorded runs for CI; N repeats with thresholds for gates | — | Open |
| R-10 | SAML/OIDC implementation flaws | Medium | High | Maintained libraries only; negative test vectors; security review | — | Open |
| R-11 | Single owner is a bottleneck for decisions, reviews and merges | High | Medium | Batch decisions at gates; cross-AI review before owner review | Owner | Open |
| R-12 | `chat_service.rs` (1,433 lines) is touched by many Phase 1 WPs → merge conflicts | High | Medium | Sequence WP-0121 first and modularise the loop there | Planner | Open |

## Assumptions

| ID | Assumption | Status |
|---|---|---|
| A-01 | The spec dated 2026-10-14 (copied to `spec/`, SHA-256 `529ffe55…`) is the baseline; changes go through ADRs | Open |
| A-02 | Multi-user features (SSO, worker pool, remote workers) target the Team Workspace server first; desktop is single-user | Open |
| A-03 | The stack stays Rust + TypeScript + SQLite; Python appears only inside the Phase 4 code sandbox | Open |
| A-04 | Phase durations in the spec are proposals, not commitments | Open |
| A-05 | CI uses GitHub-hosted Linux runners; Windows is built only on release | Open |

## Issues (facts found, needing action)

| ID | Issue | Found | Action | Status |
|---|---|---|---|---|
| I-01 | `npm ci` fails: `desktop/package-lock.json` out of sync with `package.json`; the release workflow runs `npm ci` and would fail | 2026-10-16 | WP-0002 | Open |
| I-02 | `desktop/Cargo.lock` records lanesra-os 0.14.0 while the crate is 0.15.0 | 2026-10-16 | WP-0002 | Open |
| I-03 | Test failing on `main`: `auto_service_reference_package::repair_completed_workflow_closes_the_originating_appointment` (appointment stays "Checked In") | 2026-10-16 | WP-0016 | Open |
| I-04 | No pull-request CI | 2026-10-16 | WP-0003 | Open |
| I-05 | Spec says a THIRD_PARTY_NOTICES file exists; it does not | 2026-10-16 | WP-0012 | Open |
| I-06 | Migration runner skips any version below the max applied; out-of-order merges lose migrations | 2026-10-16 | ADR-0003; CI check in WP-0003; test in WP-0007 | Open |
| I-07 | jsdom 30 requires Node ≥ 22.22.2; the dev container has 22.22.0 (warning only today) | 2026-10-16 | WP-0002 pins Node | Open |
| I-08 | Frontend has 5 test files (40 tests) for 134 `.tsx` files; no E2E | 2026-10-16 | WP-0006, WP-0014 | Open |

## Dependencies (external)

| ID | Dependency | Needed by | Status |
|---|---|---|---|
| D-01 | Owner enables branch protection and required checks on `main` | After WP-0003 | Open |
| D-02 | IdP test tenants or a containerised IdP (Keycloak) for SSO tests | WP-0102, WP-0103 | Open |
| D-03 | Model API keys as GitHub secrets for non-blocking live evaluation | Phase 2 gate evidence | Open |
