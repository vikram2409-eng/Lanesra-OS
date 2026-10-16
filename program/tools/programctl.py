#!/usr/bin/env python3
"""programctl - the build program's control tool.

Every AI agent and human working on the Agent Playground build uses this one
tool to read and check the program state kept in program/. Standard library
only, so it runs anywhere Python 3.9+ does.

    python3 program/tools/programctl.py validate        # check every tracking file
    python3 program/tools/programctl.py status          # regenerate program/STATUS.md
    python3 program/tools/programctl.py status --check  # fail if STATUS.md is stale (CI)
    python3 program/tools/programctl.py next            # work packages ready to start
    python3 program/tools/programctl.py show WP-0101    # one work package + its deps
    python3 program/tools/programctl.py trace           # requirement -> WP -> test coverage
    python3 program/tools/programctl.py new-log --agent claude --wp WP-0101
    python3 program/tools/programctl.py brief WP-0160       # self-contained task card to hand an AI
    python3 program/tools/programctl.py export --out data.json  # all program data for the dashboard page
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROGRAM = ROOT / "program"
WORK = PROGRAM / "work"
REQS = PROGRAM / "spec" / "REQUIREMENTS.md"
STATUS = PROGRAM / "STATUS.md"
LOGS = PROGRAM / "log"
TEST_DIRS = [ROOT / "desktop", ROOT / "tests", ROOT / "program" / "tools"]

STATUSES = ["proposed", "ready", "in_progress", "in_review", "blocked", "done", "dropped"]
PHASES = ["0", "1", "2", "3", "4", "v2"]
ESTIMATES = ["S", "M", "L", "XL"]
EDITIONS = ["desktop", "workspace", "demo"]  # Tauri desktop app, Team Workspace server, online demo
EDITION_NAMES = {"desktop": "Desktop", "workspace": "Team Workspace", "demo": "Online demo"}
REQUIRED_FIELDS = ["id", "title", "phase", "epic", "status", "owner", "depends_on",
                   "requirements", "estimate", "editions", "updated"]
REQUIRED_SECTIONS_READY = ["## Goal", "## Acceptance criteria", "## Test plan"]
WP_ID = re.compile(r"^WP-\d{4}$")
REQ_LINE = re.compile(r"^- \*\*(REQ-[A-Z]+-\d{3})\*\* \[P(\d|v2)\] (.+)$")
REQ_TAG = re.compile(r"\b(REQ-[A-Z]+-\d{3})\b")
WP_TAG = re.compile(r"\b(WP-\d{4})\b")


class Problem(Exception):
    pass


# ---------------------------------------------------------------- parsing ---

def parse_scalar(raw: str):
    raw = raw.strip()
    if raw.startswith("[") and raw.endswith("]"):
        inner = raw[1:-1].strip()
        return [x.strip().strip("'\"") for x in inner.split(",") if x.strip()] if inner else []
    if raw in ("", "~", "null"):
        return ""
    return raw.strip("'\"")


def parse_front_matter(path: Path):
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        raise Problem(f"{rel(path)}: missing front matter (must start with '---')")
    end = text.find("\n---\n", 4)
    if end == -1:
        raise Problem(f"{rel(path)}: front matter is not closed with '---'")
    meta = {}
    for n, line in enumerate(text[4:end].splitlines(), start=2):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        if ":" not in line:
            raise Problem(f"{rel(path)}:{n}: expected 'key: value', got {line!r}")
        key, val = line.split(":", 1)
        meta[key.strip()] = parse_scalar(val)
    return meta, text[end + 5:]


def rel(p: Path) -> str:
    try:
        return str(p.relative_to(ROOT))
    except ValueError:
        return str(p)


def load_requirements():
    reqs, order = {}, []
    if not REQS.exists():
        return reqs, order
    for n, line in enumerate(REQS.read_text(encoding="utf-8").splitlines(), start=1):
        m = REQ_LINE.match(line)
        if m:
            rid, phase, text = m.groups()
            if rid in reqs:
                raise Problem(f"{rel(REQS)}:{n}: duplicate requirement {rid}")
            reqs[rid] = {"id": rid, "phase": phase, "text": text, "line": n}
            order.append(rid)
        elif line.startswith("- **REQ-"):
            raise Problem(f"{rel(REQS)}:{n}: malformed requirement line; expected "
                          "'- **REQ-AREA-NNN** [P1] text'")
    return reqs, order


def load_work_packages():
    wps = {}
    for path in sorted(WORK.rglob("WP-*.md")):
        meta, body = parse_front_matter(path)
        meta["_path"] = path
        meta["_body"] = body
        wid = meta.get("id", "")
        if wid in wps:
            raise Problem(f"{rel(path)}: duplicate id {wid} (also {rel(wps[wid]['_path'])})")
        wps[wid] = meta
    return wps


# ------------------------------------------------------------- validation ---

def validate(wps, reqs, strict_coverage=True):
    errors, warnings = [], []
    for wid, wp in wps.items():
        where = rel(wp["_path"])
        for f in REQUIRED_FIELDS:
            if f not in wp:
                errors.append(f"{where}: missing field '{f}'")
        if not WP_ID.match(wid):
            errors.append(f"{where}: id {wid!r} must look like WP-0123")
        if not wp["_path"].name.startswith(wid + "-"):
            errors.append(f"{where}: file name must start with '{wid}-'")
        if wp.get("status") not in STATUSES:
            errors.append(f"{where}: status {wp.get('status')!r} not in {STATUSES}")
        if str(wp.get("phase")) not in PHASES:
            errors.append(f"{where}: phase {wp.get('phase')!r} not in {PHASES}")
        if "agent" in wp:
            errors.append(f"{where}: remove the 'agent' field - work is not pre-assigned to an AI (ADR-0005)")
        eds, dlv = wp.get("editions") or [], wp.get("delivered") or []
        if not isinstance(eds, list) or any(e not in EDITIONS for e in eds):
            errors.append(f"{where}: editions {eds!r} must be a list drawn from {EDITIONS}")
        if not isinstance(dlv, list) or any(e not in eds for e in dlv):
            errors.append(f"{where}: delivered {dlv!r} must only list editions from 'editions'")
        if wp.get("status") == "done" and set(eds) - set(dlv):
            errors.append(f"{where}: done, but not delivered in {sorted(set(eds) - set(dlv))}")
        if wp.get("estimate") not in ESTIMATES:
            errors.append(f"{where}: estimate {wp.get('estimate')!r} not in {ESTIMATES}")
        expected_dir = f"P{wp.get('phase')}"
        if wp["_path"].parent.name != expected_dir:
            errors.append(f"{where}: phase {wp.get('phase')} work packages live in work/{expected_dir}/")
        try:
            dt.date.fromisoformat(str(wp.get("updated")))
        except ValueError:
            errors.append(f"{where}: updated must be an ISO date (YYYY-MM-DD)")
        for dep in wp.get("depends_on") or []:
            if dep not in wps:
                errors.append(f"{where}: depends_on {dep} does not exist")
        for rid in wp.get("requirements") or []:
            if rid not in reqs:
                errors.append(f"{where}: requirement {rid} is not in spec/REQUIREMENTS.md")
        status = wp.get("status")
        if status in ("ready", "in_progress", "in_review", "done"):
            for sec in REQUIRED_SECTIONS_READY:
                if sec not in wp["_body"]:
                    errors.append(f"{where}: status {status} needs a '{sec}' section")
        if status in ("in_progress", "in_review", "blocked") and wp.get("owner") in ("", "unassigned"):
            errors.append(f"{where}: status {status} needs an owner")
        if status in ("in_review", "done") and not wp.get("pr"):
            errors.append(f"{where}: status {status} needs a pr link")
        if status == "done":
            for dep in wp.get("depends_on") or []:
                if dep in wps and wps[dep].get("status") not in ("done", "dropped"):
                    errors.append(f"{where}: done, but dependency {dep} is {wps[dep].get('status')}")
            if "- [ ]" in section(wp["_body"], "## Acceptance criteria"):
                errors.append(f"{where}: done, but has unticked acceptance criteria")
        if status == "blocked" and "## Blocker" not in wp["_body"]:
            errors.append(f"{where}: status blocked needs a '## Blocker' section saying what unblocks it")
    errors += find_cycles(wps)

    covered = {r for wp in wps.values() if wp.get("status") != "dropped"
               for r in (wp.get("requirements") or [])}
    uncovered = [r for r in reqs if r not in covered]
    if uncovered:
        msg = f"{len(uncovered)} requirement(s) not covered by any work package: " + ", ".join(uncovered[:12])
        (errors if strict_coverage else warnings).append(msg + (" ..." if len(uncovered) > 12 else ""))
    return errors, warnings


def section(body: str, heading: str) -> str:
    i = body.find(heading)
    if i == -1:
        return ""
    rest = body[i + len(heading):]
    j = rest.find("\n## ")
    return rest if j == -1 else rest[:j]


def find_cycles(wps):
    errors, state = [], {}

    def visit(n, stack):
        state[n] = 1
        for d in wps[n].get("depends_on") or []:
            if d not in wps:
                continue
            if state.get(d) == 1:
                errors.append("dependency cycle: " + " -> ".join(stack + [n, d]))
            elif state.get(d) is None:
                visit(d, stack + [n])
        state[n] = 2

    for n in wps:
        if state.get(n) is None:
            visit(n, [])
    return errors


# ----------------------------------------------------------------- status ---

def claimed_branches():
    """WP ids that have a remote branch wp/WP-XXXX-*. A pushed branch is a claim."""
    try:
        out = subprocess.run(["git", "ls-remote", "--heads", "origin", "wp/*"], cwd=ROOT,
                             capture_output=True, text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if out.returncode != 0:
        return None
    claims = {}
    for line in out.stdout.splitlines():
        ref = line.split("\t")[-1].replace("refs/heads/", "")
        m = re.match(r"wp/(WP-\d{4})", ref)
        if m:
            claims.setdefault(m.group(1), []).append(ref)
    return claims


def ready_to_start(wps):
    out = []
    for wid, wp in sorted(wps.items()):
        if wp.get("status") != "ready":
            continue
        deps = wp.get("depends_on") or []
        if all(wps.get(d, {}).get("status") in ("done", "dropped") for d in deps):
            out.append(wid)
    return out


def render_status(wps, reqs):
    lines = ["# Build status", "",
             "> **Generated file - do not edit by hand.** Run "
             "`python3 program/tools/programctl.py status` after changing any work package. "
             "If this file conflicts in a merge, take either side and re-run the command.", ""]
    by_phase = {}
    for wp in wps.values():
        by_phase.setdefault(str(wp.get("phase")), []).append(wp)

    lines += ["## Phase summary", "",
              "| Phase | Work packages | Done | In progress / review | Ready | Blocked | Proposed | Points done / total | Progress |",
              "|---|---|---|---|---|---|---|---|---|"]
    names = {"0": "0 · Precursors", "1": "1 · Foundations", "2": "2 · Playground",
             "3": "3 · Teams and modes", "4": "4 · Enterprise scale", "v2": "v2 · Open interop"}
    weight = {"S": 1, "M": 2, "L": 4, "XL": 8}
    for ph in PHASES:
        items = [w for w in by_phase.get(ph, []) if w.get("status") != "dropped"]
        if not items:
            continue
        c = {s: sum(1 for w in items if w.get("status") == s) for s in STATUSES}
        total_w = sum(weight.get(w.get("estimate"), 2) for w in items)
        done_w = sum(weight.get(w.get("estimate"), 2) for w in items if w.get("status") == "done")
        pct = round(100 * done_w / total_w) if total_w else 0
        bar = "█" * (pct // 10) + "░" * (10 - pct // 10)
        lines.append(f"| {names[ph]} | {len(items)} | {c['done']} | {c['in_progress'] + c['in_review']} | "
                     f"{c['ready']} | {c['blocked']} | {c['proposed']} | {done_w} / {total_w} | `{bar}` {pct}% |")
    lines += ["", "## Edition coverage", "",
              "Every capability ships in each edition it applies to (REQ-PRE-015). A work package is done "
              "only when it is delivered in all of its editions.", "",
              "| Edition | Work packages targeting it | Delivered | Points delivered / total |", "|---|---|---|---|"]
    for ed in EDITIONS:
        tw = [w for w in wps.values() if ed in (w.get("editions") or []) and w.get("status") != "dropped"]
        dw = [w for w in tw if ed in (w.get("delivered") or [])]
        lines.append(f"| {EDITION_NAMES[ed]} | {len(tw)} | {len(dw)} | "
                     f"{sum(weight.get(w.get('estimate'), 2) for w in dw)} / {sum(weight.get(w.get('estimate'), 2) for w in tw)} |")
    lines += ["", "Progress is weighted by estimate points (S=1, M=2, L=4, XL=8). The timeline is driven by "
              "how many AI sessions and tokens are available, so forecasts use points delivered per session "
              "(see ROADMAP.md, Forecasting), not calendar weeks.", ""]

    active = [w for w in wps.values() if w.get("status") in ("in_progress", "in_review", "blocked")]
    lines += ["## Active now", ""]
    if active:
        lines += ["| WP | Title | Status | Owner | Branch | PR |", "|---|---|---|---|---|---|"]
        for w in sorted(active, key=lambda w: w["id"]):
            lines.append(f"| [{w['id']}]({rel_link(w)}) | {w['title']} | {w['status']} | {w.get('owner')} | "
                         f"`{w.get('branch') or '-'}` | {w.get('pr') or '-'} |")
    else:
        lines.append("Nothing in progress.")
    lines += ["", "## Ready to start (all dependencies done)", ""]
    ready = ready_to_start(wps)
    if ready:
        lines += ["Whichever AI you are using takes the first one that is not claimed (a `wp/WP-XXXX-*` branch "
                  "or a draft PR means claimed). Hand it over with `programctl brief WP-XXXX`.", "",
                  "| WP | Title | Phase | Estimate |", "|---|---|---|---|"]
        for wid in ready:
            w = wps[wid]
            lines.append(f"| [{wid}]({rel_link(w)}) | {w['title']} | {w['phase']} | {w['estimate']} |")
    else:
        lines.append("None.")

    covered = {}
    for w in wps.values():
        for r in w.get("requirements") or []:
            covered.setdefault(r, []).append(w)
    done_reqs = sum(1 for r in reqs if covered.get(r) and all(w.get("status") == "done" for w in covered[r]))
    lines += ["", "## Requirement coverage", "",
              f"- Spec requirements tracked: **{len(reqs)}**",
              f"- Mapped to a work package: **{sum(1 for r in reqs if r in covered)}**",
              f"- Delivered (every mapped work package done): **{done_reqs}**", ""]

    for ph in PHASES:
        items = sorted(by_phase.get(ph, []), key=lambda w: w["id"])
        if not items:
            continue
        lines += [f"## Phase {names[ph]}", "", "| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |",
                  "|---|---|---|---|---|---|---|"]
        for w in items:
            deps = ", ".join(w.get("depends_on") or []) or "-"
            eds = " ".join(("✓" if e in (w.get("delivered") or []) else "○") + EDITION_NAMES[e]
                           for e in (w.get("editions") or [])) or "—"
            lines.append(f"| [{w['id']}]({rel_link(w)}) | {w['title']} | {w.get('epic')} | "
                         f"{badge(w.get('status'))} | {w.get('estimate')} | {eds} | {deps} |")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def rel_link(w):
    return str(w["_path"].relative_to(PROGRAM))


def badge(s):
    return {"done": "✅ done", "in_progress": "🔨 in progress", "in_review": "👀 in review",
            "blocked": "⛔ blocked", "ready": "🟢 ready", "proposed": "⚪ proposed",
            "dropped": "~~dropped~~"}.get(s, s)


# ------------------------------------------------------------------ brief ---

def render_brief(w, wps, reqs):
    """A task card an AI with no repo access can work from: everything it needs, nothing it must look up."""
    wid, path = w["id"], rel(w["_path"])
    log = section(w["_body"], "## Progress log").strip()
    entries = [e for e in log.split("\n### ") if e.strip() and not e.strip().startswith("<!--")]
    last = ("### " + entries[-1].strip()) if entries and not entries[-1].startswith("###") else (entries[-1].strip() if entries else "No previous entry - you are the first agent on this work package.")
    deps = "\n".join(f"- {d}: {wps[d].get('status') if d in wps else 'missing'} - {wps.get(d, {}).get('title', '')}"
                     for d in w.get("depends_on") or []) or "- none"
    req_lines = "\n".join(f"- {r}: {reqs.get(r, {}).get('text', 'missing')}" for r in w.get("requirements") or []) or "- none"
    body_keep = "\n".join(part for part in [
        "## Goal" + section(w["_body"], "## Goal"),
        "## Scope" + section(w["_body"], "## Scope") if "## Scope" in w["_body"] else "",
        "## Out of scope" + section(w["_body"], "## Out of scope") if "## Out of scope" in w["_body"] else "",
        "## Acceptance criteria" + section(w["_body"], "## Acceptance criteria"),
        "## Test plan" + section(w["_body"], "## Test plan"),
        "## Notes" + section(w["_body"], "## Notes") if "## Notes" in w["_body"] else "",
    ] if part)
    today = dt.date.today().isoformat()
    return f"""# Task card: {wid} - {w['title']}

Lanesra OS Agent Playground build. Generated by `programctl brief {wid}` on {today} from `{path}`.
Phase {w['phase']} · Estimate {w['estimate']} · Status now: {w['status']}
Must ship in: {", ".join(EDITION_NAMES[e] for e in (w.get("editions") or [])) or "no product edition (program/test infrastructure)"}
Already delivered in: {", ".join(EDITION_NAMES[e] for e in (w.get("delivered") or [])) or "none yet"}

## Rules (from AGENTS.md and program/PROTOCOL.md)

- Enterprise grade, no shortcuts: meet every acceptance criterion and prove it with a test.
- Never skip, delete or weaken an existing test. Clean room: do not use CrewAI, Agent Framework,
  LangGraph or AutoGen source or prompts; use Lanesra words (Team, Playbook, Run, Step).
- The product is bring-your-own-model; tests never call a hosted model API.
- Touch only what this work package needs. If the spec is unclear, write the question in your
  handoff note instead of guessing.

## Depends on

{deps}

## Requirements it delivers

{req_lines}

## The work package

{body_keep}

## Where the previous agent left off

{last}

## What you must deliver

1. The code and test changes, committed on the work package's `wp/` branch with a draft PR (or, if you
   cannot commit, as one patch for the owner).
2. These edits to `{path}`, included in the same patch:
   - front matter: `status: in_review` if every acceptance criterion is met, else `in_progress`;
     `owner: <your agent name, e.g. claude or muse>`; `updated: {today}`
   - tick `[x]` only the acceptance criteria your tests prove; if you cannot run tests, write "(untested locally - see CI)" after them
   - deliver every edition listed above: desktop and Team Workspace share the Rust core and React UI
     (test both modes); the online demo (`app.js`) gets the same behaviour simulated on sample data,
     write tools always simulated, following the demo architecture ADR (WP-0013). Add each finished
     edition to `delivered:` after merge
   - you may stop part-way: then leave `status: in_progress` and make the Progress log entry precise enough
     that a different AI can continue from it tomorrow
   - append to `## Progress log`:
     ```
     ### {today} <agent>
     Done: ...
     State: untested locally - see CI / green / RED because ...
     Next: ...
     Gotchas: ...
     ```
3. A new session log `program/log/{today[:4]}/{today[5:7]}/{today[8:10]}-0000-<agent>-{wid}.md` (short: goal, done, next, points, tokens used if known).
4. Do not edit `program/STATUS.md` or GitHub Issues - they are regenerated automatically.
5. Before you call it finished, go through `program/FINISH-CHECKLIST.md` (prove it, record it,
   publish it). The items you cannot do without a shell (running the suite, opening the PR,
   regenerating STATUS.md) are done by CI and the tracker-upkeep workflow (WP-0018).
"""


# ----------------------------------------------------------------- export ---

WEIGHT = {"S": 1, "M": 2, "L": 4, "XL": 8}
PHASE_TITLES = {"0": "Precursors", "1": "Foundations", "2": "Playground", "3": "Teams and modes",
                "4": "Enterprise scale", "v2": "Open interop"}
LOG_HEAD = re.compile(r"^(\d{4}-\d{2}-\d{2})\s+(.+)$")


def progress_entries(body):
    text = section(body, "## Progress log")
    out = []
    for chunk in text.split("\n### ")[1:]:
        head, _, rest = chunk.partition("\n")
        m = LOG_HEAD.match(head.strip())
        date, who = (m.group(1), m.group(2)) if m else ("", head.strip())
        agent = who.split()[0].strip("()").lower() if who else ""
        out.append({"date": date, "agent": agent, "heading": who, "text": rest.strip()})
    return out


def criteria(body):
    out = []
    for line in section(body, "## Acceptance criteria").splitlines():
        m = re.match(r"^- \[([ xX])\] (.+)$", line.strip())
        if m:
            out.append({"done": m.group(1) != " ", "text": m.group(2)})
    return out


def sequence(wps):
    """Wave = longest dependency chain below a WP; critical path = heaviest chain by points."""
    wave, heavy, nxt = {}, {}, {}

    def depth(w):
        if w in wave:
            return wave[w]
        deps = [d for d in wps[w].get("depends_on") or [] if d in wps]
        wave[w] = 1 + max((depth(d) for d in deps), default=0)
        return wave[w]

    def weight(w):
        if w in heavy:
            return heavy[w]
        deps = [d for d in wps[w].get("depends_on") or [] if d in wps]
        best = max(deps, key=weight, default=None)
        heavy[w] = WEIGHT.get(wps[w].get("estimate"), 2) + (weight(best) if best else 0)
        nxt[w] = best
        return heavy[w]

    for w in wps:
        depth(w)
        weight(w)
    end = max(wps, key=lambda w: heavy[w])
    critical = []
    while end:
        critical.append(end)
        end = nxt.get(end)
    phase_rank = {ph: i for i, ph in enumerate(PHASES)}
    order = sorted(wps, key=lambda w: (phase_rank.get(str(wps[w].get("phase")), 9), wave[w], w))
    return order, wave, list(reversed(critical))


def load_checklist():
    path = PROGRAM / "FINISH-CHECKLIST.md"
    groups, cur = [], None
    if not path.exists():
        return groups
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("## "):
            cur = {"group": line[3:].strip(), "items": []}
            groups.append(cur)
        m = re.match(r"^- \[ \] \*\*(.+?)\*\* — (.+)$", line)
        if m and cur is not None:
            cur["items"].append({"name": m.group(1), "detail": m.group(2)})
    return groups


def load_gates():
    out = []
    gates_dir = PROGRAM / "gates"
    for p in sorted(gates_dir.glob("G*.md")) if gates_dir.exists() else []:
        text = p.read_text(encoding="utf-8")
        title = text.splitlines()[0].lstrip("# ").strip()
        m = re.search(r"Status: \*\*(.+?)\*\*", text)
        checks = re.findall(r"^\| \d+ \| (.+?) \|.*\| (☐|☑|✅|✔) \|$", text, re.M)
        out.append({"id": p.stem.split("-")[0], "title": title, "status": m.group(1) if m else "",
                    "file": rel(p), "checks": [{"text": c, "done": mark != "☐"} for c, mark in checks]})
    return out


def load_raid_open():
    path = PROGRAM / "RAID.md"
    out = []
    if not path.exists():
        return out
    for line in path.read_text(encoding="utf-8").splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if cells and re.match(r"^[QRAID]-\d{2}$", cells[0]) and len(cells) >= 3:
            status = cells[-1]
            if status.startswith(("Open", "Mitigation")):
                out.append({"id": cells[0], "text": cells[1], "status": status})
    return out


def git_info():
    def run(*a):
        try:
            return subprocess.run(["git", *a], cwd=ROOT, capture_output=True, text=True, timeout=10).stdout.strip()
        except (OSError, subprocess.TimeoutExpired):
            return ""
    return {"commit": run("rev-parse", "--short", "HEAD"), "branch": run("rev-parse", "--abbrev-ref", "HEAD"),
            "remote": run("config", "--get", "remote.origin.url")}


def export_data(wps, reqs):
    order, wave, critical = sequence(wps)
    tags = scan_test_tags()
    unblocks = {w: [] for w in wps}
    for w, wp in wps.items():
        for d in wp.get("depends_on") or []:
            if d in unblocks:
                unblocks[d].append(w)
    out_wps = []
    for i, w in enumerate(order, 1):
        wp = wps[w]
        entries = progress_entries(wp["_body"])
        built_by = []
        for e in entries:
            if e["agent"] and e["agent"] not in built_by:
                built_by.append(e["agent"])
        owner = wp.get("owner") or ""
        if owner and owner != "unassigned" and owner not in built_by:
            built_by.append(owner)
        out_wps.append({
            "seq": i, "wave": wave[w], "critical": w in critical, "id": w, "title": wp.get("title"),
            "phase": str(wp.get("phase")), "epic": wp.get("epic"), "status": wp.get("status"),
            "owner": owner, "estimate": wp.get("estimate"), "points": WEIGHT.get(wp.get("estimate"), 2),
            "depends_on": wp.get("depends_on") or [], "unblocks": sorted(unblocks[w]),
            "requirements": wp.get("requirements") or [], "branch": wp.get("branch") or "",
            "editions": wp.get("editions") or [], "delivered": wp.get("delivered") or [],
            "pr": wp.get("pr") or "", "updated": str(wp.get("updated")), "file": rel(wp["_path"]),
            "goal": section(wp["_body"], "## Goal").strip(), "scope": section(wp["_body"], "## Scope").strip(),
            "out_of_scope": section(wp["_body"], "## Out of scope").strip(),
            "criteria": criteria(wp["_body"]), "test_plan": section(wp["_body"], "## Test plan").strip(),
            "notes": section(wp["_body"], "## Notes").strip(), "blocker": section(wp["_body"], "## Blocker").strip(),
            "progress": entries, "built_by": built_by,
            "ready_now": w in ready_to_start(wps),
        })
    by_req = {}
    for wp in out_wps:
        for r in wp["requirements"]:
            by_req.setdefault(r, []).append(wp)
    out_reqs = []
    for r in reqs.values():
        ws = by_req.get(r["id"], [])
        if ws and all(x["status"] == "done" for x in ws):
            state = "delivered"
        elif any(x["status"] in ("in_progress", "in_review", "done") for x in ws):
            state = "in progress"
        else:
            state = "planned"
        out_reqs.append({"id": r["id"], "area": r["id"].split("-")[1], "phase": r["phase"], "text": r["text"],
                         "wps": [x["id"] for x in ws], "state": state, "tests": sorted(tags.get(r["id"], []))})
    return {
        "generated_at": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d %H:%M UTC"),
        "git": git_info(), "phases": [{"id": p, "title": PHASE_TITLES[p]} for p in PHASES],
        "critical_path": critical, "work_packages": out_wps, "requirements": out_reqs,
        "checklist": load_checklist(), "gates": load_gates(), "raid_open": load_raid_open(),
    }


# ------------------------------------------------------------------ trace ---

def scan_test_tags():
    """Map REQ ids and WP ids to test files that mention them in a comment."""
    tags = {}
    for base in TEST_DIRS:
        if not base.exists():
            continue
        for p in base.rglob("*"):
            if not p.is_file() or "node_modules" in p.parts or "target" in p.parts:
                continue
            name = p.name
            is_test = ("tests" in p.parts or "__tests__" in p.parts or ".test." in name
                       or ".spec." in name or name.startswith("test_"))
            if not is_test or p.suffix not in (".rs", ".ts", ".tsx", ".js", ".py"):
                continue
            try:
                text = p.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            for tag in set(REQ_TAG.findall(text)) | set(WP_TAG.findall(text)):
                tags.setdefault(tag, set()).add(rel(p))
    return tags


# ------------------------------------------------------------------- main ---

def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    v = sub.add_parser("validate", help="check every tracking file")
    v.add_argument("--allow-uncovered", action="store_true",
                   help="report uncovered requirements as warnings, not errors")
    s = sub.add_parser("status", help="regenerate program/STATUS.md")
    s.add_argument("--check", action="store_true", help="fail if STATUS.md is out of date")
    n = sub.add_parser("next", help="list work packages ready to start and not yet claimed")
    n.add_argument("--offline", action="store_true", help="skip the git ls-remote claim check")
    ex = sub.add_parser("export", help="write all program data as JSON for the dashboard page")
    ex.add_argument("--out", required=True)
    b = sub.add_parser("brief", help="print a self-contained task card for handing a WP to an AI")
    b.add_argument("wp")
    sh = sub.add_parser("show", help="show one work package")
    sh.add_argument("wp")
    t = sub.add_parser("trace", help="requirement -> work package -> test coverage")
    t.add_argument("--fail-on-gaps", action="store_true",
                   help="exit 1 if a done work package has a requirement no test mentions")
    nl = sub.add_parser("new-log", help="create a session log file from the template")
    nl.add_argument("--agent", required=True, help="e.g. claude, codex, gemini, human-vikram")
    nl.add_argument("--wp", default="general")
    args = ap.parse_args(argv)

    try:
        reqs, _ = load_requirements()
        wps = load_work_packages()
    except Problem as e:
        print(f"error: {e}", file=sys.stderr)
        return 1

    if args.cmd == "validate":
        errors, warnings = validate(wps, reqs, strict_coverage=not args.allow_uncovered)
        for w in warnings:
            print(f"warning: {w}")
        for e in errors:
            print(f"error: {e}", file=sys.stderr)
        print(f"{len(wps)} work packages, {len(reqs)} requirements, "
              f"{len(errors)} error(s), {len(warnings)} warning(s)")
        return 1 if errors else 0

    if args.cmd == "status":
        new = render_status(wps, reqs)
        if args.check:
            old = STATUS.read_text(encoding="utf-8") if STATUS.exists() else ""
            if old != new:
                print("error: program/STATUS.md is stale. Run: python3 program/tools/programctl.py status",
                      file=sys.stderr)
                return 1
            print("STATUS.md is up to date")
            return 0
        STATUS.write_text(new, encoding="utf-8")
        print(f"wrote {rel(STATUS)}")
        return 0

    if args.cmd == "next":
        claims = None if args.offline else claimed_branches()
        ready = ready_to_start(wps)
        if claims is None and not args.offline:
            print("note: could not reach origin; claims not checked (a remote wp/WP-XXXX-* branch is a claim)")
        shown = 0
        for wid in ready:
            if claims and wid in claims:
                continue
            w = wps[wid]
            print(f"{wid}  [P{w['phase']} {w['estimate']}]  {w['title']}\n        {rel(w['_path'])}")
            shown += 1
        if not shown:
            print("No unclaimed work package is ready. Check STATUS.md 'Active now' and blocked items.")
        return 0

    if args.cmd == "export":
        Path(args.out).write_text(json.dumps(export_data(wps, reqs), ensure_ascii=False, indent=1), encoding="utf-8")
        print(f"wrote {args.out}")
        return 0

    if args.cmd == "brief":
        w = wps.get(args.wp)
        if not w:
            print(f"error: no such work package {args.wp}", file=sys.stderr)
            return 1
        print(render_brief(w, wps, reqs))
        return 0

    if args.cmd == "show":
        w = wps.get(args.wp)
        if not w:
            print(f"error: no such work package {args.wp}", file=sys.stderr)
            return 1
        print(w["_path"].read_text(encoding="utf-8"))
        print("--- dependencies ---")
        for d in w.get("depends_on") or []:
            print(f"{d}: {wps[d].get('status') if d in wps else 'MISSING'}  {wps.get(d, {}).get('title', '')}")
        print("--- requirements ---")
        for r in w.get("requirements") or []:
            print(f"{r}: {reqs.get(r, {}).get('text', 'MISSING')}")
        dependants = [x for x, ww in wps.items() if args.wp in (ww.get("depends_on") or [])]
        print("--- unblocks ---\n" + (", ".join(dependants) or "nothing"))
        return 0

    if args.cmd == "trace":
        tags = scan_test_tags()
        gaps = 0
        print("| Requirement | Phase | Work packages | Tests mentioning it |\n|---|---|---|---|")
        for rid, r in reqs.items():
            ws = [w for w in wps.values() if rid in (w.get("requirements") or [])]
            tests = sorted(tags.get(rid, []))
            wtxt = ", ".join(f"{w['id']}({w.get('status')})" for w in ws) or "UNMAPPED"
            if ws and all(w.get("status") == "done" for w in ws) and not tests:
                gaps += 1
                wtxt += " **NO TEST**"
            print(f"| {rid} | {r['phase']} | {wtxt} | {', '.join(tests[:3]) or '-'} |")
        print(f"\n{gaps} delivered requirement(s) without a tagged test")
        return 1 if (gaps and args.fail_on_gaps) else 0

    if args.cmd == "new-log":
        now = dt.datetime.now(dt.timezone.utc)
        d = LOGS / now.strftime("%Y") / now.strftime("%m")
        d.mkdir(parents=True, exist_ok=True)
        slug = re.sub(r"[^a-z0-9-]+", "-", args.agent.lower()).strip("-")
        path = d / f"{now.strftime('%d-%H%M')}-{slug}-{args.wp}.md"
        tmpl = (PROGRAM / "templates" / "session-log.md").read_text(encoding="utf-8")
        path.write_text(tmpl.replace("{{date}}", now.strftime("%Y-%m-%d %H:%M UTC"))
                        .replace("{{agent}}", args.agent).replace("{{wp}}", args.wp), encoding="utf-8")
        print(f"wrote {rel(path)}")
        return 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
