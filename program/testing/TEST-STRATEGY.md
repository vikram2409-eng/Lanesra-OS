# Test strategy

How the Agent Playground build is tested, so that every agent tests the same way and the regression
suite only grows.

## Principles

1. **Every acceptance criterion is proven by an automated test.** Manual checks are extra, never instead.
2. **The regression suite is everything CI runs** on a pull request. It must be green to merge.
3. **The suite only grows.** Test totals are recorded in `REGRESSION.md`; a PR that lowers a total
   must explain why (e.g. tests merged into a table-driven one) and the reviewer must agree.
4. **Never skip to pass.** No new `#[ignore]`, `.skip`, `.only`, `todo!()`, or weakened assertions.
   A flaky test is a bug: fix it or raise a RAID issue; never retry-until-green.
5. **Deterministic by default.** No live model calls, network or wall-clock sleeps in CI tests. The
   build team has no model API keys (ADR-0004); nothing may depend on one. The *product* supports
   every provider an organisation brings (REQ-LOOP-022), so each adapter needs contract tests.
   Agent behaviour uses the scripted model provider; time uses a controllable clock.
6. **Traceable.** Each test that proves a requirement carries a tag comment:
   `// Covers: REQ-LOOP-008 (WP-0121 AC-1)`. `programctl trace` builds the matrix from these tags.

## Test layers

| Layer | Tooling | Location | Runs | Proves |
|---|---|---|---|---|
| Rust unit | `#[test]` in modules | `desktop/*/src/**` | Every PR | Pure logic: compilers, state machines, parsers, rankers |
| Rust integration | `cargo test`, in-memory SQLite | `desktop/core/tests/`, `desktop/server/tests/`, `desktop/cli/tests/` | Every PR | Services end to end through repos and the firewall |
| Agent behaviour | Scripted model provider (WP-0005) | Same as integration | Every PR | Loop limits, retries, guardrails, delegation, modes, traces |
| API contract | JSON Schema validation, OpenAPI | `desktop/server/tests/` | Every PR | Event and endpoint shapes are stable |
| Migration upgrade | Fixture databases (WP-0007) | `desktop/core/tests/migration_upgrade.rs` | Every PR | No data loss on upgrade; numbering rule |
| Frontend unit | Vitest + Testing Library + axe | `desktop/src/**/__tests__`, `*.test.tsx` | Every PR | Components, hooks, accessibility |
| End to end | Playwright + pre-installed Chromium (WP-0006) | `desktop/e2e/` | PRs touching UI/server/demo; nightly full | User journeys in server mode and the online demo |
| Security | Negative tests, authz matrix, injection corpus, `cargo deny`, `npm audit`, secret scan | Throughout + CI | Every PR; corpus nightly | Threat model mitigations hold |
| Performance | Benchmarks / timed tests (WP-0015) | `desktop/core/benches/` | Nightly | Budgets in `PERFORMANCE.md` |
| Provider contracts | Fixtures from each provider's published API format (OpenAI, Anthropic, Gemini, Azure OpenAI, Vertex AI, Bedrock incl. SigV4 test vectors, OpenAI-compatible, vLLM, Ollama) | `desktop/core/tests/provider_contracts.rs` | Every PR | Every bring-your-own-model provider works (WP-0146) |
| Real-model evaluation | Local model via vLLM or Ollama, recorded for replay; organisations run the in-product conformance suite with their own keys | Evaluation suites | On demand; CI replays the recordings | Template quality (G2 evidence, WP-0224) |
| Program | `programctl` self-tests, validate, status --check | `program/tools/` | Every PR | Tracker integrity |

## What a work package must add

- One or more tests per acceptance criterion, tagged with the requirement and AC.
- Negative tests for every rule that denies, blocks, caps or limits (the deny path is the product).
- For any new table or column: an upgrade-fixture assertion.
- For any UI: a vitest with axe, and an E2E step if it is part of a user journey.
- For any new event or endpoint: schema validation in tests.

## Naming

- Rust test files by capability: `agent_loop_limits.rs`, `approvals_unification.rs`.
- Test names read as behaviour: `round_limit_requests_final_answer_without_tools`.
- E2E specs by journey: `playground-run-template.spec.ts`.

## The scripted model provider (from WP-0005)

Usage examples will be added here when WP-0005 lands. Until then, do not write tests that call a real provider.

## Running everything locally

```bash
cd desktop
cargo test --no-fail-fast -p lanesra-core -p lanesra-server -p lanesra-cli
npm ci && npx tsc --noEmit && npm test && npm run build
cd .. && node --check app.js
python3 -m unittest discover -s program/tools -p 'test_*.py'
python3 program/tools/programctl.py validate && python3 program/tools/programctl.py status --check
```

Note: `cargo test` without `--no-fail-fast` stops at the first failing test binary and under-reports.

## Red main

A red `main` stops the line. The next agent to start a session fixes it or reverts the cause before
any other work, and records it in the RAID log.
