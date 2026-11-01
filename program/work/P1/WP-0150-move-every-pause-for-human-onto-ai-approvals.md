---
id: WP-0150
title: Move every pause-for-human onto ai_approvals
phase: 1
epic: E1.6 Approvals
status: in_progress
owner: claude
depends_on: [WP-0110, WP-0007]
requirements: [REQ-GOV-011]
threats: [T-05]
estimate: L
editions: [desktop, workspace, demo]
delivered: [desktop, workspace, demo]
docs: [agent-versioning-and-approvals]
branch: claude/enterprise-build-planning-jxy2kq
pr: 
updated: 2026-11-01
---

# WP-0150 — Move every pause-for-human onto ai_approvals

## Goal

Retire the three older pause mechanisms in ai_orchestration_service; every human decision goes through the durable approvals inbox.

## Scope

- Map each legacy path (found in WP-0008)
- Migration of pending legacy pauses
- Resume semantics unchanged for users
- Gap audit findings (WP-0008, `program/research/gap-audit.md`):
  - Legacy paths found by the audit: (a) pipeline `requires_approval` in all three topologies (`ai_orchestration_service.rs:449,508,644` → `ai_agent_runs.awaiting_approval`), (b) voice plans (`voice_execution_service.rs:257-262`; no resolver for `awaiting_approval`)
  - Policy `RequireApproval` writes an `ai_approvals` row but drops the tool call: make it pause and resume
  - Fix the stuck graph run when its approval is resolved from the generic inbox command (I-13)

## Acceptance criteria

- [x] AC-1: Each topology's pause/resume works via ai_approvals (existing tests updated, none deleted)
- [x] AC-2: In-flight legacy pauses survive the upgrade (migration fixture test)

## Test plan

- Add `core/tests/approvals_unification.rs` and `server/tests/approvals_http.rs`; update `firewall_chain.rs` step 3 (a held call now pauses the run); `ai_orchestration_topologies.rs` and `ai_hitl_and_observability.rs` pass unchanged
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0007 run and step states; ADR-0008 trace and events; ADR-0010 migration conventions; ADR-0012 feature flags. ADR-0010 applies to any migration this work package adds.
- Refine at the Gate 0 review: confirm scope against the gap audit (WP-0008) and ADRs (WP-0009), then set status to ready.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-01 claude: Refined proposed → ready and claimed (owner decision 2026-11-01, RAID Q-26: Claude takes all build work). Dependencies met (WP-0110, WP-0007 done); ADR-0007, ADR-0008 and ADR-0010 Accepted. Help lives in `agent-versioning-and-approvals` (there is no `approvals-inbox` article). Scope decisions: a held tool call pauses the run inside a sequential pipeline step or a graph agent node; consensus and peer-review steps, the Assistant chat and direct agent chats keep telling the agent the call needs approval, and an approval granted later lets the same call run once. Grants match agent, tool and arguments exactly and are used once, the core of WP-0152 (approvals bound to exact arguments).
- 2026-11-01 claude: Built. Migration 0088 (`ai_agent_runs.pending_approval_id`, `voice_action_plans.approval_id`, `ai_approvals.consumed_at`, backfill of waiting runs and plans). `approval_service::decide` resolves and resumes graph runs (Approval node, or a held call re-running its agent node), pipeline runs (any topology's approval step, or a held call re-running its step) and voice plans (back to the speaker to confirm); `resolve_unless_waiting` for the synchronous command. `approval_hold` pauses a step on a held call; `policy_engine_service::take_tool_grant` lets the approved call run once. Desktop inbox describes each kind of approval; Team Workspace gets `/api/admin/approvals/:id/decide`, and the browser's Agent Team run actions now use their async routes (RAID I-33, found here). Demo: pipelines and Agent Teams pause on a pending approval and a Pending Approvals inbox resumes them. Tests: 6 core (each topology through the inbox, run-screen approval decides the row, I-13, held call in a pipeline and a graph node, upgrade of waiting pauses), 1 server (HTTP), 1 frontend; `firewall_chain` step 3 updated.
