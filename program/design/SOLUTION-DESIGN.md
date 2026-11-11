# Lanesra OS — Solution design

The complete design of the Agent Foundry and Orchestration platform: what the October spec already
defines (the baseline), and what the platform extension of 2026-11-11 adds (marked [NEW], decided
in ADR-0020). The October spec (`spec/agent-playground-design-spec-v1.html`) stays the source for
baseline detail; this document is the source for the extension and for how the parts fit together.

Table and endpoint names for [NEW] parts are proposals. The design work packages (WP-0114,
WP-0147) and each lane's first work package fix them in ADRs before code is written.

## 1. Goals and principles

Lanesra OS is a self-hosted platform where an organisation builds, tests, runs and governs AI
agents and teams of agents on its own business data. It ships as a desktop app, a Team Workspace
server and an online demo from one Rust core.

- **Governed by default.** Every tool call goes through one firewall; every run is traced, budgeted, replayable and attributable to a person or an Agent Identity.
- **One runtime.** Every orchestration mode compiles to one durable execution graph. There is one checkpoint, approval and audit path.
- **Grounded in the business model.** Agents read the Ontology, Object 360, the Glossary and record-level access, not raw text.
- **No-code first, code welcome.** Business users build in the Playground; developers use the API, CLI, SDKs and Teams as code. [NEW]
- **Bring your own model.** Any configured provider, including fully local ones. Sensitive data is forced to local models.
- **Govern what you did not build.** Agents built on other frameworks are registered, called and observed under the same controls. [NEW]
- **Clean room.** Ideas are re-expressed; no third-party code, prompts or structure is copied.

## 2. Architecture

The platform is ten layers over one core. Items marked with a star are added by the extension.

```arch
Experience :: Agent Studio; Playground; Team canvas; Approvals inbox; Work queue*; Fleet view*; Compliance centre*
Channels and access :: REST API; MCP server; CLI; Team as an endpoint; A2A server*; MCP gateway*; Slack*; Microsoft Teams*; Email*; Web widget*; Python and TypeScript SDKs*
Control plane :: Lifecycle and versions; Publish gate; Budgets; Kill switch; Feature flags; External agent registry*; Promotion between environments*; Alerts and SLOs*
Orchestration :: Mode compiler; Run controller; Graph runtime; Checkpoints; Replay and fork; Human seats*; A2A client seats*
Agent runtime :: Agent loop; Context assembly; Reasoning modes; Limits and partial results; Compaction; Policy hooks
Model gateway :: Provider adapters; Model catalog; Fallback chain; Data boundaries; Forced air-gap; Right-sizing report*
Tool plane :: Tool catalog; Firewall; Connectors; MCP client; Sandbox; Token vault*; MCP registry and pinning*; Signed custom tools*
Knowledge and memory :: Memory scopes; Composite recall; Knowledge Collections; Citations; Hybrid search and rerank*; Graph retrieval*; Permission-aware sync*; Ask the data*
Evaluation and improvement :: Golden cases; Graders; Recorded runs; Shadow and A/B; Simulated users*; Annotation queues*; Red-team suite*; Optimizer*
Governance and operations :: Identity and SSO; Policies and guardrails; Approvals; Hash-chained audit; OTLP export; SCIM*; External KMS*; SIEM export*; Trace ingest*; Compliance pack*; Helm and HA*
```

### Editions

| Edition | What runs | Notes |
|---|---|---|
| Desktop | Tauri shell, Rust core, SQLite | Fully offline with a local model. No inbound endpoints, so channels, the MCP gateway, trace ingest and SCIM do not apply. |
| Team Workspace | Rust server, same core, SQLite or PostgreSQL, worker pool | Hosts several workspaces. All extension features ship here. |
| Online demo | Single-file browser app on sample data | Write tools always simulated. New screens appear once both editions have released them. |

## 3. Core model

| Object | Purpose | State |
|---|---|---|
| Agent | A persona, instructions, tools, ontology scope, memory scopes and limits, versioned | Baseline |
| Team | Agents in seats under a mode, with budget, policy and shared memory | Baseline |
| Seat | A role in a Team that tasks are assigned to. Filled by an agent, a nested Team, a person or Work Team [NEW], or an external agent [NEW] | Extended |
| Task | A contract: instruction, expected output, output schema, context bindings, guardrails, retry budget, approval gate | Baseline |
| Playbook | A graph of tasks and control nodes with triggers and an inputs schema | Baseline |
| Run and Step | One execution and its units, with attempts, tokens, cost, verdicts and timings | Baseline |
| External agent | A registered agent built elsewhere: endpoint, protocol, owner, risk tier, data classes | [NEW] |
| Channel and Conversation | A binding of a Team to an outside surface, and one session on it | [NEW] |
| Grant | A delegated credential a user or Agent Identity holds for a connector or MCP server | [NEW] |
| Work item | A task waiting on a human seat | [NEW] |
| Proposal | An optimizer suggestion with its diff, evidence and scores | [NEW] |
| Control mapping | A link from a platform control to a framework obligation and its evidence | [NEW] |

Agents, Teams and Playbooks follow Draft → Test → Published → Deprecated → Disabled. A published
version is immutable. External agents follow Registered → Approved → Paused → Retired.

## 4. Orchestration and the run lifecycle

Every mode (Sequential, Consensus, Peer review, Manager-led, Round Table, Handoff, Debate, Graph,
Event Playbooks) is compiled by the mode compiler into one execution graph. The run controller
admits, starts, pauses, budgets, replays and forks runs. The graph runtime walks nodes, commits
parallel branches together and writes checkpoints.

A run moves through `queued → running → (waiting_approval | waiting_budget | waiting_model | waiting_human [NEW]) → completed | completed_partial | failed | cancelled`.

- **Durability.** Checkpoints per step (or relaxed per Team). Write tools carry an idempotency key, so a resume never repeats a write.
- **Budget lease.** A run holds a lease; exhausting it pauses the run instead of failing it.
- **Replay and fork.** Replay re-runs a step and everything after it. Fork copies a run up to a step into the Playground. Outputs are immutable.
- **Human seats [NEW].** A step on a human seat creates a work item and the run enters `waiting_human`. The submitted result is validated against the output schema and guardrails like an agent's output. This reuses the approvals inbox's durable wait and routing (owner, Org Unit, Work Team, out-of-office), and adds claim, reassign and return.
- **External seats [NEW].** A step on an external agent is a tool-plane call over A2A (section 8). It appears in the trace as a child step and counts against the same budgets and limits.

## 5. Agent runtime loop

One loop runs every agent step:

- Assemble context from explicit bindings, each trimmed to its token budget, and record a context receipt.
- Call the model through the gateway, with per-step routing, a fallback chain and data boundaries.
- Run tool calls through the firewall.
- Validate the output against the schema, then the guardrails; return failures to the agent within its retry budget.
- Persist the output, memory updates and audit record.

Limits end a step cleanly: a tool-round or token limit asks once for a final answer and marks the
step `completed_partial`; a wall-clock limit cancels the call and retries once on the fallback
model. Policies attach at three hook points: around the step, each model call and each tool call.

The extension adds one binding kind, `related:` (graph retrieval, section 10), and no new loop.

## 6. Tool plane, delegated credentials and the MCP gateway

Every tool, whatever its source (native, connector, MCP, team, external agent [NEW], custom
[NEW]), has a risk tier and passes one pipeline:

`schema → permission → policy → approval → execute → audit`

Baseline rules: tools never see credentials; rate limits apply per tool, connector and workspace;
results from outside sources are fenced as untrusted and cannot change instructions or the tool list.

### Delegated credentials [NEW] (WP-0147, WP-0510, WP-0511)

Today a connector holds one shared credential. The extension lets a call act as the person.

- **Grant.** A user or an Agent Identity connects an account with OAuth 2.1 (authorisation code with PKCE). The token vault stores tokens encrypted under the workspace data key. Tokens are never shown again and never enter a model's context, a trace or a checkpoint.
- **Selection.** The connector layer picks the grant from the run's effective identity.

| Run kind | Connector mode | Credential used |
|---|---|---|
| Interactive | Delegated | The starting user's grant; refused with "connect your account" if there is none |
| Unattended | Delegated | The Agent Identity's own grant; refused if there is none |
| Any | Shared | The connector's credential, as today |
| Any | Another user's grant | Never, including through delegation, handoff or nested Teams |

- **Lifecycle.** The vault refreshes before expiry, rotates and revokes. Users see My connections; admins see all grants but cannot read or use them. Deactivating a user (SCIM) revokes their grants.

### MCP registry and gateway [NEW] (WP-0512, WP-0513)

- **Registry.** Only approved MCP servers are callable. Each tool definition is pinned by hash. A changed definition is quarantined until an admin approves the diff.
- **Scan.** Tool descriptions are scanned for instruction-like content at registration and on change; a finding blocks approval by default.
- **Gateway.** One Lanesra endpoint offers approved tools from several servers to outside MCP clients, with namespaced names, scoped keys, the firewall, rate limits and audit.

### Signed custom tools [NEW] (WP-0574)

A builder packages a Python tool with a manifest: schema, risk tier, data classes, network needs.
It runs only in the sandbox, only when signed by a publisher the workspace trusts, with no access
to the database, the data key or any credential.

## 7. Identity and access

| Principal | Signs in with | Acts with |
|---|---|---|
| User | OIDC or SAML, or a local account | Access Role and Record Scope |
| Agent Identity | Short-lived, scoped, platform-issued credential | Its own Access Role and Record Scope |
| API client | Scoped key, optional network restriction | Granted scopes |
| Channel user [NEW] | Mapped from the channel's directory identity | The mapped user's rights, or the guest policy |
| External agent [NEW] | Mutual authentication fixed by WP-0114 | A bound Agent Identity; never more than its registry entry allows |

An interactive run acts as the starting user intersected with the agent's ontology scope: an agent
never sees more than its user. An unattended run acts as its Agent Identity. Every step records
both the agent and the effective identity.

SCIM 2.0 [NEW] (WP-0580) creates, updates and deactivates users and groups from the directory.
Deactivation ends sessions, revokes grants and reassigns open approvals and work items.

## 8. Agent control plane [NEW]

The aim is one inventory, one policy set and one run history for every agent an organisation runs,
whatever built it. This is the lead differentiator of the extension.

| Part | What it does | Work package |
|---|---|---|
| Registry | Records external agents with owner, framework, endpoint, protocol, data classes and risk tier. An AI Admin approves before anything can call them. | WP-0501 |
| A2A server | Each published Team has an agent card and an A2A endpoint. Tasks map to runs; approvals surface to the caller as input required. | WP-0502 |
| A2A client | An approved external agent fills a seat or is bound as a tool. Calls pass the firewall; replies are fenced as untrusted. | WP-0503 |
| Trace ingest | An OTLP receiver maps incoming spans onto run → step → round → model and tool calls. Ingested runs are read-only and marked externally reported. | WP-0504 |
| Fleet governance | Policies, data boundaries, budgets and the kill switch cover external agents. The fleet view shows every agent with state, cost, success rate and findings. | WP-0505 |

**Trust levels.** An external agent is one of three kinds, and the fleet view shows which:

| Kind | Lanesra can | Lanesra cannot |
|---|---|---|
| Called (A2A client) | Refuse, budget, pause, audit every call | See inside the agent's own steps unless it also sends traces |
| Observed (trace ingest) | Show runs and cost, raise findings and alerts | Stop the agent; numbers are self-reported |
| Called and observed | Both of the above, joined by a correlation id | — |

**Deny paths.** A call is refused, with a readable reason and without failing the run, when the
endpoint is not registered, the agent is not approved, it is paused, or its budget is spent.

## 9. Channels [NEW]

A channel binds a published Team or agent to an outside surface. All channels share one framework
(WP-0520); each surface is an adapter.

- **Conversation.** A session with its own history, linked runs and stop rules. It appears in run history like any run.
- **Identity.** A channel user maps to a Lanesra principal through the directory. An unmapped sender gets the channel's guest policy, which allows nothing by default.
- **Inbound safety.** Every inbound request is signature-verified and replay-protected. Message bodies and attachments are untrusted content.
- **Adapters.** Slack (WP-0521), Microsoft Teams (WP-0522), email (WP-0523), embeddable web widget with allowed origins and signed visitor tokens (WP-0524).
- **Approvals in channels** (WP-0525). An approver gives any of the four review responses from a card. The decision is bound to the exact arguments and the mapped identity. The four-eyes rule still applies.

Interactive output cards (record card, form, table, chart) render through the adapter as native
blocks; their actions use the normal write path.

## 10. Knowledge and memory

Baseline: Session, Working, Entity, run-board and team memory scopes, each with TTL and policy;
composite recall by semantic match, recency and importance; Knowledge Collections with citations;
every memory write records its source step; erasure cascades.

The extension grounds retrieval in the business model:

| Capability | Design | Work package |
|---|---|---|
| Hybrid search and rerank | Fuse full-text and vector results, rerank with a configured model (local allowed), record score parts in the context receipt, fall back to full text | WP-0550 |
| Graph retrieval | A `related:` binding expands from a bound record along Link Types within hop and row limits, returning cited records under record-level access | WP-0551 |
| Permission-aware sync | A Knowledge Source syncs from a connector on a schedule and keeps each document's source permissions; a chunk is returned only to an identity allowed at the source | WP-0552 |
| Ask the data | A tool answers numeric questions only through Glossary Metric definitions, cites the metric version and honours Record Scope; no free-form SQL | WP-0553 |

## 11. Evaluation and improvement

Baseline: golden cases, graders (schema, LLM judge, tool path, cost and latency), recorded runs
replayed at no model cost, the publish gate (blocked below the threshold), impact analysis, online
sampling, A/B and shadow mode.

The extension closes the loop from production back to a better version:

```arch
Observe :: Online sampling; End-user feedback*; Alerts*
Review :: Annotation queues*; Judge agreement report*
Build cases :: Save as test case; Synthetic cases*; Simulated users*; Red-team probes*
Improve :: Optimizer proposals*; Model right-sizing*
Gate :: Suite score; Attack success rate*; Shadow run; Publish
```

- **Simulated users** (WP-0530). A scripted or model-played persona with a goal holds a multi-turn conversation; graders score the whole conversation. Turn and cost caps apply. Recordings make results repeatable.
- **Synthetic cases** (WP-0531). Draft cases from a knowledge collection, sample records or traces. A person accepts each one. Restricted data is never a source.
- **Annotation queues and feedback** (WP-0532). Sampled, flagged or thumbs-down runs enter a queue. Reviewer scores calibrate the LLM judges, and the agreement rate is reported. A corrected run becomes a golden case.
- **Red-team suite** (WP-0533). A versioned probe library (prompt injection, data exfiltration, privilege escalation, unsafe tool use, jailbreak), run with writes simulated. Attack success rate is a publish-gate metric.
- **Optimizer** (WP-0540, WP-0541). Failing cases produce proposed changes to instructions, examples, tool descriptions or reasoning mode, each as a diff with evidence. A proposal is re-scored on the suite and on a held-out set, applied only as a draft version on the owner's approval, and never published automatically. A right-sizing report names the cheapest model per step that still passes.

## 12. Developer surface [NEW]

| Part | Design | Work package |
|---|---|---|
| Teams as code | A Team with its agents, tasks, playbook, policies and suites exports to a declarative, versioned document with a published schema. Import validates, shows a diff and applies nothing until confirmed. Secrets are never exported. | WP-0570 |
| Promotion | Workspaces carry an environment label. A promotion from Git or the CLI runs the suite at the target and records who promoted which version. | WP-0571 |
| SDKs | Python and TypeScript clients generated from the API description: agents, Teams, runs, streamed events, approvals, evaluations. They are clients of the public API; there is no in-process agent framework. | WP-0572, WP-0573 |
| Custom tools | Section 6. | WP-0574 |

## 13. Governance and compliance

Baseline: roles (Viewer, Builder, Approver, AI Admin, Team Owner, Auditor); guardrails as versioned
policy objects; data boundaries; approvals with four responses, bound to exact arguments, with
standing approvals, SLA and escalation; the four-eyes rule; kill switch; hash-chained audit;
retention and erasure; signed evidence export; content labels.

### AI compliance pack [NEW]

- **AI inventory** (WP-0590). Every agent, Team, model, tool and external agent with owner, purpose, data classes, risk tier and versions. Each published version has a generated agent card.
- **Risk classification** (WP-0591). A questionnaire assigns a risk class. High-risk classes require named controls before Publish: human oversight, logging, evaluation thresholds.
- **Control mappings** (WP-0591). Platform controls map to EU AI Act obligations, ISO/IEC 42001 and the NIST AI RMF as versioned data. Each row links to live evidence. The mappings are aids, not legal advice.
- **Compliance report** (WP-0592). A signed bundle for an agent, a Team or the workspace, listing controls, evidence and gaps.

## 14. Observability and operations

Baseline: span tree run → step → round → model and tool calls; live events over SSE; OTLP export;
orchestration dashboard; cost reports; outcome metrics; backups, key recovery, upgrades and health
endpoints; several workspaces per server; PostgreSQL; worker pool; remote workers.

| Extension | Design | Work package |
|---|---|---|
| Alerts and SLOs | Rules on success, latency, cost, approval wait, guardrail failures and tool denials per Team; routes to email, webhooks and channels; de-duplication and quiet hours | WP-0583 |
| SIEM export | Audit and security events over syslog and HTTPS with a documented schema, at least once, with gap detection from the hash chain | WP-0582 |
| External KMS | The master key can live in AWS KMS, Azure Key Vault, Google Cloud KMS or HashiCorp Vault; connector secrets can come from a secrets manager; rotation without downtime; a provider outage fails closed | WP-0581 |
| Kubernetes | A Helm chart for server, workers, PostgreSQL and ingress, linted and smoke-tested in CI | WP-0584 |
| High availability | Stateless replicas; run ownership by lease with takeover from the last checkpoint; a restore drill against documented targets | WP-0585 |

## 15. Data model additions

All follow the existing conventions: UUID ids, `workspace_id` on every row, audit columns, JSON
config columns, migrations numbered by ADR-0003. Names are proposals.

| Table | Holds |
|---|---|
| `ai_external_agents` | Registry entries, state, risk tier, bound Agent Identity |
| `ai_external_agent_calls` | One row per A2A call, linked to the run step |
| `ai_grants` | Grant owner, connector or MCP server, scopes, expiry, state; token ciphertext in the vault |
| `ai_mcp_servers`, `ai_mcp_tool_pins` | Approved servers and pinned tool definition hashes |
| `ai_channels`, `ai_conversations`, `ai_conversation_messages` | Channel bindings, sessions and messages |
| `ai_work_items` | Tasks waiting on human seats, with claim and SLA fields |
| `ai_eval_personas`, `ai_eval_annotations`, `ai_feedback` | Simulated users, reviewer scores, end-user feedback |
| `ai_red_team_probes`, `ai_red_team_results` | Probe library versions and results |
| `ai_optimizer_proposals` | Proposal diff, evidence cases, scores, decision |
| `ai_alert_rules`, `ai_alert_events` | Rules and fired alerts |
| `ai_risk_assessments`, `ai_control_mappings` | Risk class per agent or Team; control mapping data |
| `ai_promotions` | Who promoted which version between which environments |

## 16. API and event additions

| Surface | Additions |
|---|---|
| REST | External agents; grants (list, revoke); MCP servers; channels and conversations; work items; personas, annotation queues, feedback; red-team runs; optimizer proposals; alert rules; inventory and compliance reports; import, export and promotion |
| Protocols | A2A server per published Team; A2A client; OTLP receiver; MCP gateway; SCIM 2.0 |
| Events | `external_agent.called`, `external_agent.denied`, `grant.used`, `grant.revoked`, `mcp.tool_quarantined`, `conversation.started`, `work_item.created`, `work_item.submitted`, `alert.fired`, `proposal.created` |
| CLI | `lanesra team export`, `team import`, `promote`, `fleet list`, `grants list` |

Every new endpoint sits behind its own scope with an allow test and a deny test, as ADR-0018 requires.

## 17. Security

New surfaces and the control that covers each:

| Threat | Control |
|---|---|
| An external agent returns instructions that steer a Team | Replies are untrusted content; they cannot add tools or change instructions; write-tier actions based on them follow the untrusted-content rule |
| A forged or replayed channel message | Signature and timestamp verification per adapter; guest policy denies by default |
| Token theft from the vault | Encryption under the workspace data key (optionally an external KMS); tokens never leave the connector layer; audit on every use |
| Confused deputy: a run uses someone else's access | Grant selection only from the effective identity; a test per row of the decision table |
| An MCP server changes a tool after approval | Hash pinning and quarantine; description scan |
| Malicious custom tool | Publisher signing and trust list; sandbox with declared network only |
| Poisoned ingested traces | Per-agent keys, size and rate limits; ingested runs are read-only and labelled self-reported |
| Optimizer drifts an agent into unsafe behaviour | Proposals are drafts; the publish gate, including the red-team threshold, still applies |

The G5 gate includes a security review of all of these (WP-0599).

## 18. Build sequence and workload

Phase 5 runs beside Phase 4 once the G3 security gate has passed. Each lane starts when its entry
dependencies are done. The two design packages run now, in Phase 1.

| Order | Lane | Work packages | Enters after | Planned for |
|---|---|---|---|---|
| Now | Design ADRs | WP-0114, WP-0147 | Agent Identities (done), trace spans (done), tool audit (done) | Muse |
| 1 | Delegated credentials and MCP gateway | WP-0510 → WP-0511; WP-0512 → WP-0513 | OIDC sign-in, MCP request gate | Codex |
| 1 | Enterprise operations | WP-0580 to WP-0585 | Directory mapping, audit log, PostgreSQL, worker pool | Codex; Muse for the Helm chart |
| 1 | Evaluation depth | WP-0530 to WP-0533 | Deeper evaluation, publish gate, content labels | Codex and Muse |
| 2 | Agent control plane | WP-0501 → WP-0503, WP-0504 → WP-0505; WP-0502 | Teams, handoff, run APIs, OTLP export, kill switch | Codex |
| 2 | Knowledge | WP-0550 → WP-0551, WP-0552; WP-0553 | Composite recall, outcome metrics, token vault | Codex |
| 2 | Humans as seats | WP-0560 → WP-0561 | Teams, approval routing | Codex, then Muse |
| 2 | Developer surface | WP-0570 → WP-0571; WP-0572 → WP-0573; WP-0574 | Run APIs, sandbox | Muse; Codex for the Python SDK and custom tools |
| 3 | Channels | WP-0520 → WP-0521 to WP-0524 → WP-0525 | Team as an endpoint, approvals inbox | Claude; Muse for email and widget |
| 3 | Optimizer | WP-0540 → WP-0541 | Deeper evaluation, model catalog | Codex, then Muse |
| 3 | AI compliance | WP-0590 → WP-0591 → WP-0592 | Registry, audit log | Muse |
| Last | Gate G5 | WP-0599 | The last package of each security-relevant lane | Claude |

Workload at the time of writing: Codex 34 packages and 120 points (including sign-in, provider
coverage and toolchain upgrades from the existing plan); Muse AI 16 packages and 40 points; Claude
Code keeps the critical path of Phases 1 to 4 plus 5 packages in Phase 5. Live numbers are in
`STATUS.md`. The reasoning for the split is in ADR-0021.

## 19. Deferred

Not planned; each needs its own ADR before it enters the plan.

- Multimodal and voice agents (image, PDF and audio inputs; realtime voice)
- Gateway cost and quality routing, semantic and prompt caching, key pools
- Computer-use and browser tools
- A signed public hub for templates, tools and MCP servers
- Policy as code with what-if simulation
- Event-stream triggers (queues and streams)

## 20. Open questions for the owner

- RAID Q-27: how Codex works on this repo (shell, network, who merges).
- RAID Q-28: confirm the ten areas and the six deferred ones.
- Should the first external-agent protocol be A2A only, or A2A plus a plain HTTP adapter for agents that do not speak it? (decided in WP-0114)
- Which key management services are required for the first release of WP-0581: all four, or the one the first customer uses?
