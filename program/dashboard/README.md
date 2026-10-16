# Program dashboard

One page with everything about the build: every requirement, every work package with its full
details and handoff history, the build sequence with gates, who built what, what is next, and the
"when a work package finishes" checklist.

- **Live page:** https://claude.ai/artifact/TANp9Vzmur4A37bsgHEtKg (owner shares it from the page's Share menu)
- **Source:** `index.html` here; its data is generated, never edited by hand

## Refresh it (Claude Code, at the start or end of a session)

```bash
python3 program/tools/programctl.py export --out /tmp/data.json
```

Then publish `program/dashboard/index.html` with `data.json` as a supporting file **to the URL above**
(Artifact tool: `url` = the live page, `files` = {"data.json": "/tmp/data.json"}). Publishing without
the URL creates a second page; do not do that.

Muse AI cannot publish the page. During a Muse-only week, `program/STATUS.md` and the GitHub Issues
mirror stay current; the dashboard catches up at the next Claude Code session.
