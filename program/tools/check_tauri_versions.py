#!/usr/bin/env python3
"""Fail if the desktop shell's Tauri crate and Tauri JavaScript API are on different minor releases.

`tauri build` refuses to build when the `tauri` Rust crate (desktop/Cargo.lock) and the
`@tauri-apps/api` npm package (desktop/package-lock.json) differ in major or minor version. CI
only compiles the shell, so a mismatch used to surface first in the Windows release build: the
desktop-v0.16.0 installer failed this way after a grouped dependency update moved the npm package
to 2.12 and left the crate on 2.11.

Usage: python3 program/tools/check_tauri_versions.py   (exit 1 and a message on a mismatch)
"""
import json
import re
import sys
from pathlib import Path


def crate_version(cargo_lock_text, name):
    m = re.search(r'\[\[package\]\]\nname = "' + re.escape(name) + r'"\nversion = "([^"]+)"', cargo_lock_text)
    return m.group(1) if m else None


def npm_version(package_lock, name):
    entry = package_lock.get("packages", {}).get(f"node_modules/{name}")
    return entry.get("version") if entry else None


def minor(version):
    return tuple(version.split(".")[:2])


def check(desktop):
    crate = crate_version((desktop / "Cargo.lock").read_text(), "tauri")
    api = npm_version(json.loads((desktop / "package-lock.json").read_text()), "@tauri-apps/api")
    if crate is None:
        return ["Cargo.lock has no tauri crate"]
    if api is None:
        return ["package-lock.json has no @tauri-apps/api package"]
    if minor(crate) != minor(api):
        return [
            f"tauri crate {crate} and @tauri-apps/api {api} are on different minor releases; "
            "tauri build refuses this. Move both to the same major.minor "
            "(cargo update -p tauri --precise <version>, or npm install @tauri-apps/api@<version>)."
        ]
    return []


def main():
    desktop = Path(__file__).resolve().parents[2] / "desktop"
    errors = check(desktop)
    for e in errors:
        print(e)
    if errors:
        sys.exit(1)
    print("tauri versions: ok")


if __name__ == "__main__":
    main()
