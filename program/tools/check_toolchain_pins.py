#!/usr/bin/env python3
"""Fail if the Team Workspace image's Rust or Node version drifts from the repo's pins (WP-0020).

The pins are `desktop/rust-toolchain.toml` (Rust) and `desktop/.nvmrc` (Node major version). The
Dockerfile declares `ARG RUST_VERSION=...` and `ARG NODE_VERSION=...`; both must match, or the
server image is built with a different compiler or runtime from the one CI tests.

Usage: python3 program/tools/check_toolchain_pins.py   (exit 1 and a message on drift)
"""
import re
import sys
from pathlib import Path


def read_pins(desktop):
    toolchain = (desktop / "rust-toolchain.toml").read_text()
    m = re.search(r'^\s*channel\s*=\s*"([^"]+)"', toolchain, re.M)
    rust = m.group(1) if m else None
    node = (desktop / ".nvmrc").read_text().strip().lstrip("v")
    return rust, node


def read_dockerfile_args(dockerfile):
    text = dockerfile.read_text()
    args = dict(re.findall(r"^ARG\s+(\w+)=(\S+)", text, re.M))
    return args.get("RUST_VERSION"), args.get("NODE_VERSION")


def check(desktop):
    errors = []
    rust_pin, node_pin = read_pins(desktop)
    rust_img, node_img = read_dockerfile_args(desktop / "Dockerfile")
    if rust_pin is None:
        errors.append("rust-toolchain.toml has no channel")
    if rust_img != rust_pin:
        errors.append(f"Dockerfile RUST_VERSION={rust_img} but rust-toolchain.toml pins {rust_pin}")
    if node_img is None or node_img.split(".")[0] != node_pin.split(".")[0]:
        errors.append(f"Dockerfile NODE_VERSION={node_img} but .nvmrc pins {node_pin}")
    return errors


def main():
    desktop = Path(__file__).resolve().parents[2] / "desktop"
    errors = check(desktop)
    for e in errors:
        print(e)
    if errors:
        sys.exit(1)
    print("toolchain pins: ok")


if __name__ == "__main__":
    main()
