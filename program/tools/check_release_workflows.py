#!/usr/bin/env python3
"""Fail if the desktop and Team Workspace release workflows get tangled together (ADR-0019).

Desktop and Team Workspace are separate releases with separate tags and release pages:
  desktop         `desktop-v*`  -> desktop-release.yml, publish-release.yml
  Team Workspace  `team-workspace-server-v*`   -> server-image-release.yml, publish-server-release.yml

A tag push must start exactly one of the two, and the desktop publisher must never touch the
server image or its `:latest` tag. This reads the workflow files as text (standard library only).

Usage: python3 program/tools/check_release_workflows.py   (exit 1 and a message on a problem)
"""
import re
import sys
from pathlib import Path

DESKTOP = ["desktop-release.yml", "publish-release.yml"]
SERVER = ["server-image-release.yml", "publish-server-release.yml"]


def tag_patterns(text):
    """The patterns under `on: push: tags:`."""
    m = re.search(r"^\s*tags:\s*\n((?:\s+-\s*\S+\s*\n)+)", text, re.M)
    if not m:
        return []
    return [p.strip().strip("\"'") for p in re.findall(r"-\s*(\S+)", m.group(1))]


def check(workflows):
    errors = []

    def read(name):
        path = workflows / name
        if not path.exists():
            errors.append(f"{name} is missing")
            return None
        return path.read_text()

    desktop_build = read("desktop-release.yml")
    server_build = read("server-image-release.yml")
    for name in DESKTOP + SERVER:
        read(name)
    if desktop_build is not None and tag_patterns(desktop_build) != ["desktop-v*"]:
        errors.append(f"desktop-release.yml must start only on desktop-v* tags, not {tag_patterns(desktop_build)}")
    if server_build is not None and tag_patterns(server_build) != ["team-workspace-server-v*"]:
        errors.append(f"server-image-release.yml must start only on team-workspace-server-v* tags, not {tag_patterns(server_build)}")

    for name in DESKTOP:
        text = read(name)
        if text is None:
            continue
        for needle in ("ghcr.io", "lanesra-os-server", "packages: write", "team-workspace-server-v"):
            if needle in re.sub(r"(?m)^\s*#.*$", "", text):
                errors.append(f"{name} is a desktop workflow and must not mention '{needle}'")
    for name in SERVER:
        text = read(name)
        if text is None:
            continue
        if "desktop-v" in re.sub(r"(?m)^\s*#.*$", "", text):
            errors.append(f"{name} is a Team Workspace workflow and must not use desktop-v tags")
    return errors


def main():
    workflows = Path(__file__).resolve().parents[2] / ".github" / "workflows"
    errors = check(workflows)
    for e in errors:
        print(e)
    if errors:
        sys.exit(1)
    print("release workflows: ok")


if __name__ == "__main__":
    main()
