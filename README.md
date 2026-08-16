# Lanesra OS

[![License: MIT](https://img.shields.io/github/license/vikram2409-eng/Lanesra-OS)](LICENSE)
[![Latest desktop release](https://img.shields.io/github/v/release/vikram2409-eng/Lanesra-OS?include_prereleases&label=desktop%20release)](https://github.com/vikram2409-eng/Lanesra-OS/releases)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Try the demo](https://img.shields.io/badge/try%20it-online%20demo-4f7cff)](https://lanesraos.com/demo)

Modern, open-source sales and business management software for small businesses — with a no-code admin panel that lets you reshape the workspace itself: your own record types, relationships, business rules and automations, not just the fixed CRM fields. Run it privately on Windows (offline, no cloud account, no licence key), share it with a small team over your local network, or try it instantly online with no install at all.

**[Try the demo](https://lanesraos.com/demo)** · **[Download](#download-the-desktop-edition)** · **[Features](#features)** · **[Docs](desktop/README.md)** · **[Contributing](CONTRIBUTING.md)**

## Features

**Make it yours, no code required:**

- **Custom Objects** — define an entirely new record type (Vendors, Assets, Projects, …) with its own fields, ID format and navigation section, no code change
- **Custom Relationships** — connect any two record types (built-in or custom) with one-to-one, many-to-one or many-to-many links; a related-records list appears automatically on both sides
- **Business Rules** — multi-condition AND/OR logic across 10 operators, driving require/hide/lock/set-value/block-save/show-message effects on any field
- **Workflow Automation** — trigger on a status or field change, a date reached or overdue, or a schedule; create a task, assign an owner, create a related record, update a field, or post an in-app notification
- **Custom fields** with validation (min/max, length, regex) and capability flags, on every major object, built-in or custom
- **App Builder** — group a set of objects, their screens and a dashboard into one named, publishable app; grant it to roles or users as Viewer or Editor, enforced server-side on every create/edit/archive and status-lifecycle action, not just hidden in the UI

**The core CRM:**

- **Companies, Contacts & Sales Pipeline** — connected customer records with a Kanban/list opportunity pipeline
- **Products & Services**, **Quotes**, **Orders**, **Invoices**, **Contracts**, **Tasks** — the full flexible sales lifecycle (Company → Opportunity → Quote → Order → Invoice), plus direct-quote/direct-order shortcuts
- **Dashboard & global search** with clickable, filterable KPIs
- **Reports** beyond the dashboard (revenue by month, win rate, AR aging, sales by owner), plus a simple report builder — including on custom fields and Custom Objects
- **Branding & print customization** — logo, business profile, and PDF/print output for quotes, orders and invoices
- **Admin panel**: user accounts and roles, admin-configurable ID/numbering formats per object (e.g. `CUS-000001` → `ACC-000001`), dashboard KPI picker
- Windows task reminder notifications, session inactivity auto-lock
- CSV import/export, whole-workspace backup & restore, self-service password change
- Runs fully offline — no cloud account, licence key, or mandatory internet connection

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

- `/principles` — The product decisions and beliefs behind Lanesra OS
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
