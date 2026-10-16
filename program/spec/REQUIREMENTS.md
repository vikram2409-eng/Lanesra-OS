# Requirements register

Every commitment in the [Agent Playground design spec](agent-playground-design-spec-v1.html)
(Oct 14, 2026) as one line with a stable ID. Work packages cite these IDs; tests cite them in a
comment (`// Covers: REQ-LOOP-004`); `programctl validate` fails if any ID is not covered by a work
package, and `programctl trace` shows which tests prove each one.

**Format (parsed by tooling, keep exactly):** `- **REQ-AREA-NNN** [P<phase>] text`

**Phase source:** the spec's text and its roadmap diagram ("Four phases, each closed by a gate").
Where the spec gives no phase, the plan assigned one and the line ends with `(phase: plan)`.
Changing a phase or a requirement's meaning needs an ADR in `program/decisions/`. Never renumber an
ID; retire it by striking the text and adding `RETIRED (ADR-xxxx)`.

Precursor requirements (`[P0]`) are not in the spec. They are the engineering preconditions the
plan adds so the build can be done to enterprise standard by several AI agents in turn.

---

## PRE — Precursors (added by the plan)

- **REQ-PRE-001** [P0] Every pull request runs an automated gate: Rust format, lint and tests for core, server and cli; a Tauri build check; TypeScript typecheck, unit tests and production build; `node --check app.js`; program validation
- **REQ-PRE-002** [P0] Dependency lockfiles are in sync and toolchains are pinned so a clean install (`npm ci`, `cargo build --locked`) works on every machine and CI runner
- **REQ-PRE-003** [P0] A deterministic, scripted model provider lets every agent, pipeline and orchestration test run without network access or API keys
- **REQ-PRE-004** [P0] A browser end-to-end test harness drives the desktop UI (server mode) and the online demo
- **REQ-PRE-005** [P0] Database upgrade tests prove every new migration applies cleanly to a database created by the last released version, with no data loss
- **REQ-PRE-006** [P0] New capabilities ship behind workspace feature flags so `main` stays releasable throughout the multi-month build
- **REQ-PRE-007** [P0] Cross-cutting designs that many work packages depend on (run and step state machine, trace event schema, error taxonomy, prompt catalog format, migration conventions) are fixed in ADRs before Phase 1 work starts
- **REQ-PRE-008** [P0] Every "existing piece" the spec relies on is verified against the code with file references; discrepancies are recorded and resolved
- **REQ-PRE-009** [P0] A threat model for agent orchestration (including prompt injection, excessive agency and data exfiltration) exists before Phase 1 and is updated each phase; dependency and secret scanning run in CI
- **REQ-PRE-010** [P0] A decision on how the online demo (single-file `app.js`) shares the Team format and runs simulated Playground runs is recorded before Phase 2
- **REQ-PRE-011** [P0] Design tokens and components needed by the mockups are inventoried against the desktop UI kit, and automated accessibility checks run in tests
- **REQ-PRE-012** [P0] A performance baseline (agent loop overhead, graph runtime step latency, SQLite growth per run) is measured so later phases can be held to budgets
- **REQ-PRE-013** [P0] Work packages are mirrored one-way to GitHub Issues with phase milestones and status labels; the repo files stay the source of truth (owner decision, ADR-0004)
- **REQ-PRE-014** [P0] An AI that cannot run a shell or git (Muse AI) can deliver a change through a GitHub Action that applies it to a `wp/` branch and runs the same validation and CI as any other change (owner decision, ADR-0004)

## CLEAN — Clean-room approach

- **REQ-CLEAN-001** [P0] Analysts and implementers are separate roles; implementers never open CrewAI, Agent Framework, LangGraph or AutoGen source while writing Lanesra code
- **REQ-CLEAN-002** [P0] All system prompts, manager personas and tool formats are written fresh; no third-party prompt text is reused
- **REQ-CLEAN-003** [P0] Lanesra vocabulary only (Team, Playbook, Run, Step); no third-party names or structure mirrored
- **REQ-CLEAN-004** [P0] A `THIRD_PARTY_NOTICES` file records any third-party code pulled in as a dependency
- **REQ-CLEAN-005** [P0] Every orchestration pull request carries the clean-room checklist: no copied prompt, no transliterated function, design traced to the spec

## CORE — Core concept model

- **REQ-CORE-001** [P1] Agent gains persona fields (role, goal, background), limits and reasoning_mode (phase: plan)
- **REQ-CORE-002** [P1] Persona, instructions, guardrails, memory digest and ontology context compile into the system prompt, shown read-only (phase: plan)
- **REQ-CORE-003** [P1] Task is a contract: instruction, expected output, optional output JSON Schema, context bindings, guardrails, retry budget, assignee, approval gate
- **REQ-CORE-004** [P1] Context bindings (`step:`, `record:`, `knowledge:`, `memory:`) each carry a token budget; nothing is passed implicitly
- **REQ-CORE-005** [P3] Team: name, mode, members (agent + seat), optional manager, shared memory scope, budget, policy
- **REQ-CORE-006** [P3] Seats: tasks are assigned to seats, not agents; swapping a seat's agent keeps the playbook intact
- **REQ-CORE-007** [P3] Playbook: graph of tasks and control nodes, triggers, inputs schema, version (execution_graph source kind `playbook`)
- **REQ-CORE-008** [P1] Run extends ai_runs and Step extends ai_run_nodes with attempt, tokens, cost, guardrail verdicts and timings (phase: plan)
- **REQ-CORE-009** [P3] Agents, Teams and Playbooks follow the Draft → Test → Published → Deprecated → Disabled lifecycle

## ORCH — Orchestration engine

- **REQ-ORCH-001** [P3] Every mode compiles to one execution graph run by the existing graph runtime (one durability, checkpoint, approval and audit path)
- **REQ-ORCH-002** [P3] Mode compiler in `core/src/services/orchestration/compiler.rs`, one file per mode, cached per Team version (phase: plan)
- **REQ-ORCH-003** [P3] Run controller owns admit, start, pause, budget, replay and fork; takes over run and trigger handling from ai_orchestration_service (phase: plan)
- **REQ-ORCH-004** [P3] Graph runtime gains node types plan, selector, handoff, judge and mission (phase: plan)
- **REQ-ORCH-005** [P3] Sequential, Consensus and Peer review keep working as modes
- **REQ-ORCH-006** [P3] Manager-led mode: plan as validated structured JSON, human plan approval by default, steps added as real graph nodes, accept/retry/re-plan capped at 2, final answer against the output contract
- **REQ-ORCH-007** [P3] Manager delegates only to seats in its team by ID; delegation depth 4 and fan-out cap 5 per step
- **REQ-ORCH-008** [P3] Progress card each round (goal met, looping, progress, next seat, instruction, each with reason); 3 no-progress rounds force re-plan; 2 looping answers pause for a human
- **REQ-ORCH-009** [P3] Round Table mode: selector picks next seat by rule, round-robin or manager; termination rule or round cap
- **REQ-ORCH-010** [P3] Handoff mode: handoff tool routes only along an admin-edited routes matrix; turn cap (default 30); optional user input between turns
- **REQ-ORCH-011** [P3] Graph mode: hand-drawn canvas gains Task contracts and is type-checked before Publish (phase: plan)
- **REQ-ORCH-012** [P3] Debate mode: two or more advocates, K rounds, a judge with a rubric
- **REQ-ORCH-013** [P3] Event Playbooks react to record events, schedules or webhooks on the visual canvas (phase: plan)
- **REQ-ORCH-014** [P4] Team Workspace worker pool runs parallel branches concurrently (default 4 per workspace, admin-set)
- **REQ-ORCH-015** [P3] Desktop keeps ordered branch walking and marks branches "logically parallel" in the trace (phase: plan)
- **REQ-ORCH-016** [P3] Each run holds a budget lease; exhausting it pauses the run in `waiting_budget` instead of failing
- **REQ-ORCH-017** [P3] Superstep commits: parallel branch writes stage and commit together; same-field writes are flagged as conflicts and resolved by the Join rule (phase: plan)
- **REQ-ORCH-018** [P2] Replay from step re-runs a step and everything downstream, reusing upstream outputs
- **REQ-ORCH-019** [P2] Fork copies a run up to a step into a Playground session; run outputs are immutable and replays/forks link to their parent
- **REQ-ORCH-020** [P3] Every existing Pipeline converts to a Team automatically; triggers and run history relinked; Pipelines screen becomes read-only with a link (phase: plan)
- **REQ-ORCH-021** [P3] Pipeline REST endpoints, `run_ai_agent_pipeline` tool and Pipelines in Solution exports keep working as aliases (phase: plan)
- **REQ-ORCH-022** [P4] Pipeline aliases removed two releases after Teams ship, after a deprecation notice; pipeline tables stay read-only for audit (phase: plan)
- **REQ-ORCH-023** [P3] Teams as building blocks: a published Team can fill a seat or be a graph node; child approvals answered by the parent within policy or passed up; depth counts toward cap 4
- **REQ-ORCH-024** [P3] Plan and Execute modes per agent: Plan mode allows read-tier tools only; switching to Execute needs user or policy approval
- **REQ-ORCH-025** [P4] Background model calls: long provider calls run in the background; run waits in `waiting_model`, survives restart, resumes on result
- **REQ-ORCH-026** [P3] Safe expressions shared with Business Rules for conditions, routes and transforms, with hard limits on nesting, value count and text size
- **REQ-ORCH-027** [P3] Field merge rules on a UI-edited state schema: keep latest, append, sum, union, highest-confidence or ask a human
- **REQ-ORCH-028** [P3] For-each step with concurrency cap, per-item timeout, merge step, per-item failure list and retry-failed-only
- **REQ-ORCH-029** [P3] Agent-chosen next step in Graph mode, only among author-ticked targets, each choice logged with reason
- **REQ-ORCH-030** [P3] Run-at-end nodes run once all other branches finish, even if some failed
- **REQ-ORCH-031** [P4] Delta checkpoints store only changes, with a full snapshot every 20 steps
- **REQ-ORCH-032** [P3] Durability per Team: save every step (required when writing records), save in background (read-only teams), or save at end (Playground drafts)
- **REQ-ORCH-033** [P1] Retry with backoff and jitter per node for transient failures (default 3 attempts); policy and validation errors never retry
- **REQ-ORCH-034** [P3] Busy-run policy per Team: reject, queue, interrupt and merge, or cancel and restart; duplicate record events merged by default
- **REQ-ORCH-035** [P3] Team variants (by region or unit) change settings only, versioned and evaluated together
- **REQ-ORCH-036** [P3] Composable stop rules (turns, phrase, tokens, cost, time, seat speaks, handoff to person, tool called, Stop pressed) with AND/OR; every run records its stop reason
- **REQ-ORCH-037** [P3] Wait for all or any: incoming connections grouped; the canvas checks group consistency before Publish
- **REQ-ORCH-038** [P3] Round Table speaker settings: eligibility rule, same-seat-twice switch, fixed rule overriding the manager
- **REQ-ORCH-039** [P3] Nested team returns: final answer only, model-written summary, or full transcript; inner team starts fresh unless memory is on
- **REQ-ORCH-040** [P3] Per-seat visibility rules in conversational modes
- **REQ-ORCH-041** [P4] Team events: a Team publishes named business events others subscribe to, with the same permission checks as record events
- **REQ-ORCH-042** [P4] Remote workers: chosen seats run on registered worker machines over mutual TLS and appear in the trace like any step
- **REQ-ORCH-043** [P4] Missions: long-running runs with a visible todo list and background sub-runs, looping until done, out of budget or stopped
- **REQ-ORCH-044** [P2] Edit state and resume: an authorised user corrects a field at a checkpoint and resumes as a new branch; edit, author and reason audited
- **REQ-ORCH-045** [P2] Breakpoints on live published runs set by an AI Admin; expire after 24 hours by default

## LOOP — Agent runtime loop

- **REQ-LOOP-001** [P1] One agent loop (`run_agent_once_with_text`) is extended, not replaced, and runs every agent step
- **REQ-LOOP-002** [P1] Context assembly trims each binding to its token budget and records a context receipt of what was included and why
- **REQ-LOOP-003** [P1] Model calls go through the AI Gateway with per-step model routing, fallback chain and forced air-gap for sensitive bound data
- **REQ-LOOP-004** [P1] Native function calling where supported, else a strict JSON tool format (no free-text ReAct)
- **REQ-LOOP-005** [P1] A trace event per round (tokens, cost, latency, tool calls and verdicts) streams over the existing SSE channel
- **REQ-LOOP-006** [P1] Output is validated against the output schema and then guardrails; failures return reasons to the agent and retry within the retry budget
- **REQ-LOOP-007** [P1] Step output, memory updates and audit record are persisted at step end
- **REQ-LOOP-008** [P1] Tool-round limit (default 8, up to 25): ask once for a final answer with tools disabled, mark the step `completed_partial`
- **REQ-LOOP-009** [P1] Per-step token limit behaves like the round limit
- **REQ-LOOP-010** [P1] Wall-clock limit per step (default 5 minutes): cancel the provider call and retry once on the fallback model
- **REQ-LOOP-011** [P1] Guardrail retries (default 2) then fail the step with verdicts attached and follow its `on_fail` edge if any
- **REQ-LOOP-012** [P1] Delegation depth 4: the delegate tool returns a readable refusal and the run continues
- **REQ-LOOP-013** [P1] Context window overflow compresses oldest tool results, then summarises earlier rounds; never exits the process
- **REQ-LOOP-014** [P1] Per tool binding: calls per step, failures before disable, and largest result kept in context (larger results stored and referenced)
- **REQ-LOOP-015** [P1] Reasoning modes Direct (default), Plan-first (plan shown in trace) and Reflect (one self-critique) (phase: plan)
- **REQ-LOOP-016** [P1] Prompt catalog: fresh, calm, versioned prompt templates; changes are diffable and go through evaluation
- **REQ-LOOP-017** [P1] Policies and guardrails attach at three hook points: around the step, each model call and each tool call, configured as Policy objects (phase: plan)
- **REQ-LOOP-018** [P1] Per-agent compaction policy (drop oldest, recent window, prune tool calls, shorten tool results); default shorten then window (phase: plan)
- **REQ-LOOP-019** [P1] Head-and-tail trimming option keeps the first and most recent messages
- **REQ-LOOP-020** [P1] Budget awareness: each agent turn is told remaining steps, tokens and dollars
- **REQ-LOOP-021** [P1] Do-not-persist state fields are never written to checkpoints, traces or audit, and are re-fetched on resume

## TOOL — Tools, connectors and MCP

- **REQ-TOOL-001** [P1] One tool catalog from native, connector, MCP and team sources, every tool with a risk tier, every call through schema → permission → policy → approval → execute → audit (phase: plan)
- **REQ-TOOL-002** [P3] Team tools `handoff_to_seat`, `ask_seat`, `post_to_board`, `request_human_input`, each registered with a risk tier
- **REQ-TOOL-003** [P2] Tool card: purpose, input schema, risk tier, data classes, origin, average latency and cost, last 5 calls with verdicts (phase: plan)
- **REQ-TOOL-004** [P1] `final_output: true` on a tool binding ends the step only if the result passes the output schema (phase: plan)
- **REQ-TOOL-005** [P1] Write tools take an idempotency key per step attempt so retries never double-write
- **REQ-TOOL-006** [P2] Simulate mode runs write-tier-and-above tools in a rollback shadow transaction and shows the diff
- **REQ-TOOL-007** [P1] Tools never see credentials; the connector layer injects them (AES-256-GCM at rest)
- **REQ-TOOL-008** [P1] Rate limits per tool, per connector and per workspace, enforced in the firewall before the call (phase: plan)
- **REQ-TOOL-009** [P4] Sandboxed code tool runs Python or SQL in isolation with no network by default at the Privileged tier (phase: plan)
- **REQ-TOOL-010** [P3] Progressive skills: agents see skill names and purposes until they load one (phase: plan)
- **REQ-TOOL-011** [P1] Server-initiated MCP requests (sampling, user prompts) are denied by default; when allowed they go through the approvals inbox with per-server request and token caps
- **REQ-TOOL-012** [P4] Notebook sandbox: stateful code session per run; code shown to a reviewer before running when policy requires
- **REQ-TOOL-013** [P1] Test button on every model, tool, connector, agent, team and stop rule, reporting in plain words what worked
- **REQ-TOOL-014** [P4] Untrusted tool output is marked untrusted, cannot add tools or change instructions; a prompt-injection classifier flags it in the trace
- **REQ-TOOL-015** [P1] Minimal untrusted-content defence ahead of full content labels: results from web, email, connector and external MCP tools are fenced and marked untrusted in the prompt, cannot change instructions or the tool list, and any write-tier or external tool call in a step that has read untrusted content needs approval by default (owner decision, ADR-0004)

## MEM — Memory and knowledge

- **REQ-MEM-001** [P3] Composite recall ranking: semantic match, recency (half-life per scope) and importance, weights per team; FTS5 fallback without embeddings
- **REQ-MEM-002** [P3] New memory scopes: run board (one run, every agent) and team (every seated agent) (phase: plan)
- **REQ-MEM-003** [P3] The Playground shows why each memory was chosen (phase: plan)
- **REQ-MEM-004** [P3] Opt-in auto-extraction after a step through the memory policy; sensitive and restricted classes need approval or are dropped (phase: plan)
- **REQ-MEM-005** [P3] Every memory write records its source step (phase: plan)
- **REQ-MEM-006** [P4] Deleting a run cascades to the memories it created and records an erasure entry (phase: plan)
- **REQ-MEM-007** [P2] Tasks bind knowledge collections explicitly and the Playground shows each cited chunk inline (phase: plan)
- **REQ-MEM-008** [P4] Memory consolidation job extracts durable facts into topic notes, merges duplicates; owner approves a diff; restricted content never extracted
- **REQ-MEM-009** [P3] Team drive: list, read, search and edit team files stored as attachments with content labels, versioned edits and a search time limit
- **REQ-MEM-010** [P4] Lessons from corrections: Edit-then-approve proposes a lesson; owner approves; approved lessons recalled on similar tasks and linked to their source
- **REQ-MEM-011** [P4] Identity-aware step cache for read-only steps keyed by inputs, access scope and data version

## SETUP — Setup experience and Guided Setup

- **REQ-SETUP-001** [P1] Agent settings live in eight plain-language groups (Identity, Job, Knowledge and memory, Tools and actions, Data access, Safety, Model and cost, Test and publish) with essentials and folded More options
- **REQ-SETUP-002** [P1] A new agent runs with three fields: name, job description and one tool; everything else defaults (phase: plan)
- **REQ-SETUP-003** [P1] One home per setting; other screens show it read-only with a link (phase: plan)
- **REQ-SETUP-004** [P1] Each change shows its effect in one sentence and risk and cost badges update live (phase: plan)
- **REQ-SETUP-005** [P1] Untouched optional groups show "Using defaults"; only Publish has hard requirements (phase: plan)
- **REQ-SETUP-006** [P2] Guided Setup side panel for agents and today's pipelines: Guide me button, command palette, admin assistant entry
- **REQ-SETUP-007** [P2] Progress rail with six stages and statuses; What's left list linking to fields; any-order stages; Publish gate needs safeguards reviewed and one passing test
- **REQ-SETUP-008** [P2] Each guide step explains why, prefills a recommended value with Use recommended and Show me an example
- **REQ-SETUP-009** [P2] Three ways to start: Describe it (drafts the objects), From a template, From blank
- **REQ-SETUP-010** [P2] Continue setup banner with percentage; progress saved per object and shared (ai_setup_progress); per-user guide preference with admin default
- **REQ-SETUP-011** [P3] Mode chooser asks up to three plain questions and recommends a mode with a picture; switching later keeps tasks
- **REQ-SETUP-012** [P3] Team settings in eight groups (Purpose, Members, Way of working, Work steps, Human checks, Triggers, Budget and limits, Test and go live)
- **REQ-SETUP-013** [P3] Canvas step panel with five tabs (Basics, Inputs, Result, Rules, Reliability); Basics alone is enough to run
- **REQ-SETUP-014** [P3] Guided Setup extended to Teams, every mode and the canvas
- **REQ-SETUP-015** [P3] Same group names in the Playground Build pane, editors, Solutions export and API docs (phase: plan)

## PG — Agent Playground UX

- **REQ-PG-001** [P2] Three panes: Build, Conversation, Trace; any run started elsewhere opens read-only in the same view
- **REQ-PG-002** [P2] Build pane edits settings in the same groups; unsaved edits apply only to Playground runs
- **REQ-PG-003** [P2] Conversation pane: chat or a form generated from the input schema; bind a test record with a record picker
- **REQ-PG-004** [P2] Trace pane: live timeline of steps, rounds, tool calls, delegations, guardrail verdicts, approvals, memory reads and writes, each with tokens, cost and latency; click for payload and context receipt
- **REQ-PG-005** [P3] Plan preview with cost range before a Manager-led or Graph run, waiting for Run (phase: plan)
- **REQ-PG-006** [P2] Step-through: pause after each step, inspect, edit the output, continue
- **REQ-PG-007** [P2] Simulate writes toggle with before-and-after diff per record
- **REQ-PG-008** [P2] Compare two variants (prompt, model, seat assignment or mode) side by side with output, cost and latency diff
- **REQ-PG-009** [P2] Fork here and Replay from here from any step
- **REQ-PG-010** [P2] Save as test case: one click turns a run into a golden case marked expected or corrected
- **REQ-PG-011** [P2] Promote: Publish runs the linked suite and is blocked below the threshold (default 90%)
- **REQ-PG-012** [P2] Explain this: any step explained in plain language (why this tool, memory, policy)
- **REQ-PG-013** [P2] Templates gallery; first three templates (Overdue Invoice Recovery, Lead Qualification, Claims First-Notice Intake) each with tasks, guardrails and a 10-case suite
- **REQ-PG-014** [P3] Further starter templates on the 11 industry apps (Contract Clause Review, Field Service Dispatch Triage, ...) (phase: plan)
- **REQ-PG-015** [P2] Full keyboard navigation, screen-reader-readable trace, WCAG AA colour through Theme Studio (phase: plan)
- **REQ-PG-016** [P2] The Playground ships in the online demo on sample data with write tools always simulated
- **REQ-PG-017** [P2] Interactive output cards (record card, field-edit form, approval form, table, chart) validated against a schema; actions use the normal write path
- **REQ-PG-018** [P2] Stream levels (summary, steps, tokens, debug), tool progress events, and streams that resume after the tab reopens

## UI — Screens (mockups)

- **REQ-UI-001** [P2] Agent Studio home: Describe a job, Continue setup banner, filterable list with lifecycle pill, sparkline, success and cost per run (phase: plan)
- **REQ-UI-002** [P2] Agent editor with eight groups, status dots, effect notes and Publish disabled with a reason until readiness passes (phase: plan)
- **REQ-UI-003** [P3] Team setup: way of working questions, recommended mode, members as seats, stop-rule sentence of chips (phase: plan)
- **REQ-UI-004** [P3] Work steps canvas with step palette, validation tags on steps, branch labels on connectors, dashed run-at-end (phase: plan)
- **REQ-UI-005** [P2] Playground screen per mockup 5: variants, compare, simulate, step through, budget meters, plan card, result cards (phase: plan)
- **REQ-UI-006** [P2] Run detail: stop reason, timeline with hatched not-to-scale waits, records changed, approvals, replay/fork/save actions (phase: plan)
- **REQ-UI-007** [P2] Approvals inbox with the four responses, diff of what is approved, standing approval option, and a one-decision-per-screen phone view (phase: plan)
- **REQ-UI-008** [P2] Ready to publish page: suite result against gate marker, per-seat scores, readiness list, impact list, shadow run (phase: plan)
- **REQ-UI-009** [P2] Screens use only the demo palette tokens (light and dark), risk-tier badges on every tool, step and approval, past-tense toasts (phase: plan)

## GOV — Enterprise governance

- **REQ-GOV-001** [P1] Enterprise sign-in with OIDC and SAML, built first; users, approvers and identities map to the customer directory
- **REQ-GOV-028** [P1] Enterprise sign-in is available in the desktop edition as well as Team Workspace, with providers including Okta, Microsoft Entra ID and Google (owner decision, ADR-0004)
- **REQ-GOV-002** [P3] Interactive runs act as the starting user intersected with the agent's ontology scope; an agent never sees more than its user (phase: plan)
- **REQ-GOV-003** [P3] Agent Identities: service principals with own Access Role and Record Scope for unattended runs; every step records agent and effective identity
- **REQ-GOV-004** [P2] Roles Viewer, Builder, Approver and AI Admin with the permissions in the spec (phase: plan)
- **REQ-GOV-005** [P3] Roles Team Owner and Auditor; evidence export for Auditors (phase: plan)
- **REQ-GOV-006** [P3] Plan approval required by default for every Manager-led plan; Team Owner can relax it to a cost or risk threshold
- **REQ-GOV-007** [P1] Output guardrails as versioned policy objects: PII leak, tone, required disclaimer, schema conformance, external domain allow-list
- **REQ-GOV-008** [P1] Data boundaries: which data classes may reach which model provider; sensitive data forced to local models (phase: plan)
- **REQ-GOV-009** [P4] Four-eyes rule: a run's starter cannot approve its own privileged steps
- **REQ-GOV-010** [P4] Kill switch: an AI Admin pauses all runs of a team or the workspace instantly; runs resume cleanly
- **REQ-GOV-011** [P1] All pause-for-human points move onto the durable ai_approvals inbox, retiring the three older mechanisms
- **REQ-GOV-012** [P3] Approvals carry plan, diff and evidence, SLA timers and escalation to a backup approver (phase: plan)
- **REQ-GOV-013** [P3] Cost forecast before each run from step count and model token history
- **REQ-GOV-014** [P3] Budgets at workspace, team, run and step level in USD and tokens; default monthly workspace budget only an AI Admin can raise
- **REQ-GOV-015** [P4] Monthly cost reports by team, agent, model and business unit (phase: plan)
- **REQ-GOV-016** [P4] Append-only, hash-chained audit log per run: prompts, outputs, tool inputs and outputs, verdicts, approvals
- **REQ-GOV-017** [P4] Retention and redaction: tool audit kept 7 years; step outputs 90 days by default
- **REQ-GOV-018** [P4] Evidence export: a signed bundle (JSON plus PDF summary) of one run
- **REQ-GOV-019** [P4] Desktop fully offline with Ollama; Team Workspace per-workspace provider allow-lists and region pinning; no telemetry leaves by default
- **REQ-GOV-020** [P1] Standing approvals scoped to run, team or workspace with expiry (default 30 days), listed, auditable and revocable in one place
- **REQ-GOV-021** [P1] Approval bound to exact arguments; any argument change requires approval again
- **REQ-GOV-022** [P1] Four review responses (Approve, Edit then approve, Reply with feedback, Skip this step), allowed set chosen per gate
- **REQ-GOV-023** [P1] Side-effect-safe resume: writes already made are recognised by idempotency key and not repeated
- **REQ-GOV-024** [P3] Organisation-aware approval routing by record owner, Org Unit and Work Team with out-of-office fallback
- **REQ-GOV-025** [P3] Policy hooks inspect agent-to-agent messages (delegations, handoffs, turns): redact, block or log
- **REQ-GOV-026** [P4] Content labels on by default: trust and confidentiality labels on every value, strictest wins, write/external tools refuse untrusted input unless policy allows
- **REQ-GOV-027** [P2] Record lineage: every agent change appears on the record's activity timeline linked to run, step, agent version and approver

## OBS — Observability

- **REQ-OBS-001** [P1] Span tree: run → step → round → model and tool calls; delegations nest as child steps
- **REQ-OBS-002** [P1] Span attributes: agent version, model, provider, tokens, cost, latency, policy and guardrail verdicts, context receipt hash
- **REQ-OBS-003** [P1] Span events stream over `/api/v1/events/stream`; every run produces a full trace (Phase 1 gate)
- **REQ-OBS-004** [P4] OTLP export to an admin-configured collector; prompt and output bodies excluded unless enabled per team
- **REQ-OBS-005** [P4] Orchestration dashboard: volume, success, completed_partial rate, p50/p95 latency, cost by team and model, approvals waiting, top failing guardrails, most-denied tools (phase: plan)
- **REQ-OBS-006** [P4] Outcome metrics: a Team links to Glossary metrics; dashboards show business results per run and per dollar

## EVAL — Evaluation

- **REQ-EVAL-001** [P2] Golden cases: input, bound test record, expected output or rubric; by hand or from a Playground run
- **REQ-EVAL-002** [P2] Graders: exact/schema match, LLM judge with rubric, tool-path check, cost and latency ceilings
- **REQ-EVAL-003** [P3] Team-level metrics: task success, steps used, re-plans, delegation count, cost per success (phase: plan)
- **REQ-EVAL-004** [P2] Regression gate: publishing runs the suite, blocked below threshold, results kept with the version
- **REQ-EVAL-005** [P4] Online sampling: a share of production runs (default 2%) graded for drift alerts
- **REQ-EVAL-006** [P4] A/B: two published versions split traffic and are compared on the same metrics
- **REQ-EVAL-007** [P2] Evaluation results feed the Unified Test Run so Solution readiness includes agent teams (phase: plan)
- **REQ-EVAL-008** [P2] Deeper evaluation: N repeats for variance, per-seat and whole-team scores, tool-argument checks
- **REQ-EVAL-009** [P2] Recorded runs: save a run with its model responses; suites replay them at no model cost with identical results
- **REQ-EVAL-010** [P2] Impact analysis before Publish lists dependent Teams, Workflows, Solutions, Action Types and suites that will rerun
- **REQ-EVAL-012** [P2] Real-model evaluation evidence without paid API keys: suites can run against a local open-weight model through Ollama, and recorded runs make results replayable in CI (owner decision, ADR-0004)
- **REQ-EVAL-011** [P4] Shadow mode: a new version runs alongside the published one on real triggers with writes simulated, compared before promotion

## API — Data model, APIs and interop

- **REQ-API-001** [P1] New and extended tables follow conventions: UUID ids, workspace_id on every row, audit columns, JSON config columns (phase: plan)
- **REQ-API-002** [P3] Team endpoints: create/edit draft, publish (runs suite), start run, forecast
- **REQ-API-003** [P2] Run endpoints: get, trace, replay, fork, cancel, pause, resume, to-test-case (phase: plan)
- **REQ-API-004** [P3] Plan approve endpoint (phase: plan)
- **REQ-API-005** [P1] Run events `run.started`, `step.started`, `round.completed`, `tool.called`, `tool.denied`, `approval.requested`, `plan.proposed`, `step.completed`, `run.completed`, `run.failed` with stable, documented schemas
- **REQ-API-006** [P3] Webhooks may subscribe to run event names (phase: plan)
- **REQ-API-007** [P3] Every endpoint is also exposed as MCP tools and `lanesra` CLI commands, with scoped API keys (phase: plan)
- **REQ-API-008** [P4] Team as an endpoint: OpenAI-compatible chat endpoint and MCP tool per published Team, scoped keys
- **REQ-API-009** [P3] Desktop bridge `team_commands.rs` replaces `ai_agent_pipeline_commands.rs` (phase: plan)
- **REQ-API-010** [Pv2] Teams callable over A2A
- **REQ-API-011** [Pv2] Playground events available over AG-UI
- **REQ-API-012** [Pv2] Teams exportable as declarative YAML inside a Solution
- **REQ-API-013** [P3] Workspace gallery over Solutions: reuse approved agents, tools, stop rules and Teams with owner, version and score

## NFR — Success metrics and quality bars

- **REQ-NFR-001** [P4] New admin builds a first working multi-agent team in under 15 minutes (measured in usability tests before GA) (phase: plan)
- **REQ-NFR-002** [P4] 95% or more of runs reach a final answer without hitting a limit (measured on template suites and shadow traffic) (phase: plan)
- **REQ-NFR-003** [P1] 100% of tool calls have a complete audit trail (enforced by test from Phase 1) (phase: plan)
- **REQ-NFR-004** [P3] 100% of runs show a cost estimate before start (phase: plan)
- **REQ-NFR-005** [P4] Telemetry to measure the share of Playground runs promoted to a published team (target 30%) (phase: plan)
