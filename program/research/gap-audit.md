# Gap audit: the spec's assumptions against the code (WP-0008)

Audited 2026-10-16 on `main` at 96495f1 by Claude Code (REQ-PRE-008). The audit checks every
"existing piece" in the spec's *Where this fits* table and *Code map*, and the extra questions in
WP-0008's scope. Every verdict cites `path:line`. Paths are relative to `desktop/` unless they
start with `program/` or `app.js`. Key findings were re-read by hand after the search pass.

**Verdicts:** ✅ as the spec says · ◐ partly true · ✗ not true or missing · ⚠ true, but with a
defect the spec does not mention.

## Summary

The spec's main bet holds: the plumbing exists and every agent call already goes through one loop
(`chat_service::run_agent_once`), one firewall and one gateway. But **13 assumptions are wrong or
incomplete**, and **7 defects** must be fixed before building on top. In order of impact:

1. **Durable runs are not resumed after a restart.** No startup code resumes runs, and nothing wakes `delay` nodes (I-12).
2. **Agent tool calls are not checked against the agent's own tool list** (security, I-16).
3. **The published agent version is not what runs:** only its output schema is used (I-18).
4. **Forced air-gap can fall back to a cloud provider** when no local model is set (I-17).
5. **Resolving a graph approval from the shared inbox leaves the run stuck** (I-13).
6. **Webhook triggers ignore their own trigger rows:** any `agents.trigger` client can run any agent (I-14).
7. **The records and admin chat loop bypasses the gateway:** no budgets, fallback or DLP (I-15).

Each finding is logged in `program/RAID.md` (I-12 to I-23, T-02 to T-05) and routed to a work
package. Scope changes are written into the affected work packages (see *Scope changes* below).

## 1. Graph runtime: `graph_runtime_service` + `execution_graph`

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| Durable, checkpointed graph | ✅ | `core/src/services/graph_runtime_service.rs:1027-1076`; `core/src/repositories/graph_run_repo.rs:142-205,223` | After each node: `UPDATE ai_runs` (status, context, current node, step count) plus one `ai_run_nodes` row per attempt. Tables are in `core/src/db/migrations/0062_execution_graph_runtime.sql:27-158`. |
| Resumable after restart | ✗ | `graph_runtime_service.rs:1166-1196`; `core/src/repositories/graph_run_repo.rs:123-134` | `resume_run` refuses any run that is not waiting, so a run that crashed while `running` stays stuck. `list_resumable_runs` has **no callers**. Neither `src-tauri/src/lib.rs` nor `server/src/main.rs` nor `server/src/job_scheduler.rs` resumes anything. Only manual admin routes call `resume_run`. |
| `delay` nodes resume | ✗ | `graph_runtime_service.rs:1275`; only caller `core/tests/execution_graph_runtime.rs:701` | `resume_due_delays` is never scheduled in production, so a delayed run never continues. |
| 13 node types | ◐ **14** | `core/src/models/execution_graph.rs:15-40` | Types: trigger, condition, action, agent, router, parallel_split, join, approval, delay, transform, loop, end, **run_agent_team**, **evaluate_result**. They are strings, not an enum. The comment at 0062:59-60 still says 12. |
| Parallel branches run one at a time | ✅ | `graph_runtime_service.rs:661-686,937-996` (loop at :950) | Branches are awaited in turn. A branch may contain only single-exit node types (:962). A `delay` inside a branch is an error (:981). The whole split counts as one step with one checkpoint, and a failed branch leaves its partial context writes behind. Join modes: all, first_successful, n_of_m, timeout_partial (the last falls back to first_successful, :700-708). |
| No Manager, Debate, plan, selector, handoff, judge or mission node | ◐ | `execution_graph.rs:15-40`; `graph_runtime_service.rs:878-921` | None exist by name. However, **`evaluate_result` is already an LLM-as-judge pass/fail node** (it uses `ai_eval_service`'s judge prompt). The new `judge` node should extend it, not duplicate it. |
| No step-level budgets | ✅ | `graph_runtime_service.rs:101` `MAX_STEPS_DEFAULT = 50`, :1011 | The only limit is the step count. |
| `graph_from_pipeline` exists | ◐ | `core/src/services/execution_graph_service.rs:796`, `create_from_source` :1135 | Pipeline topologies map to graph shapes, but **only tests call it**; pipelines still run on `ai_orchestration_service`. `{{candidate_outputs}}` is left unresolved (documented gap). |
| Nested teams are safe | ⚠ | `graph_runtime_service.rs:750` | `run_agent_team` blocks only direct self-recursion. An A→B→A cycle is stopped only by each run's 50-step limit. |

## 2. Orchestration: `ai_orchestration_service`

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| Sequential, consensus and peer-review topologies | ✅ | `core/src/models/ai_agent_pipeline.rs:31`; `core/src/services/ai_orchestration_service.rs:790-830` | Anything unrecognised falls back to sequential. Consensus candidates also run one after another (:486). Peer review: `MAX_PEER_REVIEW_ROUNDS = 3` (:294); a reviewer approves with "APPROVED" (:289). |
| Schedule and webhook triggers | ⚠ | `ai_agent_pipeline.rs:12`; `ai_orchestration_service.rs:1013,1040`; `server/src/job_scheduler.rs:116-123`; `server/src/agent_v1.rs:31-93` | Schedules: the server ticks every 60s. **On desktop, due work drains only while the UI polls** (`src-tauri/src/commands/ai_agent_pipeline_commands.rs:297-335`). Webhooks run **inline** in the HTTP request and **never check the `ai_agent_triggers` row** (I-14). |
| Three older pause-for-human paths, not on `ai_approvals` | ◐ | see the list below | The code note this claim came from does not exist: `graph_runtime_service.rs:33-37` points to a comment that is not in `ai_orchestration_service.rs:1-19`. The real list is below. |

There are four pause paths. Two use the shared `ai_approvals` table (`0059_ai_agent_versioning.sql:113`, `core/src/services/approval_service.rs:37-82`) and two do not:

| Path | Uses `ai_approvals`? | Evidence | Behaviour |
|---|---|---|---|
| (a) Pipeline step `requires_approval`, in all three topologies | ✗ | `ai_orchestration_service.rs:449` (sequential), `:508` (consensus synthesizer), `:644` (peer reviewer) → `core/src/repositories/ai_agent_run_repo.rs:122` | Sets `ai_agent_runs.status = 'awaiting_approval'`. Resolved by `approve_pending_step` (:873) and `reject_pending_run` (:959). **These are the spec's "three paths".** |
| (b) Voice plans | ✗ | `core/src/services/voice_execution_service.rs:257-262,319` | `awaiting_confirmation` is resolved by `confirm_plan`. **No resolver was found for voice `awaiting_approval`.** |
| (c) Policy `RequireApproval` on a tool call | ✓, but no pause | `core/src/services/policy_engine_service.rs:13-23,119-135`; `core/src/services/chat_service.rs:1113-1126` | Writes an `ai_approvals` row and returns an error to the model. The run does not pause, and the tool is not run after approval. |
| (d) Graph `approval` node | ✓ | `graph_runtime_service.rs:564-582,1202` | Pauses and resumes properly, but see I-13: resolving it from the generic inbox command (`server/src/dispatch.rs:2730`) only updates the row, and the run stays stuck. |

## 3. Agent loop: `chat_service::run_agent_once_with_text`

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| The single agent loop | ◐ | `core/src/services/chat_service.rs:1766` wraps `run_agent_once` at :1788 | Graph agent nodes (`graph_runtime_service.rs:534`), pipeline steps (`ai_orchestration_service.rs:348`) and delegation (`chat_service.rs:1616`) all use it. **A second loop exists:** the records and admin chat assistants (`chat_service.rs:978-1140`) call `ai_service::complete_with_tools` directly, **bypassing the gateway** (I-15). |
| Firewall and audit on every call | ⚠ | firewall `chat_service.rs:1518` (`apply_tool_firewall` :1098); audit :1951 | The firewall runs before dispatch. **Dispatch does not check that the tool is in the agent's `action_names`** (`chat_service.rs:1675-1733`): any record, admin, connector or MCP tool name the model emits is dispatched (I-16). |
| Up to 8 tool rounds | ✅ | `chat_service.rs:65` `MAX_ROUNDS: u8 = 8` | Also used by the chat loop at :1006. |
| Delegation depth 4 | ✅ | `chat_service.rs:1259` `MAX_DELEGATION_DEPTH: u8 = 4`; guard :1254-1276 | Thread-local RAII guard. |
| No streaming trace events per round | ✅ | `chat_service.rs:1836-1972` | Messages build up in memory; nothing is emitted. |
| No partial-result state | ✅ | `chat_service.rs:1973`, :1903 | At the limit the call returns an error and discards the work done. Three identical tool calls in a row also abort. |
| No per-step cost cap | ✅ | `core/src/services/ai_gateway_service.rs:150,189` | Only daily token budgets exist. |
| (Not in spec) Output schema validation | ◐ exists | `core/src/models/ai_agent.rs:222`; `chat_service.rs:1858-1885` | **Already built:** final text is checked against the published version's `output_schema`, with one repair retry. The validator handles only a subset of JSON Schema (type, enum, …). |
| (Not in spec) Published version runs | ✗ | `chat_service.rs:1805-1816`; `core/src/repositories/ai_agent_version_repo.rs:138-148` | Only `output_schema` comes from the published version. Prompt, tools and routing come from the **live, editable** `ai_agents` row (I-18). |
| (Not in spec) Routing settings honoured | ✗ | `core/src/models/ai.rs:153-154`; `core/src/services/ai_service.rs:379,436,657,777` | `temperature` and `max_tokens` in model routing are never read. `max_tokens` is hard-coded to 2048 or 1024 (T-03). |

## 4. Firewall and policy: `policy_engine_service` + `tool_registry_service`

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| Six risk tiers | ✅ | `core/src/models/ai_tool_registry.rs:15-45` | Read, LowWrite, Write, ExternalAction, Destructive, Privileged (ordered). Default class comes from a few hard-coded names plus name prefixes (`core/src/services/tool_registry_service.rs:32-90`), with a per-workspace override. |
| Approval threshold and blocklist | ✅ | `core/src/models/ai_agent_policy.rs:21,24`; `policy_engine_service.rs:61-87` | **With no policy row, every call is allowed.** |
| Order schema → permission → policy → approval → execute → audit | ◐ | `chat_service.rs:1802,1518,1681,1951` | Actual order: admin gate at run start, then policy, then ontology scope (record tools only), then dispatch, then audit. **No tool-argument schema check runs before dispatch.** Most tool schemas are just `{"type":"object"}`. Approval does not pause the run (path (c) above). |
| No guardrails on outputs | ◐ | `core/src/services/dlp_service.rs:24,65`; `ai_gateway_service.rs:213` | Output schema validation exists (§3). DLP `redact()` has no callers (T-02); `scan()` runs only on the outbound prompt. `guardrails_md` is prompt text only. There is no moderation. |

## 5. Memory and knowledge

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| Session, working and entity memory | ✅ | `core/src/models/ai_memory.rs:9`; TTLs `core/src/services/ai_memory_service.rs:45,48` | Session 6h, working 1h, entity no expiry. The older per-agent `memory_md` blob still exists. The session key is the conversation id (`chat_service.rs:2022`), not what the doc comment at `ai_memory_service.rs:7-9` says. |
| Classifications | ✅ | `ai_memory.rs:11`; `core/src/models/semantic.rs:18`; `policy_engine_service.rs:102-114` | Standard, sensitive, restricted. Only `restricted` changes behaviour (excluded by default). |
| Knowledge Collections with citations | ✅ | `core/src/models/ai_knowledge.rs:8,56`; `core/src/services/ai_knowledge_service.rs:107,278-310` | Each hit carries source, chunk and similarity. |
| No composite recall ranking | ✅ | `core/src/repositories/ai_memory_repo.rs:75,97,116`; `core/src/services/vector_search_service.rs:112` | Memory is ordered by `created_at DESC` (session memory capped at 25). Knowledge uses brute-force cosine similarity. |
| No team-shared memory scope | ✅ | `ai_memory_repo.rs:64-75` | Entity memory is already shared across all agents in a workspace. There is no team scope. |

## 6. Evaluation: `ai_eval_service` and `test_eval_service`

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| Golden-case suites with an LLM judge, for agents and teams | ✅ | `core/src/models/ai_eval.rs:19-20`; `core/src/services/ai_eval_service.rs:136-165,285-297`; `core/src/models/test_eval.rs:34-42` | Evaluator types: task_completion, structured_output, policy_compliance. Targets: agent or pipeline. Test types include `agent_team_eval` (execution graphs). |
| No capture of a run into an eval case | ✅ | searched from_run, capture, promote, golden | Not found. |
| No regression gate on Publish | ◐ | `core/src/services/agent_version_service.rs:209-231` | **A narrow gate exists:** publishing an agent is blocked when the latest `policy_compliance` run has failures. `task_completion` never blocks, and pipelines, graphs and Solutions have no gate. Eval runs do not record which agent version ran (`src/features/settings/AiAgentsAdmin.tsx:866-880`). |
| Per-case cost | ✗ | `core/src/repositories/test_eval_repo.rs:213-214` | `cost_usd` is always NULL. `cost_threshold_usd` is stored but never read. |

## 7. Ontology, Object 360 and Glossary

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| Typed Object and Action Types | ◐ | `core/src/models/ontology.rs:16-51`; `core/src/models/action_type.rs`; `core/src/services/action_type_service.rs:332` | Object Types are a read model over built-in and custom objects, not a typed property schema. Action Types have typed parameters and the effects set_status, set_fields and reassign_owner. |
| Agent ontology binding | ✅ | `core/src/models/ai_agent.rs:18-21,90,97`; `chat_service.rs:173-200,1681`; migration `0077_ontology_agent_binding.sql` | `object_type_scopes` (empty means unscoped) and `action_type_ids`. Enforced for record tools only. |
| No context packs from the ontology | ✅ | `chat_service.rs:1458-1491` | The system prompt is persona, `memory_md`, session memory, guardrails and the skill list. No ontology or glossary is included. |

## 8. AI Gateway: `ai_gateway_service`

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| Bring-your-own-key | ✅ | `ai_gateway_service.rs:73-125`; `core/src/services/ai_provider_service.rs:60,99` | Workspace or admin keys, encrypted. No per-user keys. `resolve_tier` always needs a stored key, so keyless Ollama needs a dummy key. |
| Fallback chain | ✅ | `ai_gateway_service.rs:218-299` | Primary, then fallback, then local fallback, on any error. |
| Daily token budgets | ◐ | `ai_gateway_service.rs:150` (workspace), `:189` (agent); `core/src/repositories/ai_token_usage_repo.rs:66-69` | The user tier is recorded but **not enforced** (T-04). |
| Forced air-gap for sensitive data | ⚠ | `ai_gateway_service.rs:200-222` with `:106-124` | When DLP matches, only `local_fallback_provider_id` is tried. **If that is unset, `resolve_tier(None)` uses the workspace default, which may be a cloud provider.** Nothing checks that the provider is actually local (I-17). |
| No cost forecast or per-step routing | ✅ | graph agent nodes get `model_routing: None` (`graph_runtime_service.rs:515`) | Routing is set per agent only. |
| (BYOM) Providers | ◐ | `core/src/models/ai.rs:16`; `ai_service.rs:24-59,595-858` | `anthropic`, `openai_compatible` (OpenAI, Ollama, vLLM and others through `/chat/completions` with a Bearer key) and `google_gemini`, all with **native tool calling**. **No Azure OpenAI, Vertex AI or Bedrock.** Anthropic has no embeddings (`ai_service.rs:1243`). This confirms the scope of WP-0146. |

## 9. Live events: `server/src/events_stream.rs`

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| A live SSE channel to the browser | ◐ | `server/src/events_stream.rs:34-96` | `GET /api/v1/events/stream` authenticates with an **API-client Bearer key** (`events.read` scope), not the UI's session cookie. It **polls `integration_executions` every second** (no event bus) and emits only `execution` and `error` events. |
| Run events can flow through it | ✗ as is | grep `EventSource` and `events/stream` in `src/`, `app.js`, `site-modern-pages.js`: none | **Nothing consumes it.** The UI polls with react-query (`src/features/settings/AgentTeamsAdmin.tsx:356`). The desktop has no Tauri `emit`/`listen`. WP-0112 must add an in-process event bus, session-cookie auth and a Tauri channel. |

## 10. Server API, background work and the desktop bridge

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| `api_v1.rs`, `dispatch.rs` | ✅ | `server/src/api_v1.rs:41-52`; `server/src/dispatch.rs` behind `/api/invoke/:command` (`server/src/routes.rs:29`) | No `/api/v1/teams` or `/api/v1/runs`. The only AI `/api/v1` routes are the two webhook triggers (`server/src/agent_v1.rs:31-35`). Admin AI routes live under `/api/admin/...` (`server/src/admin_actions.rs:52-133`). There is also `/mcp` (`server/src/mcp.rs:53`). |
| `job_scheduler.rs` (to be extended with a worker pool) | ✅ | `server/src/job_scheduler.rs:59-143` | One OS thread with a single-threaded Tokio runtime, ticking every 60s; everything runs in sequence. No worker pool, no `resume_due_delays`, no resume sweep. |
| `ai_agent_pipeline_commands.rs` (to be replaced by `team_commands.rs`) | ✅ | `src-tauri/src/commands/ai_agent_pipeline_commands.rs` (18 commands) | There is no `team_commands.rs`. Team and graph commands live in `src-tauri/src/commands/execution_graph_commands.rs`. |
| `ai_runs`, `ai_run_nodes` (extended by Run and Step) | ✅ | `0062_execution_graph_runtime.sql:96-158` | Written only by `graph_run_repo`. Pipelines write to **separate** tables, `ai_agent_runs` and `ai_agent_run_steps` (`0040_ai_agent_orchestration.sql:72,87`). Neither has token or cost columns. Daily totals are in `ai_token_usage` (`0041_ai_gateway.sql:59-67`). |

## 11. UI and the online demo

| Spec claim | Verdict | Evidence | What the code does |
|---|---|---|---|
| `AgentTeamsAdmin.tsx` drag-and-drop canvas | ◐ | `src/features/settings/AgentTeamsAdmin.tsx:37-58,158,537-581`; `src/components/visualBuilder/useNodeDrag.ts:30-31` | It edits **Execution Graphs**. The canvas is custom (no library). Nodes are added by clicking and then moved by dragging; there is no drop from a palette. Live run highlighting polls every 2s. Pipelines have a separate form editor (`src/features/settings/AiAgentPipelinesAdmin.tsx`). |
| `AgentPlaygroundAndEval`, a basic single-agent playground | ✅ | `src/features/settings/AiAgentsAdmin.tsx:882-965`; `src/components/ChatPanel.tsx:30,118,150` | Chat with tool-call names, an eval suite list with a Run button, and a read-only versions table. Traces are shown only through a link. |
| No `src/features/playground/` | ✅ | `ls src/features` | Confirmed. |
| Online demo can host simulated runs | ✅, extensive | `app.js` 447-500 (seed), 11526-13642 (gateway, chat, agents, teams canvas with `waiting_approval`, MCP, guardrails, memory, versions, per-agent chat, skills, pipelines with an OTLP trace, evals with a simulated judge) | It is all one localStorage blob (`lanesra-os-demo-v10`, `app.js:117-130`). Replies come back instantly, keyword-matched, with no simulated delay. **The demo stores data in camelCase while the desktop uses snake_case**, so the "same Team format" the spec assumes does not exist yet (WP-0013). |
| Help articles to update | ✅ | `app.js:15044-15155` (`ai-agent-foundry` category) | 12 slugs, including `orchestration-pipelines`, `execution-graph-agent-teams`, `agent-versioning-and-approvals`, `policy-engine-tool-call-firewall` and `evaluation-harness`. There is no playground article yet. |

## 12. Editions: where desktop and Team Workspace differ

The frontend uses one transport (`src/lib/api.ts:297-366`): Tauri `invoke` on desktop, and on
the server `POST /api/invoke/<command>` (matched 1:1 through `server/src/dispatch.rs`) plus
`/api/admin/...` for long-running calls. Of the 492 Tauri commands, the 36 with no dispatch arm
are covered by admin routes or special-cased in `server/src/routes.rs`. Real gaps:

- **Server only:** `get_ai_eval_suite` (`server/src/dispatch.rs:3516`) has no Tauri command, so `api.getAiEvalSuite` (`src/lib/api.ts:602`) would fail on desktop (T-05). Also the webhook triggers, `/mcp` and the SSE stream.
- **Desktop only:** `drain_pending_async_work`. On desktop, schedules, pending runs and embeddings advance **only while the app is open and polling**, whereas the server ticks every 60s. Unattended work on desktop therefore needs a background ticker in the Tauri shell (WP-0141).
- **Identity:** desktop signs in with an Argon2 password and keeps the session user in memory (`src-tauri/src/commands/auth_commands.rs:10-14`); the server uses an HttpOnly session cookie (`server/src/session.rs:6-13`). There is no OIDC or SAML anywhere (searched oidc, saml, openid, jwks; OAuth2 appears only for outbound connections, `core/src/services/connection_service.rs:48-49`). This confirms WP-0101 to WP-0105.

## 13. Identity and access for agents and API clients

| Finding | Evidence | Consequence |
|---|---|---|
| Agents have no identity of their own | `core/src/models/ai_agent.rs:83` (`acts_as_user_id`, optional); `chat_service.rs:1205-1222` | Used for access checks only when the policy sets `enforce_record_access` (default off). Otherwise agent writes run with `actor = None`. |
| `actor = None` skips Access Control | `core/src/services/access_service.rs:336-344`; `server/src/api_v1.rs:281,298,314` | AI writes and API-client object writes are **unattributed and unchecked by default** (I-20). |
| Triggered runs act as the API client's owner | `server/src/agent_v1.rs:77` | Confirms the RAID finding behind WP-0100 (unattended runs inherit the owner's rights). |
| API client auth | `core/src/services/api_client_service.rs:17-32,171-200` | `Bearer {client_id}.{secret}`, with only a SHA-256 hash of the secret stored. 11 scopes, including `agents.trigger` and `events.read`. |

## 14. Other spec assumptions

| Claim | Verdict | Evidence | Note |
|---|---|---|---|
| Draft → Test → Published → Deprecated → Disabled lifecycle | ◐ | `core/src/services/agent_version_service.rs:1-15,173-182`; `core/src/models/execution_graph.rs:13`; `ai_agent_pipeline.rs:69` | Agents have it (as strings, with an allowed-transitions table). Graphs have only draft, published and disabled. Pipelines have only `is_active`. |
| `run_ai_agent_pipeline` admin tool exists | ✅ | `chat_service.rs:334,434,851-866`; risk Privileged `tool_registry_service.rs:41` | Calls `ai_orchestration_service::run_manual`. |
| Pipelines inside Solution exports keep working | ✗ | `core/src/services/industry_package_service.rs:402-658` | Solution exports contain custom objects, fields, relationships, rules, workflows, layouts, reports, skills and agents. **No pipelines and no execution graphs**, so Agent Teams cannot be shipped in a Solution today (I-22). |
| `THIRD_PARTY_NOTICES` exists | ✗ | `git ls-files` | Only `LICENSE` (already known; WP-0012 and the finish checklist). |

## Scope changes written into work packages (AC-3)

| Work package | Change |
|---|---|
| WP-0100 Principal model and Agent Identities | Must replace `actor = None` defaults for AI and API-client writes (I-20). The webhook trigger must check its trigger row and run as that trigger's Agent Identity (I-14). |
| WP-0112 Run events over SSE | No usable event channel exists: build an in-process event bus (not table polling), session-cookie auth for the UI, a Tauri channel for desktop, and the first frontend consumer. |
| WP-0121 Step limits and partial results | Hitting the limit currently returns an error and discards `produced`; keep the partial output. Also make `temperature` and `max_tokens` from routing take effect (T-03). |
| WP-0123 Per-step routing and data boundaries | Air-gap must fail closed when no local provider is set, and check that the provider is actually local (I-17). The records and admin chat loop must go through the gateway (I-15). |
| WP-0132 Output validation | Extend the existing output-schema check (`chat_service.rs:1858-1885`) rather than build a second one. Wire up or remove DLP `redact()` (T-02). |
| WP-0141 Idempotency and side-effect-safe resume | Add the missing startup sweep (`list_resumable_runs`), a `resume_due_delays` tick on server and desktop, a recovery rule for runs that crashed while `running`, and a desktop background ticker (I-12). |
| WP-0143 Tool catalog and delegation guard audit | Enforce `action_names` membership before dispatch, with a test proving an agent cannot call an unlisted (including admin) tool (I-16). Detect A→B→A team cycles. |
| WP-0146 Bring-your-own-model coverage | Confirmed: Azure OpenAI, Vertex AI and Bedrock are missing. Ollama must work without a key. |
| WP-0150 Move every pause onto `ai_approvals` | The legacy paths are (a) pipeline `requires_approval` in three topologies and (b) voice plans. Fix the stuck graph run when it is resolved from the inbox (I-13). Make policy `RequireApproval` pause and resume rather than drop the call. |
| WP-0170 (new, proposed) Run the published agent version | Runs must execute the published snapshot (prompt, tools, routing), not the live editable row (I-18). |

Later phases (noted in RAID for their refinement): the new judge node should extend
`evaluate_result` rather than duplicate it. The Phase 3 pipeline retirement (WP-0306) must add
pipelines and graphs to Solution exports (I-22). The online demo's data needs a shared, versioned
format with desktop before "same Team format" holds (WP-0013).

## Method

- Three read-only search passes ran in parallel (runtime; governance; UI and editions), each told to cite `path:line` and to report NOT FOUND rather than guess.
- These claims were re-read by hand: 14 node types, `MAX_ROUNDS`, `MAX_DELEGATION_DEPTH`, the uncalled resume and delay sweeps, `resume_run` refusing `running`, the generic `resolve_ai_approval`, webhook auth, the provider list, the published-version read, the air-gap fallback, missing tool-list enforcement, `get_ai_eval_suite`, the Solution export contents and the missing SSE consumers.
- Not verified at run time: whether services below `dispatch_admin_tool` reject a non-admin actor. I-16 is therefore logged as high priority to confirm with a test in WP-0143, not as a proven exploit.
- The audit makes no code changes.
