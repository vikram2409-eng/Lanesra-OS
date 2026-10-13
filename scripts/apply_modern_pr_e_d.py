#!/usr/bin/env python3
"""Modern UI PR-E (issue #391): tighten the no-hardcoded-values lint.

Two false-positive classes remain on the migrated tree:
1. `.site-dark{...}` is a token-definition block (it redefines --bg/--panel/
   --text/--brand/... for the dark marketing pages), exactly like :root /
   [data-theme]. Its 15 literals are token VALUES, unmappable by definition.
2. Literals inside /* ... */ comments (#339, #352, #194, #359 are issue/epic
   numbers in comments, not colors).

This patch exempts the .site-dark block and skips comment spans (line numbers
are preserved — matches inside comments are filtered by offset).
Run from the repo root:  python3 scripts/apply_modern_pr_e_d.py
"""

PATH = "scripts/lint_no_hardcoded_css.py"


def fail(msg):
    raise SystemExit("CHAIN FAIL [lint-patch]: " + msg)


with open(PATH, encoding="utf-8") as f:
    src = f.read()

# 1) Exempt .site-dark token-definition blocks alongside :root / [data-theme].
# Anchor on the raw-string alternation head (single backslashes in source).
a_old = 're.compile(r"(:root|\\[data-theme'
a_new = 're.compile(r"(:root|\\.site-dark|\\[data-theme'
if src.count(a_old) != 1:
    fail(f"BLOCK_START anchor: expected 1 match, found {src.count(a_old)}")
src = src.replace(a_old, a_new, 1)

# 2) Skip literals that live inside /* ... */ comments.
b_old = """    stripped = strip_blocks(text)

    violations = []
    for lineno, line in enumerate(stripped.splitlines(), 1):"""
b_new = """    stripped = strip_blocks(text)
    # Comment spans in the stripped text (offsets). A literal inside a
    # comment is documentation (e.g. issue numbers like #359), not a color.
    comment_spans = []
    _ci = 0
    while True:
        _cs = stripped.find("/*", _ci)
        if _cs == -1:
            break
        _ce = stripped.find("*/", _cs + 2)
        _ce = len(stripped) if _ce == -1 else _ce + 2
        comment_spans.append((_cs, _ce))
        _ci = _ce

    def _in_comment(off):
        return any(s <= off < e for s, e in comment_spans)

    violations = []
    _off = 0
    for lineno, line in enumerate(stripped.splitlines(keepends=True), 1):"""
if src.count(b_old) != 1:
    fail(f"comment-filter anchor: expected 1 match, found {src.count(b_old)}")
src = src.replace(b_old, b_new, 1)

# 3) Apply the comment filter to both match loops, tracking offsets.
c_old = """    for lineno, line in enumerate(stripped.splitlines(keepends=True), 1):
        for m in HEX.finditer(line):
            violations.append((lineno, m.group(0)))
        for m in FUNC.finditer(line):
            violations.append((lineno, m.group(0).rstrip()))"""
c_new = """    for lineno, line in enumerate(stripped.splitlines(keepends=True), 1):
        for m in HEX.finditer(line):
            if not _in_comment(_off + m.start()):
                violations.append((lineno, m.group(0)))
        for m in FUNC.finditer(line):
            if not _in_comment(_off + m.start()):
                violations.append((lineno, m.group(0).rstrip()))
        _off += len(line)"""
if src.count(c_old) != 1:
    fail(f"match-loop anchor: expected 1 match, found {src.count(c_old)}")
src = src.replace(c_old, c_new, 1)

# 4) Update the success message to name the new exempt block.
d_old = 'print("OK: no hardcoded color literals outside :root / [data-theme] / @media blocks")'
d_new = 'print("OK: no hardcoded color literals outside :root / .site-dark / [data-theme] / @media blocks")'
if src.count(d_old) != 1:
    fail(f"ok-message anchor: expected 1 match, found {src.count(d_old)}")
src = src.replace(d_old, d_new, 1)

with open(PATH, "w", encoding="utf-8") as f:
    f.write(src)

print("lint script patched: .site-dark exempt, comment spans skipped")
