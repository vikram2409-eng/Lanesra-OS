# Lanesra OS

[![License: MIT](https://img.shields.io/github/license/vikram2409-eng/Lanesra-OS)](LICENSE)
[![Latest desktop release](https://img.shields.io/github/v/release/vikram2409-eng/Lanesra-OS?include_prereleases&label=desktop%20release)](https://github.com/vikram2409-eng/Lanesra-OS/releases)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Try the demo](https://img.shields.io/badge/try%20it-online%20demo-4f7cff)](https://lanesraos.com/demo)

Modern, open-source, AI-native business application platform — an AI Agent Foundry and Unified AI Gateway where named agents run directly against your real data, not a bolted-on chatbot; a native MCP server & CLI that expose that same data to Claude Desktop, an IDE agent or any MCP-capable client; the same no-code admin panel that lets you reshape the workspace itself (your own record types, relationships, screens, business rules and automations, not just the fixed CRM fields); a complete CRM out of the box; and an Industry Data Model of 11 ready-made industry apps you can install instead of building from scratch. Deployment Management packages and promotes your customizations between workspaces, and an Integration Hub connects it to everything else you run. Run it privately on Windows (offline, no cloud account, no licence key), share it with a small team over your local network, or try it instantly online with no install at all.

**[Try the demo](https://lanesraos.com/demo)** · **[Download](#download-the-desktop-edition)** · **[Features](#features)** · **[Docs](desktop/README.md)** · **[Contributing](CONTRIBUTING.md)**

## Features

**AI Agent Foundry — agents that run directly on your real data:**

- **Unified AI Gateway** — bring your own key (Anthropic, an OpenAI-compatible endpoint, Google Gemini, or a fully local/air-gapped model via Ollama), no markup or reselling; automatic primary → fallback → local-fallback failover, per-agent and per-workspace daily token budgets, and forced air-gapping the moment a request touches a sensitive data class you configure
- **AI Agent Foundry** — named, admin-defined AI Agents: a persona layered over a per-tool Actions checklist, a persistent Memory document the agent revises itself, a shared Skills library, operational guardrails, and delegation to other agents (depth-guarded against runaway recursion); export/import an agent or Skill as part of a Solution, exactly like a Custom Object
- **Orchestration** — chain agents into a deterministic Pipeline (sequential, consensus, or a peer-review loop); fire it manually, on a schedule, from an authenticated webhook, or as a new Workflow Automation action, with human-in-the-loop approval gates at any topology's own resume point and OTLP tracing on every run — every run lands in one unified history
- **Evaluation Harness** — a named Suite of golden test cases, each graded by an LLM-as-judge call against an Agent or Pipeline's real response, not a brittle exact-match
- **Chat Assistant** — a conversational assistant with real tool access to your records, and, for an Administrator, to the admin configuration surface itself, including an agent context primer (relationship metadata and a live platform overview) so an agent understands how your own workspace is actually built
- **Context Layer & Vector Search** — real change history for an agent's own Memory, ranked full-text search over Custom Object records (SQLite FTS5), and optional embeddings-based semantic search via your own already-configured provider — no bundled vector database
- **Agent Versioning & Approvals** — a real Draft → Test → Published → Deprecated → Disabled lifecycle per agent (a Published version is an immutable snapshot, optionally declaring a Structured Output JSON-schema contract), plus a durable, workspace-wide Approval inbox generalizing "something needs a human decision" beyond a single Pipeline step
- **Policy Engine & Tool-Call Firewall** — a Tool Registry classifying every tool (native, Connector-bridged or MCP-sourced) into one of six risk tiers, a workspace/per-agent Policy setting a require-approval threshold and blocklist on top of it, and a formalized `schema → permission → policy → approval → execute → audit` dispatch every tool call already runs through — governance in front of the same write, never a second enforcement path
- **Shared Execution Graph Runtime & Agent Teams** — a durable, checkpointed graph engine (Trigger/Condition/Action/Agent/Router/Parallel Split/Join/Approval/Delay/Transform/Loop/End nodes) now underlying every Workflow and Pipeline, resumable after a kill mid-run, plus a free-form drag-and-drop **Agent Teams** canvas to author one by hand with the same Draft → Publish lifecycle and a Runs viewer
- **Memory Architecture & Document RAG** — Session, Working and Entity memory (each scoped, TTL'd and Policy-gated) alongside an agent's own persistent Memory document, written/read via explicit `remember`/`get_memory` tool calls; Knowledge Collections and Sources are chunked, embedded and searchable via `search_knowledge`, which always returns a source citation

**MCP — connect external AI agents and scripts, or call out to theirs:**

- **Native Model Context Protocol server** — a stateless `POST /mcp` endpoint over JSON-RPC 2.0, exposing your Custom Objects, records and business logic to Claude Desktop, an IDE agent, or any MCP-capable client, authenticated with the same scoped API keys the REST API already uses
- **MCP client** — the same protocol in reverse: connect an external MCP server as a Connection, discover its tools via real `initialize`/`tools/list` JSON-RPC calls, and opt them in as agent-callable tools (read-only by default, a two-level write gate for anything destructive) through the identical Tool-Call Firewall above
- **CLI** — a companion `lanesra` command-line tool giving humans and shell scripts that same access without an agent in the loop, over the same REST API

**No-Code Platform — make it yours:**

- **Custom Objects** — define an entirely new record type (Vendors, Assets, Projects, …) with its own fields, ID format and navigation section, no code change
- **Custom Relationships** — connect any two record types (built-in or custom) with one-to-one, many-to-one or many-to-many links, including a self-referential hierarchy on one type or a polymorphic "any type" target; a related-records list appears automatically on both sides
- **Business Rules** — multi-condition AND/OR logic across 10 operators, driving require/hide/lock/set-value/block-save/show-message effects on any field, including a condition that reads a related record's own field through a relationship
- **Workflow Automation** — trigger on a status or field change, a date reached or overdue (any custom object's own date field, not just a fixed built-in set), or a schedule; create a task, assign an owner, create a related record, update a field, or post an in-app notification
- **Custom fields** with validation (min/max, length, regex) and capability flags, on every major object, built-in or custom
- **App Builder** — group a set of objects, their screens and a dashboard into one named, publishable app; grant it to roles or users as Viewer or Editor, enforced server-side on every create/edit/archive and status-lifecycle action, not just hidden in the UI
- **Theme Studio** — brand colors, typography, shape and density for the whole workspace, from 4 curated presets or from scratch, with Draft → Published → Archived versioning, rollback, and a real WCAG contrast check gating every publish

**Organization & Access Control — who owns what, and who reports to whom:**

- **Organizations & Organization Units** — a hierarchy of Organization Units (regions, divisions, teams) under one Organization per workspace, with create/edit and a move-with-impact-preview (descendant units and owned records affected, before you commit)
- **Work Teams** — named teams with time-bounded memberships; ending a membership never reassigns anything a team member owned
- **Record ownership** — every Company, Contact, Opportunity, Product, Quote, Order, Invoice, Contract, Task and Custom Object record carries a structured Owner (a user or a team) and an Owning Organization Unit, defaulted on create and changeable individually or via bulk reassignment, with a full transfer history
- **Voice-First Mode** — a PIN-gated, risk-confirmed voice channel into your records: a 4-digit Voice PIN unlocks a time-boxed session, 8 per-role capability toggles plus a Max Voice Action Level and a Processing Boundary compose with (only ever narrow, never widen) the Access Roles above, and every voice-triggered write calls the exact same service a manual edit does, so Business Rules and Workflow Automation still apply — reads and navigation execute immediately, a status/field change always shows an explicit Before → After Confirm/Reject step, and delete-class actions are hard-blocked outright. Voice-triggered AI Agents & Pipelines, multi-turn conversational context, guided step-by-step record creation, industry-specific status vocabulary, and an optional admin-opt-in LLM-backed conversational layer (an unrecognized phrasing is rewritten by your own configured provider, then fed back through the identical permission/rule/confirmation pipeline) round out the full rollout — see the [roadmap](https://lanesraos.com/roadmap) for what's proposed next
- **Access Control v1** — capability-scoped Access Roles (per-object create/read/update/delete/assign) and a Record Scope (Owner-only, Owner's Team, Owner's Org Unit and below, or Organization-wide) enforced on every create/update/delete/single-record-read across all owned objects and Custom Objects, plus list views, global search, dashboard widgets and report totals; an Access Inspector explains why a user can or can't see a given record from the identical evaluation trace enforcement itself uses

**The core CRM — your Business OS:**

- **Companies, Contacts & Sales Pipeline** — connected customer records with a Kanban/list opportunity pipeline
- **Products & Services**, **Quotes**, **Orders**, **Invoices**, **Contracts**, **Tasks** — the full flexible sales lifecycle (Company → Opportunity → Quote → Order → Invoice), plus direct-quote/direct-order shortcuts
- **Dashboard & global search** with clickable, filterable KPIs
- **Reports** beyond the dashboard (revenue by month, win rate, AR aging, sales by owner), plus a simple report builder — including on custom fields and Custom Objects
- **Branding & print customization** — logo, business profile, and PDF/print output for quotes, orders and invoices
- **Admin panel**: user accounts and roles, admin-configurable ID/numbering formats per object (e.g. `CUS-000001` → `ACC-000001`), dashboard KPI picker
- Windows task reminder notifications, session inactivity auto-lock
- CSV import/export, whole-workspace backup & restore, self-service password change
- Runs fully offline — no cloud account, licence key, or mandatory internet connection

**Industry Data Model — install instead of building from scratch:**

- **Industry Data Model** — a versioned package manifest format (objects, fields, relationships, business rules, workflows, screens, reports and a dashboard, with optional sample data) installed into an existing workspace, reusing your existing Company/Contact/Task core rather than creating a parallel data model
- **App Catalog** — install one of 11 ready-made industry apps (Field Service, Property Management, Construction, Professional Services, Practice Administration, Recruitment, Real Estate, Legal Practice, Nonprofit & Association, Auto Repair, Policy Administration & Claims Management) with a validated, backed-up, transactional install
- **Lanesra Industry Foundation** (desktop only) — a shared cross-industry package (Party, Party Role, Party Relationship, Location, Asset, Agreement and more) other industry packages can optionally declare a real, enforced dependency on and relate to, instead of each reinventing its own version

**Also included:**

- **Deployment Management** — a Publisher registry, named/versioned Solutions curated from any component you've built, real export/import between workspaces, and update-with-diff — package and promote your customizations the way a real software vendor would
- **Integration Hub** — AES-256-GCM-encrypted Connections (REST/SFTP/PostgreSQL/OData/SMTP), OpenAPI-imported Connectors (a curated template gallery — OpenAI, Cohere, Slack, GitHub, SendGrid — to start from) usable as Workflow Automation actions or, once opted in, as AI Agent Foundry tools (read-only by default, write access behind a further explicit per-connector opt-in), a generic REST API with hashed/scoped API keys, HMAC-SHA256-signed Webhooks with retry, a generalized CSV data-exchange wizard, and scheduled Integration Jobs

## Try it online

No install, no registration: **[lanesraos.com/demo](https://lanesraos.com/demo)**

## Download the desktop edition

The Windows desktop edition (Tauri + Rust + SQLite) is in active Early Access. Grab the latest installer from **[GitHub Releases](https://github.com/vikram2409-eng/Lanesra-OS/releases)** (unsigned — Windows will warn on first run) or build it from source.

Full architecture, dev setup, and a detailed "what's here / what's deferred" breakdown live in **[`desktop/README.md`](desktop/README.md)** — read that before assuming a feature exists.

## Running it for a team (on-prem / LAN)

Lanesra OS ships in two operating modes from the same codebase:

- **Personal Workspace** — the desktop app above, single user or a shared PC
- **Team Workspace** — one machine on your network runs a small local server; everyone else opens it in a browser tab and signs in with their own account. No Docker required to try it, though a `Dockerfile`/`docker-compose.yml` are included for the recommended setup.

See **[`desktop/README.md`](desktop/README.md#two-operating-modes)** for exact commands (`cargo run` locally, or `docker compose up -d`) — this targets a local network, not the public internet; put it behind a reverse proxy with TLS if you need that.

## Repository layout

```
/                website + no-registration browser demo (this file's context)
/desktop         the Tauri + Rust + SQLite desktop app and Team Workspace
                  server - see desktop/README.md for the real technical README
```

The root of this repo is the public product website (`lanesraos.com`), a static site with a browser-only demo at `/demo` — it shares product language with the desktop edition but is a separate, simpler codebase (no backend, `localStorage`-based). It auto-deploys from `main` via Netlify.

## Public product pages

- `/platform` — What you can build on top of the CRM, illustrated with real examples
- `/compare` — A factual market-positioning comparison
- `/download` — Desktop platform status, what's available today, and what's still planned
- `/roadmap` — Shipped, in-progress and proposed work, plus the recommended build sequence (formerly split across separate Roadmap and Backlog pages)
- `/releases` — Release-by-release updates (formerly `/changelog`)

## Previewing the website locally

The root site is a single-page app with no build step:

```bash
python3 -m http.server 8080
```

Then open `http://localhost:8080`.

## Contributing

Contributions are welcome — see **[CONTRIBUTING.md](CONTRIBUTING.md)** for dev setup, the branch/PR workflow, and what to verify before opening a pull request. Please report security issues privately per **[SECURITY.md](SECURITY.md)** rather than as a public issue. This project follows the **[Contributor Covenant](CODE_OF_CONDUCT.md)**.

## License

MIT — see **[LICENSE](LICENSE)**. Created by [Vikram Grover](https://vikramgrover.com).
