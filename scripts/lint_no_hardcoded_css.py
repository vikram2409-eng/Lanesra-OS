#!/usr/bin/env python3
"""Fail if styles.css contains hardcoded color literals outside the exempt blocks.

Exempt: :root blocks, [data-theme] blocks, and @media blocks (token definitions
and the reduced-motion override live there). Everything else — components, shell,
legacy page CSS — must use var(--token) references (color-mix() is allowed).

Prints <file>:<line>: <literal> for each violation and exits 1 if any are found.
"""
import re
import sys

PATH = sys.argv[1] if len(sys.argv) > 1 else "styles.css"

HEX = re.compile(r"#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{4}|[0-9a-fA-F]{3})(?![0-9a-zA-Z_-])")
FUNC = re.compile(r"\b(?:rgb|rgba|hsl|hsla)\s*\(")


BLOCK_START = re.compile(r"(:root|\[data-theme[^\]\n]*\]|@media[^{\n]*)\{")


def strip_blocks(text):
    """Remove balanced-brace :root / [data-theme] / @media blocks.

    The regex requires the opening brace on the same line as the selector, so
    keyword mentions inside comments (no adjacent brace) are left alone.
    """
    out = []
    i, n = 0, len(text)
    for m in BLOCK_START.finditer(text):
        # sanity: selector text must not contain comment or declaration chars
        sel = m.group(1)
        if any(c in sel for c in "*/;"):
            continue
        out.append(text[i:m.start()])
        depth = 0
        k = m.end() - 1  # the opening brace
        while k < n:
            if text[k] == "{":
                depth += 1
            elif text[k] == "}":
                depth -= 1
                if depth == 0:
                    break
            k += 1
        i = k + 1  # skip the whole block
    out.append(text[i:])
    return "".join(out)


def main():
    with open(PATH, encoding="utf-8") as f:
        text = f.read()
    stripped = strip_blocks(text)

    violations = []
    for lineno, line in enumerate(stripped.splitlines(), 1):
        for m in HEX.finditer(line):
            violations.append((lineno, m.group(0)))
        for m in FUNC.finditer(line):
            violations.append((lineno, m.group(0).rstrip()))

    for lineno, lit in violations:
        print(f"{PATH}:{lineno}: {lit}")
    if violations:
        print(f"\n{len(violations)} hardcoded color literal(s) outside exempt blocks", file=sys.stderr)
        return 1
    print("OK: no hardcoded color literals outside :root / [data-theme] / @media blocks")
    return 0


if __name__ == "__main__":
    sys.exit(main())
