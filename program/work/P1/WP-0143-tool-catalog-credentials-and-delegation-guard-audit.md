---
id: WP-0143
title: Tool catalog, credentials and delegation guard audit
phase: 1
epic: E1.5 Tools and firewall
status: done
owner: claude
depends_on: [WP-0008]
requirements: [REQ-TOOL-001, REQ-TOOL-007, REQ-LOOP-012]
threats: [T-04, T-13, T-14]
estimate: S
editions: []
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/687
updated: 2026-10-22
---

# WP-0143 — Tool catalog, credentials and delegation guard audit

## Goal

Confirm and lock in with tests that every tool source passes the full firewall chain, tools never see credentials, and delegation beyond depth 4 returns a readable refusal while the run continues.

## Scope

- Tests only, plus fixes for any gap the tests reveal
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - Agent tool dispatch does not check the agent's own `action_names` (I-16, security): write a test proving an agent cannot call an unlisted tool (including admin tools), then enforce membership before dispatch
  - Detect A→B→A cycles in nested `run_agent_team` nodes (only direct self-recursion is blocked)

## Acceptance criteria

- [x] AC-1: Each tool source has a firewall-order test
- [x] AC-2: Credential never present in tool input or trace
- [x] AC-3: Depth-5 delegation refused, run continues
- [x] AC-4: An admin-configured outbound address on the server's own network or a cloud metadata endpoint is refused before connecting, including through DNS resolution and redirects (added 2026-10-20 for RAID R-27, owner-approved design 2026-10-19; test `core/tests/ssrf_guard.rs`)

## Test plan

- `desktop/core/tests/firewall_chain.rs`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- `desktop/core/tests/ssrf_guard.rs`, `desktop/core/tests/ssrf_guard_gate.rs` (AC-4)
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0009 error taxonomy; ADR-0010 migration conventions. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

- 2026-10-17 muse: Refined to ready per PROTOCOL.md §4. Threats linked (WP-0011 AC-3). ACs testable, test plan names files, dependencies and requirements listed.

- 2026-10-18 muse: SSRF guard WIP for Claude. DONE: ssrf_guard.rs (308 lines, edition-aware: metadata/link-local always blocked, loopback/private by edition, allow-list support, policy.network_blocked error); ADR-0009 policy.network_blocked added; integration tests (ssrf_guard.rs). BROKE: Attempted Edition threading through chat_service agent path (15+ files) — 110+ commits, CI never green, PR #674 closed. TODO for Claude: (1) Connect-time IP check via reqwest custom resolver (DNS rebinding defense); (2) Allow-list storage + audit logging (currently empty &[] with TODO); (3) T-13 → Mitigated and R-27 → Closed after tests pass. Handoff: program/log/2026/10/18-muse-WP-0143-handoff.md. Branch: wp/WP-0143-ssrf-wip (draft PR, not green). Owner: claude from 2026-10-20.

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-20 claude: SSRF guard built (AC-4). The WIP branch did not contain Muse's guard: `src/services/ssrf_guard.rs` was a copy of the test file and was not registered. Wrote `services/ssrf_guard.rs` from scratch, using Muse's design and the 308-line draft on the closed #674 branch, and fixed its gaps: IPv4-mapped and NAT64 spellings of metadata, 0.0.0.0, 100.64/10, a `/0` panic, IPv4-only CIDR, blocking DNS in async code, and no redirect or rebinding defence. Every outbound HTTP client in core is built by `ssrf_guard::client_builder()` (DNS resolver plus redirect policy); literal IPs are checked by `check_url`; SFTP and SMTP connect only to `allowed_addrs`. A test fails if a new `reqwest::Client::builder()` appears. Design decisions: (1) the edition is declared once per process by the binary (server: Team Workspace from `LANESRA_OUTBOUND_ALLOW`; desktop: Desktop) instead of threading it through every call path, which is what sank #674; an undeclared process fails closed, except in `test-support` builds (`ssrf_guard_gate.rs` proves a release build fails closed). (2) The allow-list belongs to the server operator, not each workspace admin, so one workspace cannot open the host's network to others (T-12). Threat T-13 Mitigated, RAID R-27 Closed. AC-1 to AC-3 (firewall-chain, credential and delegation tests, I-16, cycle detection) are still open.
- 2026-10-21 claude: AC-1 to AC-3 done. RAID I-16 fixed: `chat_service::execute_agent_tool` refuses any tool the agent was not offered (`validation.unknown_tool`), before the policy firewall, so an unlisted call never queues an approval. A→B→A nested Agent Teams are refused (`graph_runtime_service::NestedTeamGuard`, `policy.denied`) instead of running until each run's step limit. `core/tests/firewall_chain.rs` (8 tests on real runs): for record, admin, agent-internal, connector read and write, and MCP read and write tools, an unlisted call is refused before a blocked one, a blocked one before approval, approval before dispatch, and only an allowed call reaches its dispatcher (counted on the connector, MCP and memory side); the provider key and a connection secret appear in no tool input, tool result or stored trace; a fifth level of delegation is refused while the run completes; an A→B→A team chain is refused.
- 2026-10-22 claude: PR #687 merged by the owner (61553d6). All acceptance criteria proven; no user-facing edition (internal guard). Done.
