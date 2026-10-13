#!/usr/bin/env python3
"""Modern UI PR-D — fidelity pass chain runner (issue #395).

Runs the four worker transform scripts in order against the files in CWD:
  d_1: shell + dashboard + lists + pipeline
  d_2: record detail pages
  d_3: ontology graph workbench + Object 360 modal
  d_4: admin center + Theme Studio

Usage: copy pristine app.js + styles.css into a work dir, cd there, then
  python3 /path/to/apply_modern_pr_d.py

CRITICAL (lesson from PR-C): worker scripts guard on `if __name__ == '__main__'`,
so they must be executed via runpy.run_path(s, run_name='__main__') — plain
runpy.run_path silently applies nothing.

d_1 and d_2 operate in-place on ./app.js and ./styles.css (CWD-relative), so
this runner chdir's into the work dir first. d_3 takes --app-in/--css-in/--app-out/--css-out.
d_4 takes its two paths from sys.argv[1], sys.argv[2] (read at module level).
sys.argv is set appropriately before each runpy invocation.
"""
import os
import runpy
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
WORKERS = [
    "apply_modern_pr_d_1.py",
    "apply_modern_pr_d_2.py",
    "apply_modern_pr_d_3.py",
    "apply_modern_pr_d_4.py",
]


def main():
    workdir = os.getcwd()
    for f in ("app.js", "styles.css"):
        if not os.path.isfile(os.path.join(workdir, f)):
            sys.exit("CHAIN FAIL: %s not found in %s" % (f, workdir))
    app = os.path.join(workdir, "app.js")
    css = os.path.join(workdir, "styles.css")

    # d_1: in-place on ./app.js, ./styles.css
    os.chdir(workdir)
    sys.argv = [os.path.join(HERE, WORKERS[0])]
    print("=== PR-D/1 ===")
    runpy.run_path(os.path.join(HERE, WORKERS[0]), run_name="__main__")

    # d_2: in-place on ./app.js, ./styles.css
    sys.argv = [os.path.join(HERE, WORKERS[1])]
    print("=== PR-D/2 ===")
    runpy.run_path(os.path.join(HERE, WORKERS[1]), run_name="__main__")

    # d_3: argparse paths (in-place via in==out)
    sys.argv = [
        os.path.join(HERE, WORKERS[2]),
        "--app-in", app, "--css-in", css,
        "--app-out", app, "--css-out", css,
    ]
    print("=== PR-D/3 ===")
    runpy.run_path(os.path.join(HERE, WORKERS[2]), run_name="__main__")

    # d_4: paths from sys.argv[1], sys.argv[2] (read at module level)
    sys.argv = [os.path.join(HERE, WORKERS[3]), app, css]
    print("=== PR-D/4 ===")
    runpy.run_path(os.path.join(HERE, WORKERS[3]), run_name="__main__")

    print("PR-D CHAIN COMPLETE")


if __name__ == "__main__":
    main()
