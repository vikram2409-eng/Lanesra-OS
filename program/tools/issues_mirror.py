#!/usr/bin/env python3
"""One-way mirror of work packages to GitHub Issues (ADR-0004, WP-0017).

The work package files in program/work/ stay the source of truth. This
script makes GitHub Issues show the same state, so the plan can be browsed,
filtered and followed in the GitHub UI:

- one issue per work package, titled "[WP-XXXX] Title"
- labels: program, phase:N, status:<status>, est:<S|M|L|XL>
- one milestone per phase
- issue body regenerated from the file on every sync (edit the file, not the issue)
- done and dropped work packages close their issue; reopened ones reopen it

It runs from .github/workflows/program-issues.yml on every push to main that
changes program/work/, so the issues always reflect merged state.

    GITHUB_TOKEN=... python3 program/tools/issues_mirror.py --repo owner/name
    python3 program/tools/issues_mirror.py --repo owner/name --dry-run

Standard library only.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import programctl as pc  # noqa: E402

API = "https://api.github.com"
MARKER = "<!-- programctl:{wid} -->"
PHASE_NAMES = {"0": "Phase 0 · Precursors", "1": "Phase 1 · Foundations", "2": "Phase 2 · Playground",
               "3": "Phase 3 · Teams and modes", "4": "Phase 4 · Enterprise scale", "v2": "v2 · Open interop"}
STATUS_COLOURS = {"proposed": "d4d4d8", "ready": "16a34a", "in_progress": "635bff", "in_review": "8b5cf6",
                  "blocked": "ef4444", "done": "0e7490", "dropped": "a1a1aa"}
CLOSED_STATUSES = {"done", "dropped"}


# ------------------------------------------------------------- rendering ---

def labels_for(wp: dict) -> list[str]:
    return ["program", f"phase:{wp['phase']}", f"status:{wp['status']}", f"est:{wp['estimate']}"]


def wanted_labels(wps: dict) -> dict[str, str]:
    """Every label the mirror uses, with its colour."""
    out = {"program": "4338ca"}
    for ph in pc.PHASES:
        out[f"phase:{ph}"] = "c4b5fd"
    for st, colour in STATUS_COLOURS.items():
        out[f"status:{st}"] = colour
    for est in pc.ESTIMATES:
        out[f"est:{est}"] = "e4e4e7"
    return out


def render_body(wp: dict, repo: str, issue_numbers: dict[str, int], branch: str = "main") -> str:
    rel = wp["_path"].relative_to(pc.ROOT).as_posix()
    url = f"https://github.com/{repo}/blob/{branch}/{rel}"

    def ref(wid: str) -> str:
        return f"#{issue_numbers[wid]} ({wid})" if wid in issue_numbers else wid

    deps = ", ".join(ref(d) for d in wp.get("depends_on") or []) or "none"
    reqs = ", ".join(wp.get("requirements") or []) or "none"
    acs = pc.section(wp["_body"], "## Acceptance criteria").strip() or "_none yet_"
    goal = pc.section(wp["_body"], "## Goal").strip()
    lines = [
        MARKER.format(wid=wp["id"]),
        f"> **Mirror of [`{rel}`]({url}).** Edit the file in a pull request, not this issue - "
        "the next sync overwrites the body, labels and state.",
        "",
        "| Field | Value |", "|---|---|",
        f"| Status | `{wp['status']}` |",
        f"| Phase | {PHASE_NAMES.get(str(wp['phase']), wp['phase'])} |",
        f"| Epic | {wp.get('epic', '')} |",
        f"| Estimate | {wp.get('estimate')} |",
        f"| Owner | {wp.get('owner') or 'unassigned'} |",
        f"| Depends on | {deps} |",
        f"| Requirements | {reqs} |",
        f"| Branch | `{wp.get('branch') or '-'}` |",
        f"| Pull request | {wp.get('pr') or '-'} |",
        "",
        "## Goal", "", goal, "",
        "## Acceptance criteria", "", acs, "",
    ]
    return "\n".join(lines)


def plan_changes(wps: dict, existing: dict[str, dict], repo: str) -> list[dict]:
    """Pure function: what to create or update. `existing` maps WP id -> issue JSON."""
    numbers = {wid: issue["number"] for wid, issue in existing.items()}
    changes = []
    for wid in sorted(wps):
        wp = wps[wid]
        want = {
            "title": f"[{wid}] {wp['title']}",
            "body": render_body(wp, repo, numbers),
            "labels": labels_for(wp),
            "milestone_title": PHASE_NAMES.get(str(wp["phase"]), str(wp["phase"])),
            "state": "closed" if wp["status"] in CLOSED_STATUSES else "open",
            "state_reason": "not_planned" if wp["status"] == "dropped" else "completed",
        }
        issue = existing.get(wid)
        if issue is None:
            changes.append({"action": "create", "wid": wid, **want})
            continue
        have_labels = sorted(lbl["name"] for lbl in issue.get("labels", []))
        have_milestone = (issue.get("milestone") or {}).get("title")
        if (issue["title"] != want["title"] or (issue.get("body") or "") != want["body"]
                or have_labels != sorted(want["labels"]) or have_milestone != want["milestone_title"]
                or issue["state"] != want["state"]):
            changes.append({"action": "update", "wid": wid, "number": issue["number"], **want})
    return changes


# ----------------------------------------------------------------- GitHub ---

class GitHub:
    def __init__(self, repo: str, token: str):
        self.repo, self.token = repo, token

    def call(self, method: str, path: str, payload: dict | None = None):
        url = path if path.startswith("http") else f"{API}/repos/{self.repo}{path}"
        data = json.dumps(payload).encode() if payload is not None else None
        req = urllib.request.Request(url, data=data, method=method, headers={
            "Authorization": f"Bearer {self.token}", "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "lanesra-programctl"})
        for attempt in range(4):
            try:
                with urllib.request.urlopen(req, timeout=30) as resp:
                    body = resp.read()
                    return json.loads(body) if body else None, resp.headers
            except urllib.error.HTTPError as e:
                if e.code in (403, 429, 502, 503) and attempt < 3:
                    time.sleep(2 ** (attempt + 2))
                    continue
                raise SystemExit(f"GitHub {method} {path} failed: {e.code} {e.read().decode(errors='ignore')[:300]}")
        return None, {}

    def paged(self, path: str):
        url = f"{API}/repos/{self.repo}{path}"
        while url:
            items, headers = self.call("GET", url)
            yield from items or []
            url = None
            for part in (headers.get("Link") or "").split(","):
                if 'rel="next"' in part:
                    url = part[part.find("<") + 1:part.find(">")]


def existing_mirror_issues(gh: GitHub) -> dict[str, dict]:
    out = {}
    for issue in gh.paged("/issues?state=all&labels=program&per_page=100"):
        if "pull_request" in issue:
            continue
        body = issue.get("body") or ""
        if body.startswith("<!-- programctl:WP-"):
            out[body[len("<!-- programctl:"):body.find(" -->")]] = issue
    return out


def ensure_labels_and_milestones(gh: GitHub, wps: dict) -> dict[str, int]:
    have = {lbl["name"] for lbl in gh.paged("/labels?per_page=100")}
    for name, colour in wanted_labels(wps).items():
        if name not in have:
            gh.call("POST", "/labels", {"name": name, "color": colour, "description": "Build program mirror"})
    milestones = {m["title"]: m["number"] for m in gh.paged("/milestones?state=all&per_page=100")}
    for title in PHASE_NAMES.values():
        if title not in milestones:
            created, _ = gh.call("POST", "/milestones", {"title": title, "description": "See program/ROADMAP.md"})
            milestones[title] = created["number"]
    return milestones


def main(argv=None):
    ap = argparse.ArgumentParser(description="Mirror work packages to GitHub Issues")
    ap.add_argument("--repo", required=True, help="owner/name")
    ap.add_argument("--dry-run", action="store_true", help="print the plan; no token needed")
    args = ap.parse_args(argv)
    wps = pc.load_work_packages()

    if args.dry_run:
        for c in plan_changes(wps, {}, args.repo):
            print(f"{c['action']:6} {c['title']}  [{', '.join(c['labels'])}]  {c['milestone_title']}  {c['state']}")
        print(f"{len(wps)} work packages")
        return 0

    token = os.environ.get("GITHUB_TOKEN")
    if not token:
        print("error: GITHUB_TOKEN is not set", file=sys.stderr)
        return 1
    gh = GitHub(args.repo, token)
    milestones = ensure_labels_and_milestones(gh, wps)
    existing = existing_mirror_issues(gh)

    # Create missing issues first so dependency links can point at numbers.
    for c in [c for c in plan_changes(wps, existing, args.repo) if c["action"] == "create"]:
        created, _ = gh.call("POST", "/issues", {"title": c["title"], "body": c["body"], "labels": c["labels"],
                                                 "milestone": milestones[c["milestone_title"]]})
        existing[c["wid"]] = created
        print(f"created #{created['number']} {c['title']}")
        time.sleep(1)  # stay well inside GitHub's content-creation rate limit

    updated = 0
    for c in plan_changes(wps, existing, args.repo):
        payload = {"title": c["title"], "body": c["body"], "labels": c["labels"],
                   "milestone": milestones[c["milestone_title"]], "state": c["state"]}
        if c["state"] == "closed":
            payload["state_reason"] = c["state_reason"]
        gh.call("PATCH", f"/issues/{c['number'] if 'number' in c else existing[c['wid']]['number']}", payload)
        updated += 1
    print(f"{len(existing)} mirrored issues, {updated} updated")
    return 0


if __name__ == "__main__":
    sys.exit(main())
