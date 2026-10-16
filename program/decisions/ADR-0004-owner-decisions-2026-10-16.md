# ADR-0004: Owner decisions of 2026-10-16

- Status: Accepted (decided by the program owner, Vikram Grover, 2026-10-16)
- Answers RAID questions Q-01 to Q-11
- Requirements added: REQ-PRE-013, REQ-PRE-014, REQ-GOV-028, REQ-TOOL-015, REQ-EVAL-012
- Work packages added or changed: WP-0003, WP-0013, WP-0017, WP-0018, WP-0101, WP-0105, WP-0145, WP-0221, WP-0224, WP-0319, WP-0406

## Decisions

| # | Question | Decision | Consequence in the plan |
|---|---|---|---|
| 1 | Which AI tools build this? | **Muse AI and Claude Code.** | Agent profiles in `program/AGENT-PROFILES.md`. Muse works through GitHub (its past commits arrive through the `apply-ontology-pr.yml` Action), so CI is where its changes get tested; WP-0018 adds a validated relay path. |
| 2 | Who reviews, merges and protects `main`? | **Claude Code turns branch protection on** once CI exists. | WP-0003 includes enabling protection. Cross-review: Claude Code reviews Muse's PRs and Muse reviews Claude Code's, wherever Muse can read the PR. The owner keeps the final merge. |
| 3 | GitHub Issues / Project mirror? | **Yes.** | One-way mirror (WP-0017): one issue per work package, milestones per phase, status labels. Files stay the source of truth. |
| 4 | Public build plan OK? | **Yes.** | No change; `program/` stays out of the website build. |
| 5 | How far may the online demo change? | **Whatever is necessary** to host the Playground. | WP-0013 unblocked; a build step for the demo is allowed. |
| 6 | Which sign-in providers, which editions? | **Both editions.** Providers: the common enterprise set. | WP-0101 scope widened; new WP-0105 for desktop SSO (system browser + loopback, keychain). First-class: Okta, Microsoft Entra ID, Google (OIDC); Okta, Entra ID (SAML); Keycloak for local tests. |
| 7 | Move a minimal prompt-injection defence earlier? | **Yes.** | New REQ-TOOL-015 and WP-0145 in Phase 1: untrusted tool output fenced, cannot change instructions or tools, write/external calls after untrusted reads need approval. WP-0406 (Phase 4) extends it. |
| 8 | API keys for live evaluation; default AI budget? | **No API keys** - only Claude Code and Muse AI are used. Budget amount not set. | All CI testing uses the scripted provider and recorded runs. Real-model evidence for Gate 2 comes from a local Ollama model (WP-0224). Paid-provider adapters are verified against documented request/response fixtures. Budget default stays open (RAID Q-09). |
| 9 | Accept Phase 0 and re-forecasting? | **Yes; the timeline depends on available AI tokens.** | Calendar weeks are dropped as a measure. Forecasts use estimate points per session and the token budget available (ROADMAP "Forecasting"). |

## Consequences

- The program has no paid model access, so no CI job or gate may depend on a hosted model API.
- Muse AI cannot run the test suite itself; WP-0003 (CI) and WP-0018 (relay path) are on the critical path for Muse's work.
- Branch protection will block direct pushes to `main` for everyone, including the Muse Action, which already works on branches.
