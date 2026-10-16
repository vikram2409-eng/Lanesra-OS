#!/usr/bin/env python3
"""Fail if a tracked image or icon is not the binary format its extension names (WP-0003).

On 2026-10-14 eleven brand images were committed as base64 text instead of binary. The desktop
shell stopped compiling (`tauri::generate_context!` reads the icons) and the website and app
showed broken logos, and no check noticed. This check reads the first bytes of every tracked
asset and compares them with the format's file signature.

Usage: python3 program/tools/check_binary_assets.py   (exit 1 and a list of files on failure)
"""
import subprocess
import sys
from pathlib import Path

SIGNATURES = {
    ".png": [b"\x89PNG\r\n\x1a\n"],
    ".jpg": [b"\xff\xd8\xff"],
    ".jpeg": [b"\xff\xd8\xff"],
    ".gif": [b"GIF87a", b"GIF89a"],
    ".ico": [b"\x00\x00\x01\x00"],
    ".icns": [b"icns"],
    ".webp": [b"RIFF"],
    ".woff2": [b"wOF2"],
    ".woff": [b"wOFF"],
}


def check(paths):
    """Return one error line per file whose first bytes do not match its extension."""
    errors = []
    for path in paths:
        expected = SIGNATURES.get(path.suffix.lower())
        if not expected or not path.is_file():
            continue
        head = path.read_bytes()[:16]
        if not any(head.startswith(sig) for sig in expected):
            hint = " (looks like base64 text: decode it with `base64 -d`)" if head[:4].isascii() and head[:4].isalnum() else ""
            errors.append(f"{path}: not a valid {path.suffix} file{hint}")
    return errors


def tracked_files(root):
    out = subprocess.run(["git", "ls-files", "-z"], cwd=root, capture_output=True, check=True).stdout
    return [root / p for p in out.decode().split("\0") if p]


def main():
    root = Path(__file__).resolve().parents[2]
    errors = check(tracked_files(root))
    for e in errors:
        print(e)
    if errors:
        sys.exit(1)
    print("binary assets: ok")


if __name__ == "__main__":
    main()
