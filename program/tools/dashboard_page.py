#!/usr/bin/env python3
"""Wrap the dashboard page source as a standalone HTML document (WP-0019).

program/dashboard/index.html is written for the claude.ai artifact host,
which adds the <!doctype>, <head> and <body> itself. GitHub Pages serves
files as they are, so this adds the same minimal skeleton: UTF-8, a
viewport, and the `[hidden]` rule the page relies on.

    python3 program/tools/dashboard_page.py program/dashboard/index.html _site/index.html
"""
import sys
from pathlib import Path

HEAD = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<style>[hidden]{display:none!important} body{margin:0} img{max-width:100%}</style>
"""


def wrap(source: str) -> str:
    # The page's own <title>, <meta>, <link> and <style> stay in <head>; the rest is <body>.
    marker = "</style>"
    cut = source.find(marker)
    if cut == -1:
        raise SystemExit("error: expected the page's <style> block near the top")
    cut += len(marker)
    return HEAD + source[:cut] + "\n</head>\n<body>\n" + source[cut:] + "\n</body>\n</html>\n"


def main(argv):
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    src, out = Path(argv[1]), Path(argv[2])
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(wrap(src.read_text(encoding="utf-8")), encoding="utf-8")
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
