#!/usr/bin/env python3
"""Render the program-bot PR comment for agent work packages (WP-0018, REQ-PRE-014).

Reads the outputs of `programctl validate`, `programctl trace`, the work package
file and a JSON list of CI jobs, and writes one Markdown comment to stdout.
The comment starts with a hidden marker so the workflow can update it in place.

Covers: REQ-PRE-014 (WP-0018)
Untested locally - see CI.
"""

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

MARKER = "<!-- program-bot:report -->"

FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)
AC_RE = re.compile(r"^- \[( |x)\] (AC-\d+): (.*)$", re.MULTILINE)


def parse_frontmatter(text):
    """Parse the YAML frontmatter of a work package file (simple key: value)."""
    m = FRONTMATTER_RE.match(text)
    data = {}
    if not m:
        return data
    for line in m.group(1).splitlines():
        if ":" not in line:
            continue
        key, _, value = line.partition(":")
        data[key.strip()] = value.strip()
    return data


def parse_list(value):
    """Parse a '[a, b]' style list value from frontmatter."""
    value = value.strip()
    if value.startswith("[") and value.endswith("]"):
        inner = value[1:-1].strip()
        if not inner:
            return []
        return [v.strip() for v in inner.split(",")]
    return [value] if value else []


def parse_acceptance_criteria(text):
    """Return [(ac_id, ticked_bool, description)] from the Acceptance criteria section."""
    return [(m.group(2), m.group(1) == "x", m.group(3).strip())
            for m in AC_RE.finditer(text)]


def parse_validate(output):
    """Parse `programctl validate` output into a dict."""
    m = re.search(
        r"(\d+) work packages, (\d+) requirements, (\d+) error\(s\), (\d+) warning\(s\)",
        output,
    )
    if not m:
        return {"raw": output.strip(), "ok": False}
    return {
        "packages": int(m.group(1)),
        "requirements": int(m.group(2)),
        "errors": int(m.group(3)),
        "warnings": int(m.group(4)),
        "ok": int(m.group(3)) == 0,
    }


def filter_trace(trace_output, requirements):
    """Return the trace table rows for the given requirement ids."""
    rows = []
    for line in trace_output.splitlines():
        line = line.strip()
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if cells and cells[0] in requirements:
            rows.append(line)
    return rows


def job_emoji(conclusion):
    return {
        "success": "\u2705",
        "failure": "\u274c",
        "cancelled": "\u26aa",
        "skipped": "\u23ed\ufe0f",
    }.get(conclusion, "\U0001f7e1")


def render(args):
    wp_text = Path(args.wp_file).read_text(encoding="utf-8")
    fm = parse_frontmatter(wp_text)
    wp_id = fm.get("id", "?")
    title = fm.get("title", "")
    status = fm.get("status", "?")
    editions = parse_list(fm.get("editions", "[]"))
    requirements = parse_list(fm.get("requirements", "[]"))
    acs = parse_acceptance_criteria(wp_text)

    validate = parse_validate(Path(args.validate_output).read_text(encoding="utf-8"))
    trace_rows = filter_trace(Path(args.trace_output).read_text(encoding="utf-8"), requirements)
    jobs = json.loads(Path(args.jobs_json).read_text(encoding="utf-8"))

    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    lines = [MARKER, f"## Program bot report for {wp_id}", ""]

    # Validation
    lines.append("### Validation")
    if validate.get("ok"):
        lines.append(
            f"\u2705 `programctl validate`: {validate['packages']} work packages, "
            f"{validate['requirements']} requirements, "
            f"{validate['errors']} error(s), {validate['warnings']} warning(s)"
        )
    else:
        lines.append("\u274c `programctl validate` failed:")
        lines.append("```")
        lines.append(validate.get("raw", "(no output)"))
        lines.append("```")
    lines.append("")

    # Work package summary
    lines.append("### Work package")
    lines.append(f"- **{wp_id}**: {title}")
    lines.append(f"- **Status**: {status}")
    lines.append(f"- **Editions**: {', '.join(editions) if editions else '(none - program tracking only)'}")
    lines.append(f"- **Requirements**: {', '.join(requirements) if requirements else '(none)'}")
    lines.append("")

    # Acceptance criteria
    lines.append("### Acceptance criteria")
    if acs:
        for ac_id, ticked, desc in acs:
            box = "x" if ticked else " "
            lines.append(f"- [{box}] {ac_id}: {desc}")
    else:
        lines.append("(no acceptance criteria found in the work package file)")
    lines.append("")

    # Requirement trace
    lines.append("### Requirement trace")
    if trace_rows:
        lines.extend(trace_rows)
    else:
        lines.append("(no trace rows for this work package's requirements)")
    lines.append("")

    # CI jobs
    lines.append("### CI jobs")
    lines.append(f"Run: {args.run_url}")
    lines.append("")
    lines.append("| Job | Result |")
    lines.append("|---|---|")
    for job in jobs:
        emoji = job_emoji(job.get("conclusion"))
        name = job.get("name", "?")
        url = job.get("html_url", "")
        if job.get("conclusion") == "failure" and url:
            lines.append(f"| {name} | {emoji} [logs]({url}) |")
        else:
            lines.append(f"| {name} | {emoji} |")
    lines.append("")
    lines.append("---")
    lines.append(f"*Untested locally - see CI. Updated {now}.*")
    lines.append("")
    return "\n".join(lines)


def main():
    p = argparse.ArgumentParser(description="Render the program-bot PR comment.")
    p.add_argument("--validate-output", required=True)
    p.add_argument("--wp-file", required=True)
    p.add_argument("--trace-output", required=True)
    p.add_argument("--jobs-json", required=True)
    p.add_argument("--run-url", required=True)
    args = p.parse_args()
    sys.stdout.write(render(args))


if __name__ == "__main__":
    main()
