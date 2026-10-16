#!/usr/bin/env python3
"""Check feature flags across editions (WP-0010, ADR-0012, REQ-PRE-021).

Rules:
- Every flag in the Rust registry has a unique `area.feature` key, a work package and a
  `remove_by` value, and appears in `REGISTRY`.
- A `building` flag defaults off in every edition.
- The online demo (`app.js` `DEMO_RELEASED_FEATURES`) lists only flags whose stage is `Ga`:
  the demo shows a feature in the same release as desktop and Team Workspace, never earlier.

Usage: python3 program/tools/check_feature_flags.py   (exit 1 with a message per problem)
"""
import re
import sys
from pathlib import Path

FLAG_RE = re.compile(
    r"pub const (?P<const>[A-Z0-9_]+): Flag = Flag \{(?P<body>.*?)\n\};",
    re.S,
)


def parse_registry(rust_source):
    flags = {}
    for m in FLAG_RE.finditer(rust_source):
        body = m.group("body")

        def field(name, body=body):
            fm = re.search(rf'{name}:\s*"([^"]*)"', body)
            return fm.group(1) if fm else None

        stage = re.search(r"stage:\s*Stage::(\w+)", body)
        desktop = re.search(r"desktop:\s*(true|false)", body)
        workspace = re.search(r"workspace:\s*(true|false)", body)
        flags[m.group("const")] = {
            "key": field("key"),
            "work_package": field("work_package"),
            "remove_by": field("remove_by"),
            "stage": stage.group(1) if stage else None,
            "desktop": desktop and desktop.group(1) == "true",
            "workspace": workspace and workspace.group(1) == "true",
        }
    reg = re.search(r"pub const REGISTRY: &\[&Flag\] = &\[(.*?)\];", rust_source, re.S)
    registered = re.findall(r"&([A-Z0-9_]+)", reg.group(1)) if reg else []
    return flags, registered


def parse_demo(app_js):
    m = re.search(r"const DEMO_RELEASED_FEATURES=\[(.*?)\];", app_js, re.S)
    if not m:
        return None
    return re.findall(r"['\"]([^'\"]+)['\"]", m.group(1))


def check(rust_source, app_js):
    errors = []
    flags, registered = parse_registry(rust_source)
    keys = [f["key"] for f in flags.values()]
    if len(keys) != len(set(keys)):
        errors.append("duplicate feature flag key in the registry")
    for const, f in flags.items():
        if const not in registered:
            errors.append(f"{const} is defined but not listed in REGISTRY")
        if not f["key"] or not re.fullmatch(r"[a-z0-9_]+\.[a-z0-9_]+", f["key"]):
            errors.append(f"{const}: key {f['key']!r} must be lower snake case area.feature")
        if not (f["work_package"] or "").startswith("WP-"):
            errors.append(f"{const}: work_package must name a WP")
        if not f["remove_by"]:
            errors.append(f"{const}: remove_by is required")
        if f["stage"] == "Building" and (f["desktop"] or f["workspace"]):
            errors.append(f"{const}: a building flag must default off in every edition")
    for const in registered:
        if const not in flags:
            errors.append(f"REGISTRY lists {const} but no such flag is defined")
    demo = parse_demo(app_js)
    if demo is None:
        errors.append("app.js has no DEMO_RELEASED_FEATURES list")
    else:
        by_key = {f["key"]: f for f in flags.values()}
        for key in demo:
            f = by_key.get(key)
            if f is None:
                errors.append(f"app.js DEMO_RELEASED_FEATURES lists unknown flag {key!r}")
            elif f["stage"] != "Ga":
                errors.append(
                    f"app.js shows {key!r} in the demo, but it is {f['stage']} and not yet released "
                    "for desktop and Team Workspace (REQ-PRE-021)"
                )
    return errors


def main():
    root = Path(__file__).resolve().parents[2]
    rust = (root / "desktop/core/src/services/feature_flag_service.rs").read_text()
    app_js = (root / "app.js").read_text()
    errors = check(rust, app_js)
    for e in errors:
        print(e)
    if errors:
        sys.exit(1)
    print("feature flags: ok")


if __name__ == "__main__":
    main()
