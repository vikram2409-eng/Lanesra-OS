# Program dashboard

One page with everything about the build: every requirement, every work package with its full
details and handoff history, the build sequence with gates, who built what, what is next, and the
"when a work package finishes" checklist.

- **Always current:** https://vikram2409-eng.github.io/Lanesra-OS/ - rebuilt by the `Program dashboard` workflow on every merge to `main` (WP-0019). Public, like the repo.
- **claude.ai copy:** https://claude.ai/artifact/TANp9Vzmur4A37bsgHEtKg - refreshed by Claude Code at the start and end of every session; it states its data commit and time
- **Source:** `index.html` here; its data is generated, never edited by hand

## Refresh it (Claude Code, at the start or end of a session)

```bash
python3 program/tools/programctl.py export --out /tmp/data.json
```

Then publish `program/dashboard/index.html` with `data.json` as a supporting file **to the URL above**
(Artifact tool: `url` = the live page, `files` = {"data.json": "/tmp/data.json"}). Publishing without
the URL creates a second page; do not do that.

Muse AI cannot publish the claude.ai copy, but it does not need to: the GitHub Pages copy updates on
every merge, whichever AI did the work. The claude.ai copy catches up at the next Claude Code session.

## How the Pages copy is built

`.github/workflows/program-dashboard.yml` runs `programctl validate`, `programctl export` (data.json) and
`program/tools/dashboard_page.py` (adds the HTML skeleton the claude.ai host normally provides), then
deploys to GitHub Pages. Pages source must be "GitHub Actions" (set by the owner 2026-10-16).
