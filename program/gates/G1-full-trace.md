# G1 — Every run produces a full trace

Closes **Phase 1 · Foundations**. Until this gate passes, **Phase 2 · Playground** work packages stay `proposed`.

Status: **In progress** (checks 1, 2, 3, 5 and 6 met on 2026-11-01)

## Checks

Each check needs evidence: a CI run link, test names, `programctl` output or a demo note.

| # | Check | Evidence | Result |
|---|---|---|---|
| 1 | Full-trace conformance suite green for every run entry point (WP-0113) | `core/tests/trace_conformance.rs` (sequential, consensus and peer-review pipelines, graph agent node, workflow action, schedule and webhook triggers, agent chat; and a test that a broken trace or audit is refused) and `server/tests/trace_conformance.rs` (webhook route, MCP-started and CLI-started runs), green locally and in CI on the WP-0113 PR | ☑ |
| 2 | 100% of tool calls have a complete audit trail (REQ-NFR-003) | Every cell of the WP-0113 suite asserts one `ai_agent_tool_call` audit entry per tool call that reached the firewall, naming the agent, the tool and the effective principal (`assert_tool_audit`) | ☑ |
| 3 | Run events validate against published JSON Schemas (WP-0112) | `core/tests/run_events.rs`, `server/tests/run_events.rs`; every WP-0113 cell also validates its run's events against `program/spec/schemas/events/` | ☑ |
| 4 | Task contracts, output schemas, limits and partial results proven by tests (WP-0121, WP-0131, WP-0132) | | ☐ |
| 5 | All approvals on ai_approvals; legacy paths removed (WP-0150) | WP-0150 (#730): pipeline steps in every topology, graph Approval nodes, held tool calls and voice plans all wait on `ai_approvals` and are decided through `approval_service::decide`; `core/tests/approvals_unification.rs`, `server/tests/approvals_http.rs` | ☑ |
| 6 | Prompt catalog in place; no hard-coded system prompts (WP-0130) | WP-0130 done: every model instruction is a catalog prompt with a cases suite (`core/tests/prompt_catalog.rs`, `prompts/catalog.lock`) | ☑ |
| 7 | OIDC and SAML sign-in working with negative tests (WP-0102, WP-0103) | | ☐ |
| 8 | `programctl trace` shows every Phase 1 requirement with a tagged test | | ☐ |
| 9 | Threat model updated for Phase 1; no open high-severity findings | | ☐ |
| 10 | Team Workspace safe to expose and recoverable: setup token, sign-in throttling, public URL (WP-0106); backups with key recovery, pre-upgrade backup, readiness check (WP-0180) | | ☐ |
| 11 | Phase 2 WPs refined and `ready`; re-forecast written below | | ☐ |

## Open RAID items at the gate

(list, with the owner's decision on each)

## Throughput and re-forecast

| Measure | Value |
|---|---|
| Work packages completed this phase | |
| Estimate points delivered (S=1, M=2, L=4, XL=8) | |
| Calendar days | |
| Forecast for remaining phases | |

## Sign-off

| Role | Name | Date | Decision |
|---|---|---|---|
| Program owner | | | |
