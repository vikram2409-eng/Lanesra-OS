#!/usr/bin/env python3
"""Modern UI PR-C: chained transform (a -> b -> c).

a: Admin Center reskin (issue #385)
b: Theme Studio TS-3 component overrides + TS-5 custom CSS + TS-6 brand assets
c: Theme Studio TS-4 layout templates + TS-7 versioning hardening + TS-8 permissions

Each sub-script asserts its anchors at match-count == 1 and fails loudly.
Run from the repo root: python3 scripts/apply_modern_pr_c.py
"""
import runpy

for s in ['scripts/apply_modern_pr_c_a.py',
          'scripts/apply_modern_pr_c_b.py',
          'scripts/apply_modern_pr_c_c.py']:
    print('running', s, flush=True)
    runpy.run_path(s, run_name='__main__')

print('PR-C chain complete: app.js + styles.css transformed')
