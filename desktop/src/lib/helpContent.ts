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
              "<p>This category covers the AI &amp; Agentic Layer end to end. Core CRM &amp; No-Code Platform is alongside it in this same list; App Catalog, Deployment Management and Integration Hub are the named next Help categories - not built yet, planned.</p>",
          },
        ],
      },
    ],
  },
  {
    key: "core-crm-no-code",
    label: "Core CRM & No-Code Platform",
    icon: "🧩",
    blurb:
      "Run the built-in sales lifecycle, then reshape the workspace itself - your own record types and relationships, admin-defined fields, conditional business rules, workflow automation, drag-and-drop screen layouts, and your own published app with its own dashboard.",
    topics: [
      {
        slug: "core-crm-sales-lifecycle",
        title: "The core CRM: companies, contacts & the sales lifecycle",
        summary: "The connected record model, the flexible Company → Quote → Order → Invoice path, and where things live.",
        sections: [
          {
            heading: "The connected record model",
            bodyHtml:
              '<p>Everything hangs off a <b>Company</b>: its Contacts, its Opportunities, and every Quote, Order, Invoice, Contract and Task that references it. A Company\'s own "360" detail page surfaces every one of those linked records in one place.</p>',
          },
          {
            heading: "The flexible path",
            bodyHtml:
              "<p>The full lifecycle is Company → Contact → Opportunity → Quote → Order → Invoice, with Products &amp; Services and Contracts attaching along the way and Tasks trackable against any of them. A <b>direct-quote</b> or <b>direct-order</b> shortcut skips straight from a Company to a Quote or Order when there was no formal Opportunity - the full pipeline isn't a mandatory gate.</p>",
          },
          {
            heading: "Line items & money math",
            bodyHtml:
              "<p>Quotes, Orders, Invoices and Contracts carry real line items against Products &amp; Services with computed totals. Money is integer cents internally, not floating-point - the class of rounding drift a spreadsheet accumulates over enough rows can't happen here.</p>",
          },
          {
            heading: "Data integrity, automatic",
            bodyHtml:
              "<p>A duplicate-name/email warning fires before creating a Company or Contact that looks like one that already exists (a warning, not a hard block), and every object gets gap-free, sequential numbering - an admin sets the ID format per object, the same prefix/digit-width mechanism Custom Objects exposes for your own record types.</p>",
          },
          {
            heading: "Finding things",
            bodyHtml:
              '<p>Every ID anywhere is a hyperlink to its record. The Dashboard has clickable, filterable KPIs; global search resolves straight to a match. Company and Contact each get a "360" page - full field overview plus every linked record, one click away.</p>',
          },
          {
            heading: "Reports & data safety",
            bodyHtml:
              "<p>Beyond the Dashboard: a fixed Reports gallery (revenue by month, win rate, AR aging, sales by owner) plus a Custom Reports builder - any object, group by status/stage or a Reportable field, count or sum, CSV export. Whole-workspace backup/restore and CSV import/export round out data safety.</p>",
          },
          {
            heading: "Next: make it yours",
            bodyHtml:
              "<p>The rest of this category - Custom Objects, Custom Relationships, Custom Fields, Business Rules, Workflow Automation, Screen Builder, App Builder &amp; Dashboards - is how an admin reshapes the workspace without writing code.</p>",
          },
        ],
      },
      {
        slug: "custom-objects",
        title: "Custom Objects: your own record types",
        summary: "Define a whole new business object at runtime - its own icon, ID format and sidebar entry - no code change.",
        sections: [
          {
            heading: "Why Custom Objects",
            bodyHtml:
              "<p>The 9 built-in entities cover the common sales lifecycle, but real businesses usually need something outside it - Vendors, Assets, Projects. A Custom Object is a genuinely new record type that plugs into every other admin tool exactly like a built-in one - not a smaller, separate feature set.</p>",
          },
          {
            heading: "Create one — Data Model → Custom Objects",
            bodyHtml:
              '<table><thead><tr><th>Field</th><th>Example</th></tr></thead><tbody><tr><td>Singular name</td><td>Vendor</td></tr><tr><td>Plural name</td><td>Vendors</td></tr><tr><td>Icon</td><td>from a provided set</td></tr><tr><td>Record-number prefix &amp; digit width</td><td><code>VEN</code> + 6 → <code>VEN-000001</code></td></tr></tbody></table><p>Same numbering mechanism every built-in object already uses, just admin-configurable here.</p>',
          },
          {
            heading: "What you get automatically",
            bodyHtml:
              "<p>A new sidebar entry appears immediately, and the object works through the platform's existing machinery straight away: Custom Fields, Custom Relationships, Business Rules, Workflow Automation, Screen Builder, App Builder - all the same tools, no separate onboarding per object.</p>",
          },
          {
            heading: "Deactivate vs. delete",
            bodyHtml:
              "<p>Delete is blocked outright while any records exist - no accidental data loss path. Deactivate is always safe and reversible: hides the object from navigation without touching a record.</p>",
          },
        ],
      },
      {
        slug: "custom-relationships",
        title: "Custom Relationships: connecting any two objects",
        summary: "One-to-one, many-to-one or many-to-many links between any two record types, with automatic related-records panels.",
        sections: [
          {
            heading: "Connect any two objects — Data Model → Relationships",
            bodyHtml:
              '<p>Pick a Source (the "many"/owning side) and a Target - either can be built-in or Custom.</p><table><thead><tr><th>Cardinality</th><th>Meaning</th></tr></thead><tbody><tr><td>Many-to-one</td><td>Many source records, one target each</td></tr><tr><td>One-to-one</td><td>Exactly one on each side</td></tr><tr><td>Many-to-many</td><td>Either side links to several of the other</td></tr></tbody></table>',
          },
          {
            heading: "Forward & reverse labels",
            bodyHtml:
              '<p>A <b>Forward label</b> (shown on the source, e.g. "Primary Vendor") and a <b>Reverse label</b> (shown on the target, e.g. "Companies supplied") let the relationship read naturally from both directions.</p>',
          },
          {
            heading: "On delete: Restrict or Archive",
            bodyHtml:
              "<table><thead><tr><th>Behavior</th><th>Effect</th></tr></thead><tbody><tr><td>Restrict</td><td>Blocks archiving a linked record until unlinked</td></tr><tr><td>Archive</td><td>Drops the link automatically, keeps both records</td></tr></tbody></table>",
          },
          {
            heading: "Two more options",
            bodyHtml:
              '<p><b>Show as a related list on both records</b> - an automatic related-records panel on each side. <b>Source record should have a target linked</b> - makes the link itself required.</p>',
          },
          {
            heading: "Where it shows up",
            bodyHtml:
              "<p>Every linked record gets a \"Related records\" panel with inline Link/Unlink, both directions - the same lookup an AI Agent's <code>get_related_records</code> tool uses (see the Agentic AI Foundry category's Context Layer article).</p>",
          },
        ],
      },
      {
        slug: "custom-fields",
        title: "Custom Fields: types, validation & capability flags",
        summary: "Five field types, min/max/length/regex validation, and the Searchable/Filterable/Reportable flags.",
        sections: [
          {
            heading: "Field types — Data Model → Custom fields",
            bodyHtml:
              "<table><thead><tr><th>Type</th><th>Notes</th></tr></thead><tbody><tr><td>Text</td><td>Optional max length + regex pattern</td></tr><tr><td>Number</td><td>Optional min/max</td></tr><tr><td>Date</td><td>&nbsp;</td></tr><tr><td>Yes/No</td><td>Boolean</td></tr><tr><td>Select</td><td>A comma-separated option list</td></tr></tbody></table><p>Same builder for a built-in entity or a Custom Object.</p>",
          },
          {
            heading: "Validation",
            bodyHtml:
              "<p>Text: maximum length and a regex pattern. Number: minimum and maximum. Both enforced at save time, on top of a plain Required toggle every type carries.</p>",
          },
          {
            heading: "The capability flags",
            bodyHtml:
              "<table><thead><tr><th>Flag</th><th>Plugs into</th></tr></thead><tbody><tr><td>Searchable</td><td>Global search results</td></tr><tr><td>Filterable</td><td>List-view filters</td></tr><tr><td>Reportable</td><td>Custom Reports group-by (on by default)</td></tr><tr><td>Unique</td><td>Rejects a duplicate value (not on Yes/No)</td></tr><tr><td>Hidden by default</td><td>Omitted from a layout unless Screen Builder places it</td></tr></tbody></table>",
          },
          {
            heading: "Default, placeholder & help text",
            bodyHtml:
              "<p>Default value fills an empty save automatically; placeholder text shows inside an empty input; help text renders under the field - three independent settings for a self-explanatory form.</p>",
          },
          {
            heading: "Changing a field later",
            bodyHtml:
              "<p>Deactivating a field an active Business Rule or Workflow still reads or writes shows a dependency warning first, naming exactly which ones.</p>",
          },
        ],
      },
      {
        slug: "business-rules",
        title: "Business Rules: conditions, effects & status transitions",
        summary: "AND/OR logic across 12 operators, validation and field-behavior effects, priority, and restricting status changes.",
        sections: [
          {
            heading: "The condition/effect model — Automation → Business rules",
            bodyHtml:
              "<p>IF (conditions, AND or OR, plus one level of nested OR-groups) THEN (any number of effects) - evaluated live in the form and enforced again server-side on save, so it can't be bypassed by skipping the UI.</p>",
          },
          {
            heading: "Conditions: 12 operators",
            bodyHtml:
              "<p>equals · does not equal · contains · does not contain · starts with · ends with · is one of · is not one of · is empty · is not empty · is greater than · is less than.</p>",
          },
          {
            heading: "Effects",
            bodyHtml:
              "<table><thead><tr><th>Group</th><th>Effects</th></tr></thead><tbody><tr><td>Validation</td><td>Require · Block save (with message) · Show error · Show warning</td></tr><tr><td>Field behavior</td><td>Show/Hide · Read-only/Editable · Set value · Clear value · Set default · Restrict choices</td></tr></tbody></table>",
          },
          {
            heading: "Priority & effective-date windows",
            bodyHtml:
              "<p>When two active rules disagree on the same field, the higher-priority one wins. An optional effective start/end date window lets a rule apply only during a period, without manually toggling it.</p>",
          },
          {
            heading: "Status Transitions",
            bodyHtml:
              '<p>A separate, narrower tool in the same Automation area: which status/stage changes are allowed, with a wildcard "Any status" start and a per-rule active toggle. No active rules leaves the field unrestricted; re-saving the same status is never blocked.</p>',
          },
          {
            heading: "Test rule before activating",
            bodyHtml:
              "<p>A dry-run Test rule mode shows exactly what a rule would do against hypothetical values - without touching real data.</p>",
          },
        ],
      },
      {
        slug: "workflow-automation",
        title: "Workflow Automation: triggers & actions",
        summary: "7 trigger types, actions from creating a task to running an AI Agent, and the same condition engine as Business Rules.",
        sections: [
          {
            heading: "Triggers — Automation → Workflow automation",
            bodyHtml:
              "<table><thead><tr><th>Trigger</th><th>Fires when</th></tr></thead><tbody><tr><td>Record created</td><td>A new record is saved</td></tr><tr><td>Record updated</td><td>Any field changes</td></tr><tr><td>Status/stage reaches</td><td>A specific value is set</td></tr><tr><td>Custom field changed</td><td>A specific field changes</td></tr><tr><td>Date reached</td><td>A date field hits today (with an offset)</td></tr><tr><td>Due/overdue</td><td>A due date passes uncompleted</td></tr><tr><td>Recurring schedule</td><td>Every N days</td></tr></tbody></table>",
          },
          {
            heading: "Extra conditions",
            bodyHtml:
              "<p>Beyond the trigger, a workflow carries the same AND/OR condition engine Business Rules uses (including nested OR-groups), so a trigger can be narrowed without a second mechanism to learn.</p>",
          },
          {
            heading: "Actions",
            bodyHtml:
              "<p>Create a task/reminder (due/remind-in days, assignee), update or clear a field (fixed value or copied from another field), set a default only if empty, assign an owner, create a new linked record or update one on an existing link, notify the owner or all admins, or <b>Run AI agent</b> - firing an Agent or Pipeline run from a business event (see Orchestration in the Agentic AI Foundry category).</p>",
          },
          {
            heading: "Test workflow before activating",
            bodyHtml:
              "<p>Test workflow shows what an active workflow would do against hypothetical values - no real task, notification, or data touched.</p>",
          },
        ],
      },
      {
        slug: "screen-app-builder",
        title: "Screen Builder: designing create/edit and detail layouts",
        summary: "Tabs of field sections, role-based assignment, and a draft-until-published model that also drives the detail view.",
        sections: [
          {
            heading: "Tabs, sections & columns — Experience → Screen layouts",
            bodyHtml:
              "<p>A layout is named tabs, each holding field sections in 1-3 columns, any field able to span full width. Anything no tab claims still shows in an always-visible spot rather than disappearing.</p>",
          },
          {
            heading: "Assign by role, with a required Default",
            bodyHtml:
              "<p>Whichever roles a user has, the first published layout naming one wins; every object needs one Default as the fallback - the same resolution rule Dashboards use.</p>",
          },
          {
            heading: "Nothing changes until Publish",
            bodyHtml:
              '<p>Edits only touch the draft - the live form keeps its last-published version until Publish. An "Unpublished changes" badge shows drift; Revert draft discards unpublished edits.</p>',
          },
          {
            heading: "Placing a related list on a tab",
            bodyHtml:
              "<p>A Custom Relationship's related-records list can be placed on a specific tab instead of always appearing in a fixed spot.</p>",
          },
          {
            heading: "Drives the detail view too",
            bodyHtml:
              "<p>A published layout also renders the record's read-only Overview, so a field you've placed is visible there too. Preview shows the draft rendered before you publish.</p>",
          },
        ],
      },
      {
        slug: "apps-dashboards",
        title: "App Builder & Dashboards: publishing your own app",
        summary: "Group objects into a named app with access grants and a sidebar switcher, and build role-assigned dashboards.",
        sections: [
          {
            heading: "App Builder — Admin → Apps",
            bodyHtml:
              "<p>An app is a named, iconed group of already-existing objects plus a layout scope and an optional dashboard. Every primitive it assembles already exists elsewhere in Admin - App Builder packages and scopes, it doesn't duplicate.</p>",
          },
          {
            heading: "Draft, Publish, and access grants",
            bodyHtml:
              "<p>An app starts Draft and is invisible until Publish. Once published, Administrators always see every app; everyone else needs a grant - to a role or one person - at Viewer or Editor level. Zero grants means genuinely invisible to everyone but Administrators.</p>",
          },
          {
            heading: "Editor is a real security boundary",
            bodyHtml:
              "<p>Once an object is in a published app, every create/update/archive/status action on it needs at least Editor access to some app containing it, checked server-side - not only by which button is visible. The strongest matching grant wins; a person-specific grant beats their role's.</p>",
          },
          {
            heading: "Dashboards — Admin → Dashboards",
            bodyHtml:
              "<table><thead><tr><th>Widget</th><th>Shows</th></tr></thead><tbody><tr><td>KPI tile</td><td>A single number from the standard KPI catalog</td></tr><tr><td>Chart</td><td>An existing saved Custom Report as a bar chart</td></tr><tr><td>Record list</td><td>An object's most recent (or soonest-due) records, click to jump to one</td></tr></tbody></table><p>Same named-layout/role-assignment/required-Default/draft-until-Publish model as Screen Builder.</p>",
          },
          {
            heading: "App-scoped automation",
            bodyHtml:
              "<p>A rule, workflow or dashboard created in an app context is tagged with the app that owns it, and both builders offer an App filter - so an app's own automation stays visibly contained as you build more.</p>",
          },
        ],
      },
    ],
  },
];
