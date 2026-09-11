// Product Help: detailed, task-oriented admin user guides, one category at a
// time - starting with the Agentic AI Foundry (this panel's own "AI Agent
// Foundry" category). Static content shipped with the app - no new table,
// no admin-editable CMS, the same delivery model as everything else on this
// Settings screen that isn't workspace data.
//
// Deliberately duplicated from (not imported from) the website's own
// /help section in the repo root's app.js - this codebase's established
// convention everywhere the online demo and the desktop app meet: each
// side maintains its own copy rather than sharing a literal file across
// that boundary (desktop/tsconfig.json only type-checks `include: ["src"]`,
// so a cross-boundary import would need new build tooling for no real
// benefit here). The two copies cover the same facts, independently
// written - if they drift on a real behavior, that's a doc bug to fix in
// both places, the same way a code fact that changes needs updating
// wherever it's mentioned today.

export interface HelpSection {
  heading: string;
  bodyHtml: string;
}

export interface HelpTopic {
  slug: string;
  title: string;
  summary: string;
  sections: HelpSection[];
}

export interface HelpCategory {
  key: string;
  label: string;
  icon: string;
  blurb: string;
  topics: HelpTopic[];
}

export const HELP_CATEGORIES: HelpCategory[] = [
  {
    key: "ai-agent-foundry",
    label: "Agentic AI Foundry",
    icon: "🏭",
    blurb:
      "Connect an LLM, build named AI Agents with memory, skills, actions and guardrails, chain them into orchestrated Pipelines, route and budget them through the Unified AI Gateway, and extend the whole thing from outside the product.",
    topics: [
      {
        slug: "connect-llm-provider",
        title: "Getting started: connect an LLM provider",
        summary:
          "Add an Anthropic, OpenAI-compatible or Google Gemini key, understand the MCP server and CLI, and try the Assistant.",
        sections: [
          {
            heading: "Nothing works until this is done",
            bodyHtml:
              "<p>The Assistant, every AI Agent, Pipelines, Vector Search and the Evaluation Harness all call through one LLM provider connection - configure it first. The setup screen and its behavior are identical whether this is a Personal Workspace or a Team Workspace server; only who can reach <b>Admin</b> differs.</p>",
          },
          {
            heading: "LLM & MCP → LLM tab",
            bodyHtml:
              "<p>This tab (its own category above) opens on <b>LLM</b>. Choose a provider type, paste a key, and for a self-hosted/local model set <b>Base URL</b> instead. Always click <b>Test key</b> before relying on it - it makes one real minimal call and reports the exact result, including a bad key or unreachable host.</p><ul><li><b>Anthropic</b> - Claude models. No embeddings endpoint - see the Context Layer article for what that affects.</li><li><b>OpenAI-compatible</b> - OpenAI itself, or anything speaking the same shape: Ollama, LM Studio, vLLM, an internal proxy. Point Base URL at a local address for a fully offline setup.</li><li><b>Google Gemini</b> - Gemini models.</li></ul>",
          },
          {
            heading: "What \"bring your own key\" means here",
            bodyHtml:
              "<p>This key is used to call the provider directly - never resold or proxied. It's encrypted at rest (AES-256-GCM) the same as every other secret on this panel and is never returned by a read; the field always renders blank. Nothing calls an LLM anywhere in the product until this save passes Test key.</p>",
          },
          {
            heading: "MCP Server tab",
            bodyHtml:
              '<p>A stateless <code>POST /mcp</code> endpoint speaking the Model Context Protocol (JSON-RPC 2.0) - what Claude Desktop and most IDE agents use. It exposes 7 tools: <code>list_objects</code>, <code>get_object_metadata</code>, <code>list_records</code>, <code>get_record</code>, <code>create_record</code>, <code>update_record</code>, <code>archive_record</code> - the same permission-checked dispatcher the REST API already wraps.</p><p class="help-note">Only reachable where a Team Workspace server is running - a pure desktop install has no listening socket for it, the exact boundary Integration Hub\'s API Access already states.</p><p>Auth reuses that same mechanism: issue an API client from <b>Integration Hub → API Access</b> and send <code>Authorization: Bearer {client_id}.{secret}</code>, scoped to <code>metadata.read</code> (the two lookups), <code>objects.read</code> and/or <code>objects.write</code>. No separate MCP credential exists.</p>',
          },
          {
            heading: "The CLI is a different tool",
            bodyHtml:
              "<p>The <code>lanesra</code> CLI (package <code>lanesra-cli</code>, <code>cargo build --release -p lanesra-cli</code>) is not an MCP client - it doesn't speak JSON-RPC at all. It's a plain shell wrapper over the same REST endpoints an agent's own record tools and MCP ultimately call, pointed at the same base URL and an API client key. Use MCP when an external AI agent needs access; use the CLI when a human or a script does.</p>",
          },
          {
            heading: "Assistant: records mode vs. admin mode",
            bodyHtml:
              "<p>Every user's Assistant carries the same 7 record tools MCP exposes, scoped to what that user can already touch. An Administrator's Assistant additionally runs in <b>admin mode</b>, with tool pairs across Business Rules, Workflow Automation, the Data Model, Integration Hub, Dashboards, Apps, Reports, Saved Views, Users and more, so it can genuinely build things, not just describe them. A Connection's or API client's secret is never read by a tool call in either mode.</p>",
          },
          {
            heading: "Next",
            bodyHtml: '<p>Continue to <b>Build your first AI Agent</b> in this same list.</p>',
          },
        ],
      },
      {
        slug: "build-your-first-agent",
        title: "Build your first AI Agent",
        summary:
          "Persona, the Actions checklist, persistent Memory, Skills, Guardrails and delegation - from AI Agents → New Agent.",
        sections: [
          {
            heading: "Create the agent",
            bodyHtml:
              "<p>From <b>AI Agents → New Agent</b>, set a name, icon, short description, and the <b>system prompt</b> - the persona and scope statement everything else below layers on top of.</p>",
          },
          {
            heading: "The Actions checklist",
            bodyHtml:
              '<p>Two catalogs of tools an agent may be granted:</p><table><thead><tr><th>Catalog</th><th>Covers</th></tr></thead><tbody><tr><td>Record (8)</td><td><code>list_objects</code>, <code>get_object_metadata</code>, <code>list_records</code>, <code>get_record</code>, <code>create_record</code>, <code>update_record</code>, <code>archive_record</code>, <code>search_records</code></td></tr><tr><td>Admin (37)</td><td>Read/write pairs across Business Rules, Workflow Automation, the Data Model, Integration Hub, Dashboards, Apps, Reports, Saved Views, Users, Numbering, the Workspace Profile, Connectors, and AI Agents/Skills themselves</td></tr></tbody></table><p class="help-note">Checking a single admin action makes the whole agent Administrator-only to chat with - computed automatically, not a separate switch to remember.</p>',
          },
          {
            heading: "Persistent Memory",
            bodyHtml:
              "<p>A free-text document read every run, revised by the agent's own <code>update_memory</code> tool or an admin's direct edit. Every real overwrite snapshots the prior value first, browsable from <b>Show memory history</b>, most-recent-first; a no-op save (empty or unchanged) writes nothing new.</p>",
          },
          {
            heading: "Skills",
            bodyHtml:
              "<p>Build reusable Skills under <b>AI Skills</b>: a name and short description the agent always sees, plus full instructions loaded only when it decides a task calls for that skill. Attach the same skill to several agents instead of duplicating instructions into each one's prompt.</p>",
          },
          {
            heading: "Guardrails",
            bodyHtml:
              "<p><code>guardrails_md</code> is advisory text folded into the system prompt as an explicit boundary statement - treat it the same as any persona instruction, not a hard rule. One guard is code-enforced regardless: the tool loop aborts the moment the identical tool call with identical arguments repeats 3 times running.</p>",
          },
          {
            heading: "Delegation",
            bodyHtml:
              "<p>Name other agents as delegates and this agent can call them mid-run and use their real answer, capped at 4 levels deep so a delegation chain can't recurse forever. This is the agent's own runtime decision - different from a Pipeline's admin-authored fixed sequence, covered next.</p>",
          },
          {
            heading: "Try it",
            bodyHtml:
              "<p>Save, then open the <b>Assistant</b> and pick this agent from the selector to chat with it directly. From here: chain it into a Pipeline, or give it its own Gateway routing policy.</p>",
          },
        ],
      },
      {
        slug: "orchestration-pipelines",
        title: "Orchestration: Pipelines & topologies",
        summary:
          "Sequential, consensus and peer-review topologies, triggers, human-in-the-loop approval gates, and OTLP tracing.",
        sections: [
          {
            heading: "Pipeline vs. delegation",
            bodyHtml:
              "<p>Delegation (previous topic) is dynamic - the agent decides. A Pipeline, built under <b>Orchestration</b>, is a fixed, admin-authored, ordered sequence of steps, each calling a named agent, under a chosen topology. A Pipeline step's agent can still delegate mid-step independently of this.</p>",
          },
          {
            heading: "Sequential",
            bodyHtml:
              "<p>Each step runs after the one before it finishes. A step's input can reference the trigger input or <code>{{previous_output}}</code> - the immediately preceding step's real text. Example: a 3-step \"research → draft → polish\" chain, where step 2 embeds <code>{{previous_output}}</code> to receive step 1's research.</p>",
          },
          {
            heading: "Consensus",
            bodyHtml:
              "<p>Every step but the last runs independently against the original trigger input - not chained - each producing its own candidate answer. The final <b>synthesizer</b> step's prompt embeds <code>{{candidate_outputs}}</code>, every candidate joined together, and combines them into one answer.</p>",
          },
          {
            heading: "Peer review",
            bodyHtml:
              '<p>A drafter/reviewer loop: draft, critique, revise, repeat until the reviewer\'s reply starts with the literal marker <b>APPROVED</b> - capped at <b>3 rounds</b>, after which the run ends in a clear failure rather than looping indefinitely.</p>',
          },
          {
            heading: "Triggers & run history",
            bodyHtml:
              '<p>Fire manually, on a schedule, from an authenticated inbound webhook, or as a <b>"Run AI agent"</b> Workflow Automation action - one unified run history records every run with its per-step results, regardless of trigger.</p>',
          },
          {
            heading: "Approval gates",
            bodyHtml:
              "<p>Any topology can pause for an Administrator at its one well-defined resume point: any sequential step, a consensus synthesizer, or a peer-review reviewer (every round). A paused run sits <b>awaiting approval</b> until approved (optionally editing the paused output before it feeds forward) or rejected outright.</p>",
          },
          {
            heading: "OTLP tracing",
            bodyHtml:
              "<p>Real per-step timing renders as a standard OTLP <code>resourceSpans</code> trace from a run's detail view - viewable as JSON, or pushed to a collector endpoint configured on the Gateway tab.</p>",
          },
        ],
      },
      {
        slug: "unified-ai-gateway",
        title: "Unified AI Gateway: routing, failover, budgets & DLP",
        summary: "Named Providers, per-agent routing, System/Agent token budgets, and DLP-driven forced air-gapping.",
        sections: [
          {
            heading: "Providers beyond the default",
            bodyHtml:
              "<p>Register additional named Providers on the Gateway tab beyond the workspace default set up earlier - another key, a second endpoint, a local model - each independently testable, so one agent can run on a different provider than everything else.</p>",
          },
          {
            heading: "Per-agent routing",
            bodyHtml:
              "<p>Click <b>Routing</b> on any row in AI Agents to set:</p><ul><li><b>Primary provider</b> - tried first</li><li><b>Fallback provider</b> - tried on an error or rate limit</li><li><b>Local / air-gapped fallback</b> - the last resort</li></ul><p>plus a per-agent temperature/max-tokens override. No policy configured means unchanged pre-Gateway behavior.</p>",
          },
          {
            heading: "Token budgets",
            bodyHtml:
              '<p>Usage is tracked per (workspace, agent, user, day) against a daily budget at two tiers: <b>System</b> (Gateway tab → "System token budget", applies to every run) and, in the same Routing panel, an optional per-<b>Agent</b> budget. Today\'s usage and any failover events show on the Gateway health view.</p>',
          },
          {
            heading: "DLP-driven forced air-gapping",
            bodyHtml:
              "<p>In Routing, name which sensitive classes force this agent straight to its local/air-gapped fallback the moment a scan detects one - regardless of the normal order:</p><ul><li>Social Security Number</li><li>Credit card number</li><li>Bank account / routing number</li><li>Phone number</li><li>Email address</li></ul><p>Leave every class unchecked and nothing changes - opt-in per agent, not workspace-wide.</p>",
          },
        ],
      },
      {
        slug: "context-layer-vector-search",
        title: "Context Layer & Vector Search",
        summary: "Lexical vs. semantic record search, setting up Vector Search, and the agent context primer tools.",
        sections: [
          {
            heading: "Two ways to search records",
            bodyHtml:
              '<p><code>search_records</code> is lexical - a SQLite FTS5 index kept current by triggers, ranked with <code>bm25()</code>, always on. <code>semantic_search_records</code> is meaning-based - real embedding vectors compared by cosine similarity, so it can find a record sharing none of the query\'s exact words. It needs Vector Search set up first.</p>',
          },
          {
            heading: "Setting up Vector Search — LLM & MCP → Gateway",
            bodyHtml:
              "<p>Vector Search lives as a card inside the <b>Gateway</b> tab, not a standalone tab. It needs an embeddings-capable provider: <b>OpenAI-compatible</b> or <b>Google Gemini</b> work; <b>Anthropic has no embeddings API</b>, so an Anthropic-only workspace keeps lexical search but not semantic search until a second provider is added.</p><ol><li>Optionally override the embedding model.</li><li><b>Reindex now</b> embeds every existing Custom Object record.</li><li>The card shows a live embedded/pending count.</li></ol><p>After that, a record's own create/update/archive enqueues it for re-embedding without blocking the save. No vector database or native extension - vectors are BLOBs in SQLite, cosine similarity computed in Rust.</p>",
          },
          {
            heading: "Memory history",
            bodyHtml:
              '<p>Covered in <b>Build your first AI Agent</b> - each agent\'s "Show memory history" panel.</p>',
          },
          {
            heading: "Agent context primer",
            bodyHtml:
              "<p><code>get_object_metadata</code> includes every relationship an object participates in with the label and related object resolved; <code>get_related_records</code> follows one for a specific record; <code>get_platform_overview</code> explains how this platform's own primitives compose (Custom Object → Fields → Relationships → Business Rules → Workflow Automation → App → Dashboard) plus a live count of this workspace's own objects, relationships and apps.</p>",
          },
        ],
      },
      {
        slug: "evaluation-harness",
        title: "Evaluation Harness: testing your agents",
        summary: "Suites, Cases with plain-English success criteria, LLM-as-judge grading, and why it fails closed.",
        sections: [
          {
            heading: "Where this lives",
            bodyHtml: "<p>The <b>Evaluations</b> tab in this category - regression-testing an Agent or Pipeline's real behavior without a brittle exact-match assertion.</p>",
          },
          {
            heading: "Suites and Cases",
            bodyHtml:
              "<p>A <b>Suite</b> names its target Agent or Pipeline. Each <b>Case</b> is an input text plus a plain-English <code>success_criteria</code> (\"must cite a specific figure, not just describe the report\") rather than a fixed expected string. Write several per suite - the behavior you want, and the edge cases you're worried about.</p>",
          },
          {
            heading: "How grading works",
            bodyHtml:
              '<p>Running a suite executes each case\'s target for real, then makes one judge call with the original input, your criteria and the actual output. Only the judge reply\'s <b>first non-empty line</b> is read - it must be exactly <code>PASS</code> (case-insensitive) to count as a pass; everything else becomes the recorded reasoning.</p>',
          },
          {
            heading: "Fails closed",
            bodyHtml:
              "<p>A case whose own run errors (provider outage, deleted agent) is recorded failed without wasting a judge call. A judge reply that doesn't parse as a clean PASS/FAIL is also graded a <b>failure</b>, never a silent pass - a confused judge can only make a suite too strict, never falsely green.</p>",
          },
          {
            heading: "Reading results",
            bodyHtml: "<p>Each case shows pass/fail next to the judge's own reasoning text, so a failure is explainable on sight.</p>",
          },
        ],
      },
      {
        slug: "extending-the-foundry",
        title: "Extending the Foundry",
        summary: "External MCP clients, Solution export/import, firing Pipelines from automation, and the REST API underneath.",
        sections: [
          {
            heading: "Point an external client at this workspace",
            bodyHtml:
              "<p>Claude Desktop, an IDE agent, or a custom script can target this Team Workspace's <code>POST /mcp</code> endpoint with an API client key from Integration Hub → API Access (scopes covered in <b>Getting started</b>). It then reads/writes real records under the identical permission checks a person's own UI action already goes through.</p>",
          },
          {
            heading: "Package an Agent or Skill into a Solution",
            bodyHtml:
              "<p>From Deployment Management, add an Agent or Skill - persona, Memory seed, guardrails, Actions, delegates - to a named Solution and export/import it exactly like a Custom Object or Business Rule. Its Gateway routing policy and live token-usage history deliberately do not travel - workspace-specific runtime state, configured fresh after import.</p>",
          },
          {
            heading: "Fire a Pipeline from automation",
            bodyHtml:
              '<p>Beyond manual/scheduled triggers, launch a Pipeline (or a lone Agent) from an authenticated inbound webhook or a <b>"Run AI agent"</b> Workflow Automation action - see Orchestration for the full list.</p>',
          },
          {
            heading: "The REST API underneath",
            bodyHtml:
              "<p>An agent's own record tools, MCP, and the CLI all call through the same generic, permission-checked <code>/api/v1/objects/...</code> API (<code>api_object_service</code>) Integration Hub exposes elsewhere. A custom integration can target that surface directly with an API client key, no AI Agent required.</p>",
          },
          {
            heading: "What comes next",
            bodyHtml:
              "<p>This category covers the AI &amp; Agentic Layer end to end. Core CRM &amp; No-Code Platform, App Catalog, Deployment Management and Integration Hub are the named next Help categories - not built yet, planned.</p>",
          },
        ],
      },
    ],
  },
];
