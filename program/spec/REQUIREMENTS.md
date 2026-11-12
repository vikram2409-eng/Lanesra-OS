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
- **REQ-PRE-015** [P0] Every capability ships in each edition it applies to - desktop, Team Workspace and the online demo (simulated on sample data, write tools always simulated) - and the tracker records, per work package, the editions it targets and the editions it has been delivered in; a work package is not done until every targeted edition is delivered (owner decision, 2026-10-16)
- **REQ-PRE-016** [P0] Every work package that ships to users updates the website help: its `/help/<slug>` articles in `app.js`, `sitemap.xml`, `llms.txt`/`llms-full.txt`, the Releases and Roadmap pages and in-product help links; the tracker names the articles per work package and refuses `done` until each article exists, names the work package and is in the sitemap, and CI fails if any help article is missing from the sitemap (owner decision, 2026-10-16)
- **REQ-PRE-017** [P0] The program dashboard is the hand-over point between AI agents working in sequence and is always current: it is regenerated and published automatically on every merge to `main`, refreshed by Claude Code at the start and end of each session, and shows how fresh its data is (owner decision, 2026-10-16)
- **REQ-PRE-018** [P0] The Team Workspace server image builds reproducibly (from `Cargo.lock`, with the pinned Rust and Node versions), is built and smoke-tested in CI on every pull request, and is published for every release (added 2026-10-16 after the Team Workspace review)
- **REQ-PRE-019** [P1] Team Workspace is safe to expose beyond one trusted person: first-run setup needs a one-time setup token, sign-in is throttled per account and per address, the public base URL is configured (needed for SSO redirects), and session lifetime and idle timeout are configurable
- **REQ-PRE-020** [P1] Team Workspace and desktop can be operated and recovered: scheduled backups, an automatic backup before every upgrade, a tested way to carry the encryption key with a backup so stored secrets survive a restore on new hardware, health and readiness endpoints, and an upgrade guide
- **REQ-PRE-021** [P0] A feature is built for desktop and Team Workspace from the same code, and appears in the online demo only once both editions have released it, never earlier (owner decision, ADR-0013; the "same release" part was replaced by REQ-PRE-024 under ADR-0019)
- **REQ-PRE-022** [P1] One Team Workspace server hosts several workspaces, each isolated (its own database, data key, backups and members); one account can belong to several workspaces; design choices keep a future SaaS edition possible (owner decision, ADR-0013, ADR-0014)
- **REQ-PRE-023** [P4] Team Workspace can run on PostgreSQL as well as SQLite, chosen by the organisation, with the full regression suite passing on both and a tool to move a workspace from SQLite to PostgreSQL (owner decision, ADR-0013, ADR-0014)
- **REQ-PRE-024** [P0] Desktop and Team Workspace are released separately, each with its own tag (`desktop-vX.Y.Z`, `team-workspace-server-vX.Y.Z`), GitHub Release page, version number and link; a desktop tag never builds or publishes the server image, and a server release never builds the installer; the image's `:latest` moves only when a server release is published (owner decision, ADR-0019)
- **REQ-PRE-014** [P0] An AI that works through GitHub without a shell (Muse AI: reads the repo, commits, opens issues and PRs) gets the same checks and tracker upkeep as one with a shell: CI regenerates the generated tracker files on its PR branch and posts validation, trace and test results on the PR where it can read them (owner decision, ADR-0004; updated 2026-10-16)

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
- **REQ-LOOP-022** [P1] Bring your own model: every agent, team and Playground feature works with any provider an organisation configures - OpenAI, Anthropic Claude, Google Gemini, Azure OpenAI, Google Vertex AI, AWS Bedrock, Ollama, and on-premises model servers (vLLM first-class; LM Studio, TGI and other OpenAI-compatible endpoints) - including fully offline, with custom base URLs, private CA certificates, proxies and auth headers (owner decision, ADR-0004)
- **REQ-LOOP-023** [P1] A model catalog records each configured model's capabilities (native tool calling, structured output, context window, streaming) and price per token (zero or an internal rate for on-premises models), and the loop, cost forecast and model routing use it rather than assuming one provider

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
- **REQ-GOV-002** [P1] Interactive runs act as the starting user intersected with the agent's ontology scope; an agent never sees more than its user (phase: plan; moved from P3 by ADR-0006)
- **REQ-GOV-003** [P1] Agent Identities: service principals with own Access Role and Record Scope for unattended runs; every step records agent and effective identity (moved from P3 by ADR-0006)
- **REQ-GOV-029** [P1] Agent credentials: an Agent Identity authenticates with Lanesra-issued, short-lived, scoped credentials (no password, no SSO login), rotatable and revocable by an admin, held by the platform and never placed in a model's context; humans, Agent Identities and API clients share one principal model (owner decision, ADR-0006)
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
- **REQ-GOV-014** [P3] Budgets at workspace, team, run and step level in USD and tokens; default monthly workspace budget of USD 100 that only an AI Admin can raise (owner decision, 2026-10-16)
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
- **REQ-EVAL-012** [P2] Provider verification without program-owned API keys: every provider adapter has contract tests from its published API format, suites can run against a local model (Ollama or vLLM) for real-model evidence, recorded runs replay in CI, and a provider conformance suite lets an organisation verify its own model and key from the product (owner decision, ADR-0004)
- **REQ-EVAL-011** [P4] Shadow mode: a new version runs alongside the published one on real triggers with writes simulated, compared before promotion
- **REQ-EVAL-013** [P5] Simulated users: a scripted or model-played persona with a goal holds a multi-turn conversation with an agent or Team; graders score the whole conversation (goal reached, turns used, policy breaches) (added by ADR-0020)
- **REQ-EVAL-014** [P5] Synthetic test cases: draft golden cases are generated from a knowledge collection, sample records or production traces, and a person reviews and accepts each one before it joins a suite (added by ADR-0020)
- **REQ-EVAL-015** [P5] Annotation queues: sampled or flagged runs enter a queue where reviewers score them against a rubric; the scores calibrate LLM judges (agreement rate reported) and corrected runs become golden cases (added by ADR-0020)
- **REQ-EVAL-016** [P5] End-user feedback: thumbs and comments on any answer, in the app and in every channel, linked to the run and step and usable as a sampling filter (added by ADR-0020)
- **REQ-EVAL-017** [P5] Red-team suite: a versioned library of adversarial probes (prompt injection, data exfiltration, privilege escalation, unsafe tool use, jailbreak) runs against an agent or Team with writes simulated; attack success rate is a publish-gate metric (added by ADR-0020)

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
- **REQ-API-010** [P5] Teams callable over A2A (moved from v2 by ADR-0020)
- **REQ-API-011** [Pv2] Playground events available over AG-UI
- **REQ-API-012** [P5] Teams exportable as declarative YAML inside a Solution (moved from v2 by ADR-0020)
- **REQ-API-013** [P3] Workspace gallery over Solutions: reuse approved agents, tools, stop rules and Teams with owner, version and score

## INT — Integration Hub and API surface (added 2026-10-21, owner decision on RAID Q-18 to Q-20; design in ADR-0018)

- **REQ-INT-001** [P1] Record events (`record.created`, `record.updated`, `record.archived`) that an active webhook subscribes to are delivered signed, with bounded retries and a delivery history, by Team Workspace and by the desktop app, with nobody pressing Test (RAID I-29)
- **REQ-INT-002** [P1] A webhook receiver can verify signatures: the signing secret is shown once when the webhook is created and again only when an admin rotates it, is never readable afterwards, and every rotation is audited without the value (RAID I-30)
- **REQ-INT-003** [P1] The webhook event types `field.changed`, `workflow.completed` and `workflow.failed` are emitted to subscribed webhooks; no event type is offered that nothing emits (RAID I-29, Q-20)
- **REQ-INT-004** [P1] An API client can be restricted to one or more networks (CIDR); the caller's address is the connection address unless the request arrives through an operator-configured trusted proxy (ADR-0018), and a restricted client is refused outside its range on REST, MCP and the event stream (RAID I-31)
- **REQ-INT-005** [P1] Every scope that can be granted to an API client is enforced by at least one endpoint, with an allow test and a deny test per scope, and the scope vocabulary and its endpoints are documented (RAID I-31)
- **REQ-INT-006** [P1] The REST API offers record search, bulk export and bulk write, webhook management and integration administration (read and operate, never secrets), each behind its own scope (ADR-0018, Q-20)

## Platform extension (added 2026-11-11, owner decision, ADR-0020; design in `program/design/SOLUTION-DESIGN.md`)

These requirements are not in the October spec. They close the gap to agent platforms (not only agent
frameworks) and are built in Phase 5, except two design requirements placed in Phase 1 so that work
in flight accounts for them.

## FLEET — Agent control plane for external agents

- **REQ-FLEET-001** [P1] The identity, trust and trace model for agents built outside Lanesra (A2A peers, MCP-hosted agents and ingested traces) is fixed in an ADR before Phase 3 team tools are built, so external agents reuse Agent Identities and the tool-call firewall instead of a parallel path
- **REQ-FLEET-002** [P5] An admin registers an external agent (name, owner, framework, endpoint, protocol, data classes, risk tier) in one registry beside Lanesra agents; a Team cannot call an endpoint that is not registered and approved
- **REQ-FLEET-003** [P5] A registered external agent can fill a seat or be called as a tool over A2A; every call passes schema → permission → policy → approval → execute → audit, counts against budgets, and its reply is fenced as untrusted content
- **REQ-FLEET-004** [P5] External agent runs received as OpenTelemetry traces are mapped onto the run → step → round span model and shown in the same run history with tokens and cost, marked as externally reported
- **REQ-FLEET-005** [P5] Policies, budgets, data boundaries and the kill switch apply to external agents: a paused, unapproved or over-budget external agent is refused at the firewall and at the A2A endpoint with a readable reason
- **REQ-FLEET-006** [P5] A fleet view lists every agent, Lanesra-built and external, with owner, version, lifecycle state, risk tier, last run, success rate, cost and open findings

## CRED — Delegated credentials and MCP gateway

- **REQ-CRED-001** [P1] The design for per-user delegated credentials, the token vault and the MCP gateway is fixed in an ADR before Phase 3 connector and MCP work, including how a grant joins the effective identity of a run
- **REQ-CRED-002** [P5] A user connects their own account to a connector or MCP server with OAuth 2.1 (authorisation code with PKCE); tokens are stored in a token vault encrypted under the workspace data key, are never shown again and are never placed in a model's context
- **REQ-CRED-003** [P5] The token vault refreshes, rotates and revokes grants; a user lists and revokes their own grants and an admin any grant; every use is audited with user, agent, tool and scope but never the token
- **REQ-CRED-004** [P5] In an interactive run a call to a delegated connector uses the starting user's grant; an unattended run uses the Agent Identity's own grant or is refused with a readable reason; a call never uses another user's grant
- **REQ-CRED-005** [P5] MCP servers are admitted through a registry: only approved servers are callable, each tool definition is pinned by hash, and a changed definition is quarantined until an admin approves it again
- **REQ-CRED-006** [P5] Tool descriptions from MCP servers are scanned for instruction-like content when registered and whenever they change; a finding blocks approval by default and is shown to the admin with the offending text
- **REQ-CRED-007** [P5] MCP gateway: one Lanesra endpoint offers the approved tools of several MCP servers to external MCP clients under scoped keys, with the same firewall, rate limits and audit as agent tool calls

## CHAN — Channels

- **REQ-CHAN-001** [P5] A channel binds a published Team or agent to an outside conversation surface; each conversation is a session with its own history, linked runs and stop rules
- **REQ-CHAN-002** [P5] Channel users map to Lanesra principals through the directory where one exists; an unmapped sender gets only what the channel's guest policy allows (nothing by default); inbound requests are signature-verified and replay-protected
- **REQ-CHAN-003** [P5] Slack channel: mentions, direct messages and threads start or continue a conversation; answers stream and interactive output cards render as Slack blocks
- **REQ-CHAN-004** [P5] Microsoft Teams channel: personal, group and channel conversations start or continue a conversation; answers stream and interactive output cards render as adaptive cards
- **REQ-CHAN-005** [P5] Email channel: mail to a channel address starts a run per thread and replies continue it; attachments are bound as untrusted content; outbound mail follows the approval policy
- **REQ-CHAN-006** [P5] Embeddable web chat widget: a script snippet with an allowed-origins list, signed visitor tokens issued by the host site, per-visitor rate limits and Theme Studio styling
- **REQ-CHAN-007** [P5] An approver can give any of the four review responses from Slack or Microsoft Teams; the decision is bound to the exact arguments and to the approver's mapped identity, and recorded exactly as one made in the approvals inbox

## OPT — Agent optimizer

- **REQ-OPT-001** [P5] The optimizer turns failing or low-scoring evaluation cases into proposed changes to instructions, examples, tool descriptions or reasoning mode, each shown as a diff with the cases that motivated it
- **REQ-OPT-002** [P5] A proposal is re-scored on the full suite and on a held-out set it never saw; it is applied only as a new draft version on the owner's approval and is never published automatically
- **REQ-OPT-003** [P5] Model right-sizing: for each step the optimizer reports the cheapest configured model that still passes the suite, with the cost and quality difference

## KNOW — Ontology-grounded knowledge

- **REQ-KNOW-001** [P5] Knowledge search combines full-text and vector results and reranks them with a configured model (a local model is allowed), reports the score parts for each chunk, and falls back to full-text search alone when no embedding or reranking model is configured
- **REQ-KNOW-002** [P5] Graph retrieval: a query can expand along Link Types from a bound record (its Object 360 neighbourhood) within a hop and row limit, returning records as cited context under the caller's record-level access
- **REQ-KNOW-003** [P5] Source sync: a Knowledge Source syncs from a connector on a schedule and keeps each document's source permissions; a chunk is returned only to an effective identity allowed to read its source document
- **REQ-KNOW-004** [P5] Ask the data: a tool answers numeric questions only through Glossary Metric definitions (metric, grain, filters, time range), never free-form SQL; the answer cites the metric version and is limited by the caller's Record Scope

## HUM — Humans as seats

- **REQ-HUM-001** [P5] A seat can be filled by a person or a Work Team; a step assigned to a human seat creates a work item with the task contract, its inputs and a form generated from the output schema, and the run waits durably
- **REQ-HUM-002** [P5] Human work items appear in one queue with claim, reassign and return; a submitted result is validated against the output schema and guardrails exactly like an agent's output
- **REQ-HUM-003** [P5] Human seats carry SLA timers, escalation and out-of-office routing; team metrics report human and agent time, cost and rework separately

## DEV — Developer surface

- **REQ-DEV-001** [P5] Teams as code: a Team with its agents, tasks, playbook, policies and suites exports to and imports from a declarative, versioned document with a published schema; import validates the document and shows a diff before anything is applied
- **REQ-DEV-002** [P5] Promotion between environments (for example development, test and production workspaces) from a Git repository or the CLI; the target runs the suite before accepting, and records who promoted which version
- **REQ-DEV-003** [P5] Python SDK over the public API: typed clients for agents, Teams, runs, streamed run events, approvals and evaluations, generated from the API description and tested against the scripted model provider
- **REQ-DEV-004** [P5] TypeScript SDK with the same coverage as the Python SDK
- **REQ-DEV-005** [P5] Custom tools: a builder packages a Python tool with a manifest (schema, risk tier, data classes, network needs); it runs only in the sandbox, only when signed by a publisher the workspace trusts, and every call goes through the firewall

## OPS — Enterprise operations

- **REQ-OPS-001** [P5] SCIM 2.0 provisioning: users and groups are created, updated and deactivated from the directory; deactivation ends sessions, revokes delegated grants and reassigns open approvals and work items
- **REQ-OPS-002** [P5] The server master key can be held in an external key management service (AWS KMS, Azure Key Vault, Google Cloud KMS or HashiCorp Vault) and connector secrets can be read from an external secrets manager; keys rotate without downtime
- **REQ-OPS-003** [P5] Audit and security events stream to a SIEM over syslog and HTTPS with a documented schema, delivered at least once, with a gap detector based on the audit hash chain
- **REQ-OPS-004** [P5] Alerts and service-level objectives: rules on run success, latency, cost, approval wait, guardrail failures and tool denials per Team; notifications go to email, webhooks and channels with de-duplication and quiet hours
- **REQ-OPS-005** [P5] A Helm chart and reference deployment for Kubernetes: server, workers, PostgreSQL, ingress with TLS, health probes and resource limits, linted and smoke-tested in CI
- **REQ-OPS-006** [P5] High availability and disaster recovery: several server replicas behind a load balancer, no run lost when a replica fails, and recovery point and recovery time targets that are documented and tested

## COMP — AI compliance pack

- **REQ-COMP-001** [P5] AI inventory: every agent, Team, model, tool and external agent is listed with owner, purpose, data classes, risk tier and versions; each published agent and Team has a generated agent card
- **REQ-COMP-002** [P5] Risk classification: a questionnaire assigns each agent or Team a risk class; high-risk classes require named controls (human oversight, logging, evaluation thresholds) before Publish
- **REQ-COMP-003** [P5] Control mappings: platform controls map to EU AI Act obligations, ISO/IEC 42001 and the NIST AI Risk Management Framework, kept as versioned data, each mapping linked to live evidence
- **REQ-COMP-004** [P5] Compliance report: a signed, dated bundle for an agent, a Team or the workspace showing controls, evidence and gaps, built on evidence export

## Admin console and Experience Studio (added 2026-11-11, owner decision, ADR-0022; design in `program/design/SOLUTION-DESIGN.md` sections 15 and 16)

A parallel track (Phase 6): it does not wait for the Phase 1 to 5 gates.

## ADM — Admin console

- **REQ-ADM-001** [P6] The Admin console has one menu of ten areas in three bands - Build (Data and ontology, Experience Studio, Automation, AI Studio), Run (AI operations, Integrations and channels, Apps and deployment), Govern (People and access, Governance and compliance, System) - defined once as data that desktop, Team Workspace and the online demo all read; every admin tool has exactly one home in it
- **REQ-ADM-002** [P6] The menu stays on screen while a tool is open, each area has its own sub-menu, and every admin screen has an address so links, bookmarks, the command palette and help articles can open it directly
- **REQ-ADM-003** [P6] The menu lists only the areas and tools the signed-in role may use, enforced by the server as well as hidden in the menu; feature-flagged tools appear only while their flag is on
- **REQ-ADM-004** [P6] A top bar on every admin screen carries search (which also finds tools by their previous names), an environment label, and a tray listing every draft with unpublished changes
- **REQ-ADM-005** [P6] Admin Home shows what needs attention with a count on each menu area, setup progress, pinned and recent tools, and recent changes, all read from real aggregates
- **REQ-ADM-006** [P6] The Admin Assistant and Help open as side panels beside any admin screen, the Help panel on the article for that screen
- **REQ-ADM-007** [P6] Every admin tool follows one pattern: a list with search, filters and a lifecycle pill, and a detail screen with the tabs Overview, Configure, Test and History where they apply
- **REQ-ADM-008** [P6] The grouping is understandable without training: every area and tool shows a one-line plain-language description; the three AI areas are labelled by purpose (AI Studio: design and test; AI operations: watch and respond; Governance and compliance: rules and proof) and an agent or Team links to its own runs and policies across them; a first-visit tour and a "where did it move" list map every previous location to the new one
- **REQ-ADM-009** [P6] Every work package of the Admin console and Experience Studio track ships in the desktop edition, Team Workspace and the online demo, and updates the website help with what the feature does, how to set it up step by step, and where to find it in the menu; when a menu path changes, every help article, llms file and website page that names the old path is updated in the same release

## EXP — Experience Studio: pages, components, blocks, templates, navigation and theme

- **REQ-EXP-001** [P6] One page model serves every kind of screen: record page, create and edit form, list page, home page, dashboard and console, with one definition format, one builder and one Draft → Published lifecycle with role assignment
- **REQ-EXP-002** [P6] A published page takes over its screen for every built-in and custom object, in desktop and Team Workspace alike; an object with no published page keeps rendering exactly as before
- **REQ-EXP-003** [P6] Existing Screen layouts (forms) are carried into the page model automatically with no change in what users see; the Screen layouts tool, its API and its Solution export keep working as aliases until removed after a deprecation notice
- **REQ-EXP-004** [P6] Existing Dashboards and Dashboard KPI choices are carried into the page model automatically with no change in what users see; role assignment and the Default fallback are kept
- **REQ-EXP-005** [P6] Custom screens: an admin creates a page that is not tied to one record, gives it data sources (an object, a saved view, a report, a Glossary metric), page filters and parameters, and assigns it by role
- **REQ-EXP-006** [P6] All 24 existing components render live data and working actions: KPI sums and counts, charts, buttons, command bar, quick actions and agent actions; no component shows a "not wired yet" placeholder
- **REQ-EXP-007** [P6] Data components: record list, board, timeline, metric tile and filter bar; a filter bar drives every data component on the page that shares its source
- **REQ-EXP-008** [P6] Form and content components: form (single or multi-step, using the object's field rules), image, link list, path (status steps) and embed (allowed domains only)
- **REQ-EXP-009** [P6] Work and AI components: agent chat, approvals, work queue and run activity; every agent call from a page goes through the tool firewall, approvals and budgets, and acts as the signed-in user
- **REQ-EXP-010** [P6] Blocks: any configured component or group can be saved as a named block with inputs, inserted linked (a change reaches every page that uses it) or as a copy, versioned with Draft → Published, shown with a where-used list before any change, and exported in a Solution
- **REQ-EXP-011** [P6] Six starter blocks ship ready to use: record summary, my open work, approval strip, KPI row, activity timeline and agent chat
- **REQ-EXP-012** [P6] Five page layouts are offered when starting blank: single column, main and side panel, master and detail, three columns, and header with tabs and footer bar
- **REQ-EXP-013** [P6] Six screen templates ship with components placed and bound to the chosen object or app: List and Detail, Analytics Board, Guided Intake, Workspace Home, Operations Console and Agent Workbench; the first three need no AI feature; the four existing record-page templates remain; a page made from a template is fully editable and never tied to it
- **REQ-EXP-014** [P6] The builder supports undo and redo, copy and paste (within and across pages) and full keyboard operation
- **REQ-EXP-015** [P6] Each component can carry visibility rules by role, by field condition (the same conditions Business Rules use) and by device size; a rule only hides presentation, and record and field access are always enforced by the server
- **REQ-EXP-016** [P6] Before publishing, an admin can preview a page as a chosen role with a real record, compare any two versions, and sees checks for broken bindings, contrast, empty states and components the role cannot use
- **REQ-EXP-017** [P6] Describe a page: the Admin Assistant drafts a page from a plain-language description as a draft for review, using only components and blocks that exist
- **REQ-EXP-018** [P6] Each app has a menu editor: pages, objects, dashboards and links in groups, visibility by role, and a home page per role
- **REQ-EXP-019** [P6] Themes can be set per app, come as light and dark pairs, and carry component styles (density, corner shape); the contrast check gates both modes
- **REQ-EXP-020** [P6] The published theme also styles printed and PDF documents and system email

## NFR — Success metrics and quality bars

- **REQ-NFR-001** [P4] New admin builds a first working multi-agent team in under 15 minutes (measured in usability tests before GA) (phase: plan)
- **REQ-NFR-002** [P4] 95% or more of runs reach a final answer without hitting a limit (measured on template suites and shadow traffic) (phase: plan)
- **REQ-NFR-003** [P1] 100% of tool calls have a complete audit trail (enforced by test from Phase 1) (phase: plan)
- **REQ-NFR-004** [P3] 100% of runs show a cost estimate before start (phase: plan)
- **REQ-NFR-005** [P4] Telemetry to measure the share of Playground runs promoted to a published team (target 30%) (phase: plan)
- **REQ-NFR-006** [P5] An admin registers an external agent, puts it under policy and sees its first governed run in the run history in under 30 minutes (measured before the Phase 5 gate) (added by ADR-0020)
- **REQ-NFR-007** [P6] In a findability test before the Phase 6 gate, at least 80% of first-time admins locate each of ten common tasks in the new menu at the first attempt, including one task in each of the three AI areas (added by ADR-0022)
