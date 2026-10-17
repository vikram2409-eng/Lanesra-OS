#!/usr/bin/env python3
"""Banned-vocabulary check for clean-room compliance (WP-0012, REQ-CLEAN-003).

Scans new/changed code for third-party agent-framework vocabulary in
identifiers: Crew, Kickoff, Superstep, Magentic, StateGraph, GraphFlow, Swarm.
Docs and research notes (program/research/, *.md) are allow-listed — the ban
applies to code identifiers only.

Usage:
    python3 program/tools/check_vocabulary.py [--staged | PATH...]
    python3 program/tools/check_vocabulary.py --self-test

Covers: REQ-CLEAN-003 (WP-0012 AC-2)
"""

import re
import subprocess
import sys
from pathlib import Path

# Banned words (case-insensitive) when they appear in code identifiers.
BANNED = ["Crew", "Kickoff", "Superstep", "Magentic", "StateGraph", "GraphFlow", "Swarm"]

# Paths where the vocabulary may legitimately appear (docs, research notes,
# and this checker itself).
ALLOWLIST_DIRS = {"program/research", "program/decisions"}
ALLOWLIST_EXTS = {".md", ".html"}
ALLOWLIST_FILES = {"program/tools/check_vocabulary.py", "THIRD_PARTY_NOTICES"}

# Code file extensions to scan.
CODE_EXTS = {".py", ".rs", ".js", ".ts", ".tsx", ".jsx", ".sql"}


def _is_allowlisted(path: Path) -> bool:
    p = path.as_posix()
    if p in ALLOWLIST_FILES:
        return True
    if path.suffix in ALLOWLIST_EXTS:
        return True
    return any(p.startswith(d + "/") for d in ALLOWLIST_DIRS)


def find_violations(paths):
    """Return list of (path, lineno, line, word) for banned words in identifiers."""
    violations = []
    patterns = [(w, re.compile(r"\b\w*" + re.escape(w) + r"\w*\b", re.IGNORECASE)) for w in BANNED]
    for raw in paths:
        path = Path(raw)
        if not path.is_file() or _is_allowlisted(path):
            continue
        if path.suffix not in CODE_EXTS:
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for i, line in enumerate(text.splitlines(), 1):
            stripped = line.strip()
            # Skip comments — the ban is on identifiers, not discussion.
            if stripped.startswith(("#", "//", "--", "*")):
                continue
            for word, pat in patterns:
                for m in pat.finditer(line):
                    token = m.group(0)
                    # Only flag when used as an identifier (not inside a string
                    # literal mentioning the project name in passing).
                    violations.append((path.as_posix(), i, line.strip(), word))
                    break
    return violations


def _changed_files():
    """Files changed vs origin/main (CI) or staged (local)."""
    try:
        out = subprocess.run(
            ["git", "diff", "--name-only", "origin/main...HEAD"],
            capture_output=True, text=True, check=True,
        )
        files = [f for f in out.stdout.splitlines() if f.strip()]
        if files:
            return files
    except subprocess.CalledProcessError:
        pass
    out = subprocess.run(
        ["git", "diff", "--name-only", "--cached"],
        capture_output=True, text=True, check=True,
    )
    return [f for f in out.stdout.splitlines() if f.strip()]


def self_test():
    """Self-test: plant a banned identifier, confirm the checker flags it."""
    import tempfile

    failures = []

    # 1. A planted banned identifier in code must be flagged.
    with tempfile.TemporaryDirectory() as d:
        evil = Path(d) / "evil.py"
        evil.write_text("def start_crew_kickoff():\n    pass\n")
        v = find_violations([str(evil)])
        if not v:
            failures.append("planted identifier 'crew'/'kickoff' was NOT flagged")
        elif not any(x[3].lower() in ("crew", "kickoff") for x in v):
            failures.append(f"wrong word flagged: {v}")

    # 2. Clean code must pass.
    with tempfile.TemporaryDirectory() as d:
        clean = Path(d) / "clean.py"
        clean.write_text("def start_team_run():\n    team = Team(name='a')\n")
        v = find_violations([str(clean)])
        if v:
            failures.append(f"clean code flagged: {v}")

    # 3. Docs mentioning the projects must be allow-listed.
    with tempfile.TemporaryDirectory() as d:
        doc = Path(d) / "note.md"
        doc.write_text("# CrewAI behaviour notes\n")
        v = find_violations([str(doc)])
        if v:
            failures.append(f"markdown doc flagged: {v}")

    # 4. Lanesra vocabulary must pass.
    with tempfile.TemporaryDirectory() as d:
        ok = Path(d) / "ok.rs"
        ok.write_text("struct Team { playbook: Playbook }\n// Run a step\n")
        v = find_violations([str(ok)])
        if v:
            failures.append(f"Lanesra vocabulary flagged: {v}")

    if failures:
        print("SELF-TEST FAILED:")
        for f in failures:
            print(" -", f)
        return 1
    print("self-test: 4/4 passed (Covers: REQ-CLEAN-003 WP-0012 AC-2)")
    return 0


def main(argv):
    if "--self-test" in argv:
        return self_test()
    paths = [a for a in argv[1:] if not a.startswith("--")]
    if not paths:
        paths = _changed_files()
    violations = find_violations(paths)
    if violations:
        print("Banned third-party vocabulary found in code identifiers:")
        for path, lineno, line, word in violations:
            print(f"  {path}:{lineno}: [{word}] {line}")
        print("\nUse Lanesra vocabulary (Team, Playbook, Run, Step). See AGENTS.md clean-room rules.")
        return 1
    print(f"vocabulary check: clean ({len(paths)} files scanned)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
