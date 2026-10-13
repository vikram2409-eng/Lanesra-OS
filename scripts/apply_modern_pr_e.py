#!/usr/bin/env python3
"""Modern UI PR-E chained transform: a (public pages) -> c (cutover/dead CSS)
-> b (lint token migration).

Order rationale:
- a before c: both touch index.html's theme-color line; a's anchor expects the
  pristine `#0A0A0A` value, c rewrites it to the light default.
- c before b: c deletes dead CSS; b migrates literals in live CSS. The two
  scripts were verified to have zero overlapping rules (b adopted c's
  boundary-aware liveness criterion).
- a's styles.css append is EOF-only; b/c anchors target existing blocks.
- d (lint-script tightening) runs last; it only touches
  scripts/lint_no_hardcoded_css.py, independent of the CSS/app transforms.

Target directory: argv[1] if given, else the current working directory.
Each worker script is executed with sys.argv[1] set to the target (script a
reads its REPO from argv) and cwd set to the target (scripts b and c use
cwd-relative paths).

Run from the repo root:  python3 scripts/apply_modern_pr_e.py
"""
import os
import runpy
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
TARGET = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()

for name in ("apply_modern_pr_e_a.py", "apply_modern_pr_e_c.py", "apply_modern_pr_e_b.py", "apply_modern_pr_e_d.py"):
    script = HERE / name
    assert script.exists(), f"missing chained script: {script}"
    print(f"=== running {name} on {TARGET} ===")
    # runpy with run_name='__main__': the worker scripts guard on
    # `if __name__ == '__main__'`; plain runpy.run_path would silently
    # apply nothing (lesson from PR-C). argv[1] carries the target dir
    # (script a reads REPO from it); cwd covers the cwd-relative scripts.
    os.chdir(TARGET)
    sys.argv = [str(script), str(TARGET)]
    runpy.run_path(str(script), run_name="__main__")

print("=== PR-E chain complete: a -> c -> b ===")
