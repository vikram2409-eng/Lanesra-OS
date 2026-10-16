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
"""
from __future__ import annotations

import argparse
import datetime as dt
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
REQUIRED_FIELDS = ["id", "title", "phase", "epic", "status", "owner", "depends_on",
                   "requirements", "estimate", "updated"]
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
              "| Phase | Work packages | Done | In progress / review | Ready | Blocked | Proposed | Progress |",
              "|---|---|---|---|---|---|---|---|"]
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
                     f"{c['ready']} | {c['blocked']} | {c['proposed']} | `{bar}` {pct}% |")
    lines += ["", "Progress is weighted by estimate (S=1, M=2, L=4, XL=8).", ""]

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
        lines += ["| WP | Title | Phase | Estimate |", "|---|---|---|---|"]
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
        lines += [f"## Phase {names[ph]}", "", "| WP | Title | Epic | Status | Est | Depends on |",
                  "|---|---|---|---|---|---|"]
        for w in items:
            deps = ", ".join(w.get("depends_on") or []) or "-"
            lines.append(f"| [{w['id']}]({rel_link(w)}) | {w['title']} | {w.get('epic')} | "
                         f"{badge(w.get('status'))} | {w.get('estimate')} | {deps} |")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def rel_link(w):
    return str(w["_path"].relative_to(PROGRAM))


def badge(s):
    return {"done": "✅ done", "in_progress": "🔨 in progress", "in_review": "👀 in review",
            "blocked": "⛔ blocked", "ready": "🟢 ready", "proposed": "⚪ proposed",
            "dropped": "~~dropped~~"}.get(s, s)


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
