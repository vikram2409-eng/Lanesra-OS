# Project subagents

These Claude Code subagents are sourced from [msitarzewski/agency-agents](https://github.com/msitarzewski/agency-agents)
(MIT-style community agent library), with a short "Applying This to Lanesra OS" section appended to
several of them to ground the generic agent in this repo's actual stack (Rust core/Tauri/Axum server,
React/TypeScript frontend, SQLite, vanilla-JS online demo).

| File | Role |
|---|---|
| `engineering-rust-refactoring-specialist.md` | Repository-scale Rust refactors — the `core`/`server`/`src-tauri` workspace |
| `engineering-desktop-app-engineer.md` | Tauri/Electron process isolation, signing, auto-update, native OS integration |
| `engineering-frontend-developer.md` | React/TypeScript UI, Core Web Vitals, WCAG 2.1 AA accessibility |
| `engineering-api-platform-engineer.md` | Contract-first API design, versioning/deprecation — the `/api/v1` REST + webhooks surface |
| `engineering-database-optimizer.md` | SQLite schema, indexing, N+1 detection, safe migrations |
| `security-appsec-engineer.md` | Threat modeling, secure code review, SAST/DAST — relevant to `secret_service`, API-key auth, webhook HMAC |
| `engineering-technical-writer.md` | README/release-notes/website copy accuracy and consistency |
| `engineering-git-workflow-master.md` | Branching/commit conventions, including this repo's "restart from `main` after merge" rule |
| `marketing-seo-specialist.md` | Technical SEO for `lanesraos.com` — meta tags, sitemap, structured data, `llms.txt` |
| `engineering-multi-agent-systems-architect.md` | Orchestration: which of the above agent(s) to use for a given task, in what order/topology |

Invoke one by name, e.g. *"Use the Database Optimizer agent to review this migration."* Claude Code
also auto-selects a matching agent for a request when its `description` fits.

Two files (`engineering-database-optimizer.md`, `engineering-technical-writer.md`) had frontmatter
that wasn't cleanly recoverable from the source fetch and was reconstructed to match the same format
used by every other file in the set — their bodies are otherwise the original content.
