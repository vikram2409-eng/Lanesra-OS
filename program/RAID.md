# RAID log — risks, assumptions, issues, dependencies, questions

Anyone adds items; never delete — close them with a date and outcome. IDs are never reused.
Questions (`Q-`) are for the program owner; agents keep working on what does not depend on the answer.

## Questions for the owner

| ID | Question | Why it matters | Default until answered | Status |
|---|---|---|---|---|
| Q-01 | Which AI tools will build (Claude Code, Codex, Gemini, Cursor, Copilot, other)? | Each reads a different instruction file; some cannot push branches or open PRs | `AGENTS.md` + `CLAUDE.md` + `GEMINI.md` cover all common tools | Answered 2026-10-16: Muse AI and Claude Code (ADR-0004, AGENT-PROFILES.md) |
| Q-02 | Will agents work in parallel, or one at a time with handoffs? | Parallel needs claims and lane discipline; sequential is simpler | Protocol supports both | Answered 2026-10-16: either; lanes allow parallel work (ADR-0004) |
| Q-03 | Who reviews and merges PRs? Will you enable branch protection on `main`? | Without required checks, CI cannot stop a bad merge | Owner merges; cross-AI review | Answered 2026-10-16: Claude Code enables branch protection after CI (WP-0003); agents cross-review; owner merges |
| Q-04 | Mirror work packages to GitHub Issues / a Project board too? | Issues give a web board and notifications; the repo files stay the source of truth | Repo only; a one-way sync can be added later | Answered 2026-10-16: yes - Issues mirror (WP-0017) |
| Q-05 | The repo appears to be public. Is a public build plan acceptable? | `program/` will be visible to anyone | Proceed (it is excluded from the website) | Answered 2026-10-16: yes |
| Q-06 | How far may the online demo change to host the Playground (WP-0013)? | `app.js` is a 1.7 MB no-build file; sharing code may need a build step | Blocks WP-0013 | Answered 2026-10-16: any change needed; WP-0013 unblocked |
| Q-07 | Which identity providers must SSO support and test against (Entra ID, Okta, Google, Keycloak, ADFS)? Desktop single-user too, or server only? | Shapes WP-0101..0104 and test tenants | Server only; test with a local mock IdP plus Keycloak | Answered 2026-10-16: both editions (WP-0101, WP-0105) |
| Q-08 | Pull a minimal "untrusted tool output" defence forward from Phase 4 to Phase 1–2? (see R-04) | External MCP and connector tools already run; the Playground widens their use | Spec order (Phase 4) | Answered 2026-10-16: yes (REQ-TOOL-015, WP-0145) |
| Q-09 | Default monthly AI budget per workspace (spec: "set in planning")? | Needed by WP-0319; customers bring their own keys, so this caps their spend | Proposal: USD 100 per workspace per month, raisable only by an AI Admin | Open - owner said no API keys for the build itself; the product default amount is still needed |
| Q-10 | Budget and API keys for live-model evaluation runs (nightly, non-blocking)? | Template gates (G2) need real-model evidence | Recorded runs only | Answered 2026-10-16: no API keys; local Ollama + recorded runs (WP-0224) |
| Q-11 | Accept Phase 0 (~3 weeks) before Phase 1, and a re-forecast at G1? | The spec's 28 weeks has no precursor phase | Yes | Answered 2026-10-16: yes; timeline depends on available AI tokens (ROADMAP Forecasting) |
| Q-13 | Besides OpenAI, Gemini, Claude, Ollama and OpenAI-compatible on-prem servers, must the Playground support enterprise-hosted variants: Azure OpenAI, AWS Bedrock, Google Vertex AI? Which on-prem servers do your customers use? | Each needs its own auth and URL scheme (WP-0146) | — | Answered 2026-10-16: add Azure OpenAI, Google Vertex AI and AWS Bedrock; vLLM is the first-class on-prem server (WP-0146) |
| Q-12 | Confirm how Muse AI works: can it read the repo and open PRs itself, or do you relay its scripts into `scripts/` by hand? | Decides whether Muse can take work packages alone and whether WP-0018 is enough | AGENT-PROFILES.md assumes GitHub-only relay | Open |

## Risks

| ID | Risk | Likelihood | Impact | Mitigation | Owner | Status |
|---|---|---|---|---|---|---|
| R-01 | Scope exceeds available AI capacity: 126 work packages (~420 estimate points); pace depends on the owner's token budget for Claude Code and Muse AI | High | High | Forecast from points per session (ROADMAP); rolling-wave refinement; split XL; prioritise the critical path | Owner | Open |
| R-02 | Different AI agents produce inconsistent designs or conventions | High | High | ADRs before parallel work (WP-0009); protocol; cross-vendor review; CI | Planner | Open |
| R-03 | Context lost between agents and sessions | High | Medium | Mandatory Progress log and session log; repo is the only memory | All agents | Open |
| R-04 | Prompt-injection defence (content labels) lands in Phase 4 while external MCP/connector tools already run and the Playground widens use | Medium | High | Minimal defence moved to Phase 1 (WP-0145, ADR-0004); full labels stay Phase 4 (WP-0406) | Owner | Mitigation planned |
| R-05 | Desktop (Tauri commands) and server (REST) paths diverge, doubling work and bugs | Medium | Medium | Gap audit WP-0008 maps both; prefer logic in `core` only | Planner | Open |
| R-06 | Online demo drifts from the desktop Team format | Medium | Medium | Shared JSON Schema (WP-0013); demo E2E | Owner | Open |
| R-07 | SQLite growth from traces and checkpoints on desktop | Medium | Medium | Perf baseline (WP-0015); size budgets; delta checkpoints (WP-0413) | — | Open |
| R-08 | Clean-room contamination: AI implementers know these frameworks from training | Medium | Medium | Vocabulary check, fresh prompts, review checklist (WP-0012); design traced to spec | Reviewers | Open |
| R-09 | Live-model non-determinism makes evaluation gates flaky | High | Medium | Scripted provider and recorded runs for CI; N repeats with thresholds for gates | — | Open |
| R-10 | SAML/OIDC implementation flaws | Medium | High | Maintained libraries only; negative test vectors; security review | — | Open |
| R-11 | Single owner is a bottleneck for decisions, reviews and merges | High | Medium | Batch decisions at gates; cross-AI review before owner review | Owner | Open |
| R-12 | `chat_service.rs` (1,433 lines) is touched by many Phase 1 WPs → merge conflicts | High | Medium | Sequence WP-0121 first and modularise the loop there | Planner | Open |
| R-13 | The build team has no hosted-model keys, but organisations will bring any provider (OpenAI, Gemini, Claude, Ollama, on-prem): adapters cannot be tested against live hosted APIs in CI | High | High | Contract fixtures from published API formats; local Ollama/vLLM for real runs; in-product provider conformance suite organisations run with their own keys (WP-0146, WP-0224); a release check with a borrowed or owner key where possible | — | Open |
| R-14 | Muse AI cannot run tests; in a week when only Muse has usage left, nothing tests its changes except CI | High | High | CI early (WP-0003); Agent apply Action (WP-0018); owner review mandatory when no second agent is available (ADR-0005) | Owner | Open |
| R-17 | A usage limit cuts an agent off mid-session before it writes its handoff, losing context | High | Medium | Push early and often; next agent reconstructs from commits and CI and writes the missing entry first (AGENT-PROFILES "Switching") | All agents | Open |
| R-15 | `set_entity_values` treats a missing ordinary-field key as "clear" (lookup fields fixed in WP-0016): API callers sending partial updates can blank other fields | Medium | Medium | Audit callers and decide partial-update semantics in WP-0008; add tests | — | Open |
| R-16 | Debug test builds use ~30 GB of disk; agent sandboxes can run out | High | Low | `cargo clean` guidance in AGENT-PROFILES; CI on hosted runners | — | Open |

## Assumptions

| ID | Assumption | Status |
|---|---|---|
| A-01 | The spec dated 2026-10-14 (copied to `spec/`, SHA-256 `529ffe55…`) is the baseline; changes go through ADRs | Open |
| A-02 | Worker pool and remote workers target the Team Workspace server first; SSO covers both editions (ADR-0004) | Revised 2026-10-16 |
| A-03 | The stack stays Rust + TypeScript + SQLite; Python appears only inside the Phase 4 code sandbox | Open |
| A-04 | Phase durations in the spec are proposals; the real pace depends on AI token budget (ADR-0004) | Confirmed 2026-10-16 |
| A-05 | CI uses GitHub-hosted Linux runners; Windows is built only on release | Open |

## Issues (facts found, needing action)

| ID | Issue | Found | Action | Status |
|---|---|---|---|---|
| I-01 | `npm ci` fails: `desktop/package-lock.json` out of sync with `package.json`; the release workflow runs `npm ci` and would fail | 2026-10-16 | WP-0002 | Fixed 2026-10-16 |
| I-02 | `desktop/Cargo.lock` records lanesra-os 0.14.0 while the crate is 0.15.0 | 2026-10-16 | WP-0002 | Fixed 2026-10-16 |
| I-03 | 7 tests failing on `main` (of 1,004): six industry reference-package workflows that should update a related record (auto service, field service, property management, real estate ×2, recruitment) and one System Graph hard-delete test. List in `testing/REGRESSION.md` | 2026-10-16 | WP-0016 | Fixed 2026-10-16 (root cause: lookup-field cache, see WP-0016) |
| I-04 | No pull-request CI | 2026-10-16 | WP-0003 | Open |
| I-05 | Spec says a THIRD_PARTY_NOTICES file exists; it does not | 2026-10-16 | WP-0012 | Open |
| I-06 | Migration runner skips any version below the max applied; out-of-order merges lose migrations | 2026-10-16 | ADR-0003; CI check in WP-0003; test in WP-0007 | Open |
| I-07 | jsdom 30 requires Node ≥ 22.22.2; the dev container has 22.22.0 (warning only today) | 2026-10-16 | WP-0002 pins Node | Fixed 2026-10-16 |
| I-08 | Frontend has 5 test files (40 tests) for 134 `.tsx` files; no E2E | 2026-10-16 | WP-0006, WP-0014 | Open |

## Dependencies (external)

| ID | Dependency | Needed by | Status |
|---|---|---|---|
| D-01 | Branch protection and required checks on `main` | After WP-0003 | Claude Code enables it (ADR-0004); owner clicks if the API is unavailable |
| D-02 | IdP test tenants or a containerised IdP (Keycloak) for SSO tests | WP-0102, WP-0103 | Open |
| D-03 | Model API keys as GitHub secrets | — | Closed 2026-10-16: none will be provided (ADR-0004); see WP-0224 |
